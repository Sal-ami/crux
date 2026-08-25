#!/bin/sh
set -e

TARGETS="x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-apple-darwin x86_64-pc-windows-msvc"

mkdir -p dist

for target in $TARGETS; do
    echo "building $target..."
    cargo build --release --target "$target" 2>/dev/null || continue
    bin="target/$target/release/crux"
    [ -f "$bin.exe" ] && bin="$bin.exe"
    [ -f "$bin" ] || continue
    cp "$bin" "dist/crux-$target"
    sha256sum "$bin" > "dist/crux-$target.sha256"
done

echo "dist/:"
ls -lh dist/
