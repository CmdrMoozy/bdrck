#!/usr/bin/env bash

BASE_DIR="$(dirname "$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )")"
cd "$BASE_DIR" || exit 1

cargo build || exit 1
cargo test || exit 1

kcov \
    --verify \
    --include-path="$BASE_DIR/src" \
    --exclude-path="$BASE_DIR/src/tests" \
    target/cov \
    target/debug/bdrck-d26078de399a9663 || exit 1

CHROME=$(command -v google-chrome-stable || echo -n "/dev/null")
FIREFOX=$(command -v firefox || echo -n "/dev/null")

if [ -x "$CHROME" ] && pgrep -x chrome > /dev/null; then
    echo "Opening coverage report in Chrome."
    "$CHROME" "$BASE_DIR/target/cov/index.html" chrome://newtab/
elif [ -x "$FIREFOX" ] && pgrep -x firefox > /dev/null; then
    echo "Opening coverage report in Firefox."
    "$FIREFOX" -new-tab -url "$BASE_DIR/target/cov/index.html"
else
    echo "Coverage report: $BASE_DIR/target/cov/index.html"
fi
