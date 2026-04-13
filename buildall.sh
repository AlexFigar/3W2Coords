#!/bin/bash

set -e

echo "Building 3W2Coords for all platforms..."

mkdir -p target/release

check_target() {
    rustup target list --installed | grep -q "^$1$"
}

build_target() {
    local target=$1
    local output=$2
    
    if check_target "$target"; then
        echo "Building for $output..."
        cargo build --release --target "$target" 2>/dev/null && {
            local bin_path="target/$target/release/3W2Coords.exe"
            if [ -f "$bin_path" ]; then
                cp "$bin_path" "target/release/$output"
                echo "  -> target/release/$output"
                return 0
            fi
            local bin_path2="target/$target/release/3W2Coords"
            if [ -f "$bin_path2" ]; then
                cp "$bin_path2" "target/release/$output"
                echo "  -> target/release/$output"
                return 0
            fi
        }
        echo "  -> Skipped (build failed or target not available)"
        return 1
    else
        echo "  -> Skipped (target $target not installed)"
        return 1
    fi
}

# macOS
build_target "x86_64-apple-darwin" "3W2Coords-darwin-x64" || true
build_target "aarch64-apple-darwin" "3W2Coords-darwin-arm64" || true

# Linux
build_target "x86_64-unknown-linux-gnu" "3W2Coords-linux-x64" || true
build_target "aarch64-unknown-linux-gnu" "3W2Coords-linux-arm64" || true

# Windows
build_target "x86_64-pc-windows-gnu" "3W2Coords-windows-x64.exe" || true
build_target "aarch64-pc-windows-gnu" "3W2Coords-windows-arm64.exe" || true

# Fallback: build for current platform
if [ ! -f "target/release/3W2Coords-darwin-x64" ] && [ ! -f "target/release/3W2Coords-darwin-arm64" ] && [ ! -f "target/release/3W2Coords-linux-x64" ] && [ ! -f "target/release/3W2Coords-windows-x64.exe" ]; then
    echo "Building for current platform..."
    cargo build --release
    if [ -f "target/release/3W2Coords" ]; then
        OS=$(uname -s | tr '[:upper:]' '[:lower:]')
        ARCH=$(uname -m)
        if [ "$ARCH" = "x86_64" ]; then
            ARCH="x64"
        elif [ "$ARCH" = "arm64" ]; then
            ARCH="arm64"
        fi
        cp target/release/3W2Coords "target/release/3W2Coords-${OS}-${ARCH}"
    elif [ -f "target/release/3W2Coords.exe" ]; then
        cp target/release/3W2Coords.exe "target/release/3W2Coords-windows-$(uname -m).exe"
    fi
fi

echo ""
echo "Builds complete! Output in target/release/:"
ls -la target/release/ 2>/dev/null || echo "No builds produced"

echo ""
echo "Done!"