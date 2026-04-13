#!/bin/bash

set -e

echo "Building 3W2Coords..."

cargo build --release

OS=$(uname -s)
EXT=""
if [ "$OS" = "Windows" ] || [ "$OS" = "MINGW"* ] || [ "$OS" = "MSYS"* ]; then
    EXT=".exe"
fi

EXECUTABLE="target/release/3W2Coords${EXT}"

if [ -f "$EXECUTABLE" ]; then
    echo "Build successful: $EXECUTABLE"
else
    echo "Error: Executable not found at $EXECUTABLE"
    exit 1
fi

echo "Done!"