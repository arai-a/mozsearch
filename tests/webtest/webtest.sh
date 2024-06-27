#!/usr/bin/env bash

set -e

cargo install geckodriver

if ! [ -d mozsearch-firefox ]; then
    curl -L -o mozsearch-firefox.tar.bz2 "https://download.mozilla.org/?product=firefox-latest&os=linux64"
    tar xf mozsearch-firefox.tar.bz2
    mv firefox mozsearch-firefox
fi

kill_geckodriver() {
    PID=$(pgrep geckodriver)
    if [ "x${PID}" != "x" ]; then
        echo "Killing geckodriver: PID=${PID}"
        kill $PID
    fi
}

make build-rust-tools

set +e

kill_geckodriver

echo "Starting geckodriver"
geckodriver -b /vagrant/mozsearch-firefox/firefox >/dev/null 2>&1 &

echo "Running tests"
./tools/target/release/searchfox-tool "web-test"

kill_geckodriver
