#!/usr/bin/env python3

from __future__ import absolute_import
import json
import sys
import socket
import os
import os.path
import time
from logger import log

# A workaround until https://github.com/grpc/grpc/pull/37666 gets merged.
import warnings
warnings.filterwarnings("ignore", "Protobuf gencode version 5.27.2 is older than the runtime version 5.28.2", UserWarning)

import grpc
from livegrep import livegrep_pb2
from livegrep import livegrep_pb2_grpc

def collateMatches(matches):
    paths = {}
    for m in matches:
        # For results in the "mozilla-subrepo" repo, which is the mozilla/
        # subfolder of comm-central, we need to adjust the path to reflect
        # the fact that it's in the subfolder.
        path = m.path
        if m.tree == 'mozilla-subrepo':
            path = 'mozilla/' + path

        line = {
            'lno': m.line_number,
            'bounds': [m.bounds.left, m.bounds.right],
            'line': m.line
        }

        if len(m.context_before):
            # The before context is provided in reverse order which is not what
            # we want.
            before = list(m.context_before)
            # This does not return the list, so it's on its own line.
            before.reverse()
            line['context_before'] = before
        if len(m.context_after):
            line['context_after'] = list(m.context_after)

        paths.setdefault(path, []).append(line)
    results = [ {'path': p, 'icon': '', 'lines': paths[p]} for p in paths ]
    return results

def do_search(host, port, pattern, fold_case, file, context_lines):
    t = time.time()
    use_file = []
    # file is now a repeatd arg; if we pass an empty array, no constraints are
    # applied.  For efficiency, we convert a useless wildcard of '.*' to just be
    # no constraint.
    if file and file != '.*':
        use_file.append(file)
    query = livegrep_pb2.Query(line = pattern, file = use_file, fold_case = fold_case,
                               context_lines = context_lines)
    log('QUERY %s', repr(query).replace('\n', ', '))

    channel = grpc.insecure_channel('{0}:{1}'.format(host, port))
    grpc_stub = livegrep_pb2_grpc.CodeSearchStub(channel)
    result = grpc_stub.Search(query) # maybe add a timeout arg here?
    channel.close()

    matches = collateMatches(result.results)
    log('  codesearch result with %d line matches across %d paths - %f : %s',
        len(result.results), len(matches), time.time() - t,
        repr(result.stats).replace('\n', ', '))
    return (matches, livegrep_pb2.SearchStats.ExitReason.Name(result.stats.exit_reason) == 'TIMEOUT',
            livegrep_pb2.SearchStats.ExitReason.Name(result.stats.exit_reason) == 'MATCH_LIMIT')


def stop_codesearch(stat_file, ports):
    data = load_stat_file(stat_file)
    if data is not None:
        owner_pid = data['owner_pid']
        log('Stopping codesearch.py')
        # NOTE: Use SIGTERM here for the codesearch.py itself to let
        #       infrastructure/with-auto-restart.sh gracefully stop.
        os.system(f'kill {owner_pid}')

    for port in ports:
        log('Stopping codesearch on port %d', port)
        os.system("pkill -f '^codesearch.+localhost:%d '" % port)


def startup_codesearch_with(index_path, port):
    log('Starting codesearch on port %d', port)

    use_threads = 4
    try:
        # Defined to return None if "undetermined", so we handle that but also
        # are prepared for things to throw.
        maybe_count = os.cpu_count()
        # Limit us to 8 cores primarily to avoid the Vagrant VM getting too
        # resource hungry.  We may tend to want to give it as many cores as
        # possible for rust compilation, but our current EC2 core max is 8.
        if maybe_count is not None:
            use_threads = min(8, maybe_count)
    except:
        pass

    args = ['codesearch', '-grpc', 'localhost:' + str(port),
            '--noreuseport',
            '-load_index', index_path,
            # Note that because multiple threads are involved, this limit
            # potentially will not return the same results every time it is run
            # and that's okay.  But because of our app-level caching, it ends
            # up that we will usually only run one exact query once.
            '-max_matches', '4000',
            '-threads', f'{use_threads}',
            # We set the timeout to 30 seconds up from 10 seconds because our
            # caching policy requires our searches to be deterministic in the
            # face of I/O slowness.  Note that this differs from the
            # non-determinism of the "max_matches" limit which is acceptable.
            # We do expect to have addressed this problem by ensuring the cache
            # is fully loaded via vmtouch before serving begins in earnest.
            '-timeout', '30000',
            '-context_lines', '0']

    # Dump our arguments to the log so someone investigating things can just
    # kill the server and then copy and paste the arguments to run it
    # non-daemonized.  (Unfortunately, we don't have a way to get at its output
    # otherwise because of how we daemonize it.)
    log(' '.join(args))

    import subprocess

    return subprocess.Popen(args,
                            stdin=subprocess.DEVNULL,
                            stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL)

