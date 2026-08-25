#!/bin/sh
# crux installer. installs the latest release binary into ~/.local/bin
# usage: curl -fsSL https://crux.rweb.site/install.sh | bash
# override: CRUX_VERSION=v0.1.0 sh install.sh
set -eu

REPO="Emran-goat/crux"
BINDIR="${CRUX_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf '%s\n' "$*"; }
fail() { printf 'error: %s\n' "$*" >&2; exit 1; }

fetch() {
    url="$1"
    out="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$out"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$out" "$url"
    else
        fail "need curl or wget to download files"
    fi
}

case "$(uname -s)" in
    Linux) OS="linux" ;;
    Darwin) OS="darwin" ;;
    *)
        say "this script covers macOS and linux."
        say "on windows use install.ps1 from the repository instead."
        exit 1
        ;;
esac

case "$(uname -m)" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) fail "unsupported architecture: $(uname -m)" ;;
esac

if [ "$OS" = "darwin" ]; then
    TARGET="x86_64-apple-darwin"
else
    TARGET="$ARCH-linux-musl"
fi

say "resolving latest release..."
TAG=$(fetch "https://api.github.com/repos/$REPO/releases/latest" - \
    | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p')
[ -n "$TAG" ] || fail "could not determine the latest release. is there one published?"

ARTIFACT="crux-$TARGET.tar.gz"
URL="https://github.com/$REPO/releases/download/$TAG/$ARTIFACT"
say "downloading crux $TAG ($TARGET)"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
fetch "$URL" "$TMP/$ARTIFACT"

CHECKSUM_URL="$URL.sha256"
if fetch "$CHECKSUM_URL" "$TMP/checksum" 2>/dev/null; then
    EXPECTED=$(cut -d' ' -f1 "$TMP/checksum")
    ACTUAL=$( (sha256sum "$TMP/$ARTIFACT" || shasum -a 256 "$TMP/$ARTIFACT") | cut -d' ' -f1 )
    [ "$EXPECTED" = "$ACTUAL" ] || fail "checksum mismatch for $ARTIFACT"
    say "checksum ok"
fi

mkdir -p "$TMP/extract"
tar -xzf "$TMP/$ARTIFACT" -C "$TMP/extract"

mkdir -p "$BINDIR"
mv "$TMP/extract/crux" "$BINDIR/crux"
chmod +x "$BINDIR/crux"

"$BINDIR/crux" --help >/dev/null 2>&1 || say "warning: binary did not respond to --help"

say ""
say "installed: $BINDIR/crux ($TAG)"
case ":$PATH:" in
    *":$BINDIR:"*) ;;
    *) say "note: $BINDIR is not in your PATH. add this to your shell profile:"
       say "       export PATH=\"\$HOME/.local/bin:\$PATH\"" ;;
esac
say "next: crux init"
