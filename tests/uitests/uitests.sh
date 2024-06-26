#!/usr/bin/env bash

set -e

if ! [ -d uitest-env ]; then
    python3 -m venv uitest-env
fi

./uitest-env/bin/pip3 install selenium
cargo install geckodriver

MOZSEARCH_PATH=$(pwd)

if ! [ -d mozsearch-firefox ]; then
    curl -L -o mozsearch-firefox.tar.bz2 "https://download.mozilla.org/?product=firefox-latest&os=linux64"
    tar xf mozsearch-firefox.tar.bz2
    mv firefox mozsearch-firefox
fi

FIREFOX_BINARY=$MOZSEARCH_PATH/mozsearch-firefox/firefox ./uitest-env/bin/python3 tests/uitests/uitests.py
