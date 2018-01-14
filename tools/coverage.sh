#!/usr/bin/env bash

for COMMAND in cargo grep jq kcov pgrep sed; do
    if ! command -v "$COMMAND" > /dev/null; then
        echo "Required command '$COMMAND' not found"
        exit 1
    fi
done

BASE_DIR="$(dirname "$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )")"
cd "$BASE_DIR" || exit 1

CHROME=$(command -v google-chrome-stable || echo -n "/dev/null")
FIREFOX=$(command -v firefox || echo -n "/dev/null")

BINARIES=$(cargo test --no-run --message-format=json 2>/dev/null | \
    jq '.filenames' | \
    sed -r '/^\[|\]/d' | \
    sed -r '/(\.rlib|\.so|build-script-build)"$/d' | \
    sed -r 's/^\s+//' | \
    sed -r 's/^"(.*)"$/\1/')
IFS=$'\n'
for BINARY_PATH in "$BINARIES"; do
    TEST_COUNT=$("$BINARY_PATH" --list | \
        grep -P "^\d+ tests" | \
        sed -r 's/^([0-9]+) tests.*$/\1/')
    if [[ "$TEST_COUNT" -gt 0 ]]; then
        BINARY=$(basename "$BINARY_PATH")
        COVERAGE_DIR="target/cov/$BINARY"
        COVERAGE_REPORT="$COVERAGE_DIR/index.html"

        mkdir -p "$COVERAGE_DIR" || exit 1
        kcov \
            --verify \
            --include-path="$BASE_DIR/src" \
            --exclude-path="$BASE_DIR/src/tests" \
            "$COVERAGE_DIR" \
            "$BINARY_PATH" || exit 1

        if [[ -x "$CHROME" ]] && pgrep -x $(basename "$CHROME") > /dev/null; then
            echo "Opening coverage report in Chrome."
            "$CHROME" "$COVERAGE_REPORT" chrome://newtab/
        elif [[ -x "$FIREFOX" ]] && pgrep -x $(basename "$FIREFOX") > /dev/null; then
            echo "Opening coverage report in Firefox."
            "$FIREFOX" -new-tab -url "$COVERAGE_REPORT"
        else
            echo "Coverage report: $COVERAGE_REPORT"
        fi
    fi
done
