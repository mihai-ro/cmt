#!/usr/bin/env sh
# Install the latest cmt release binary into ~/.local/bin.
set -eu
REPO="mihai-ro/cmt"
BIN_DIR="${CMT_BIN_DIR:-$HOME/.local/bin}"

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
  Linux)  plat="unknown-linux-gnu" ;;
  Darwin) plat="apple-darwin" ;;
  *) echo "Unsupported OS: $os" >&2; exit 1 ;;
esac
case "$arch" in
  x86_64|amd64) cpu="x86_64" ;;
  arm64|aarch64) cpu="aarch64" ;;
  *) echo "Unsupported arch: $arch" >&2; exit 1 ;;
esac

target="${cpu}-${plat}"
url="https://github.com/${REPO}/releases/latest/download/cmt-${target}"
mkdir -p "$BIN_DIR"
echo "Downloading cmt ($target)..."
curl -fsSL "$url" -o "$BIN_DIR/cmt"
chmod +x "$BIN_DIR/cmt"
echo "Installed to $BIN_DIR/cmt"
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "Add $BIN_DIR to your PATH." ;;
esac