def update_stat_file(stat_file, port, curr_p, prev_p):
    stat_file_data = {
        'owner_pid': os.getpid(),
        'port': port,
        'pid': curr_p.pid,
    }
    if prev_p:
        stat_file_data['prev_pid'] = prev_p.pid

    tmp_file = stat_file + '.tmp'
    with open(tmp_file, 'w') as f:
        json.dump(stat_file_data, f)

    os.replace(tmp_file, stat_file)


def load_stat_file(stat_file):
    try:
        with open(stat_file) as f:
            return json.load(f)
    except:
        return None


def startup_codesearch(stat_file, index_path, ports):
    log('Setting up codesearch, using ports [%d, %d]', ports[0], ports[1])

    if os.path.exists(stat_file):
        log('Found previous stat file %s . Removing...', stat_file)
        os.unlink(stat_file)

    # Periodically restart the codesearch process, to workaround the slow-ness
    # after running long.
    #
    # We use two ports A and B, and perform the following:
    #
    #   1. start the codesearch with port A
    #   2. update the stat JSON file, pointing port A
    #   3. wait 3 hours
    #     - at this point, clients use the codesearch on the port A
    #   4. start the codesearch with port B
    #   5. update the stat JSON file, pointing port B
    #   6. wait 10 minutes
    #     - existing clients communicating with the codesearch on the port A
    #       should finish within this period
    #     - new clients start using the codesearch on the port B
    #   7. stop the codesearch on the port A
    #   8. wait 2 hours 50 minutes
    #     - at this point, clients use the codesearch on the port B
    #   9. go to step 4, with swapping the port A and the port B
    #
    # If the codesearch process gets killed, for example with OOM,
    # automatically restart it.

    index = 0
    prev_p = None
    while True:
        port = ports[index]
        p = startup_codesearch_with(index_path, port)
        log('Started proccess %d', p.pid)

        wait_for_codesearch(None, port)

        update_stat_file(stat_file, port, p, prev_p)

        try:
            ret = p.wait(timeout=10 * 60)
            log('The codesearch process exit with %d', ret)
            p = None
        except:
            pass

        if prev_p:
            log('Terminating previous proccess %d', prev_p.pid)
            prev_p.terminate()
        prev_p = p

        if p:
            update_stat_file(stat_file, port, p, None)

            try:
                ret = p.wait(timeout=3 * 60 * 60 - 10 * 60)
                log('The codesearch process exit with %d', ret)
                p = None
            except:
                pass

        index = 1 - index

def try_info_request(host, port):
    infoq = livegrep_pb2.InfoRequest()

    channel = grpc.insecure_channel('{0}:{1}'.format(host, port))
    grpc_stub = livegrep_pb2_grpc.CodeSearchStub(channel)
    result = grpc_stub.Info(infoq) # maybe add a timeout arg here?
    channel.close()

def wait_for_codesearch(stat_file, port=None, max_tries=200):
    '''Wait for the codesearch server to become available/responsive.'''

    tries = 0
    found = False
    while tries < max_tries:
        tries += 1

        if port is None:
            data = load_stat_file(stat_file)
            if data is None:
                time.sleep(0.1)
                continue
            port = data['port']

        try:
            try_info_request('localhost', port)
            found = True
            break
        except Exception as e:
            # sleep a little to give the server time to make progress
            time.sleep(0.1)

    if found:
        log('Server on port %d found alive after %d tries', port, tries)
    else:
        log('Server not found after %d tries', tries)

def search(stat_file, pattern, fold_case, path, tree_name, context_lines):
    data = load_stat_file(stat_file)
    if data is None:
        return ([], False, False)

    port = data['port']

    try:
        return do_search('localhost', port, pattern, fold_case, path, context_lines)
    except Exception as e:
        log('Got exception: %s', repr(e))
        # TODO: better job of surfacing the error back to the user. This might be e.g.
        # a grpc.StatusCode.INVALID_ARGUMENT if say the `pattern` is a malformed regex
        return ([], False, False)

def load(config, stop=True, start=True, only_tree_name=None):
    for tree_name in config['trees']:
        if only_tree_name and tree_name != only_tree_name:
            continue

        index_path = config['trees'][tree_name]['codesearch_path']
        ports = config['trees'][tree_name]['codesearch_ports']
        stat_file = config['trees'][tree_name]['codesearch_stat']

        if stop:
            stop_codesearch(stat_file, ports)
        if start:
            startup_codesearch(stat_file, index_path, ports)
            wait_for_codesearch(stat_file)


if __name__ == '__main__':
    '''(Re)start or stop all the codesearch instances for the given config file.

    Usage:
    codesearch.py CONFIG.JSON start [only_tree_name]
    codesearch.py CONFIG.JSON stop [only_tree_name]
    '''
    stop = True
    start = True
    only_tree_name = None
    if sys.argv[2] == 'stop':
        start = False
    if len(sys.argv) > 3:
        only_tree_name = sys.argv[3]

    config = json.load(open(sys.argv[1]))
    load(config, stop=stop, start=start, only_tree_name=only_tree_name)
