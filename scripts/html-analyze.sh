#!/usr/bin/env bash

set -x # Show commands
set -eu # Errors/undefined vars are fatal
set -o pipefail # Check all commands in a pipeline

if [ $# -ne 2 ]
then
    echo "Usage: html-analyze.sh"
    exit 1
fi

TOTAL_FILES=$(cat $INDEX_ROOT/html-files | wc -l)
JOB_COUNT=8

parallel --jobs $JOB_COUNT --pipepart -a $INDEX_ROOT/html-files \
    --block -1 --halt 2 \
    js -f $MOZSEARCH_PATH/scripts/js-analyze.js -- \
    {#} $JOB_COUNT $TOTAL_FILES $MOZSEARCH_PATH $FILES_ROOT $INDEX_ROOT/analysis
echo $?
