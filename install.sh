#!/usr/bin/env bash
# install cmt — Conventional Commits CLI
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC="${SCRIPT_DIR}/cmt"

[[ -f "$SRC" ]] || { echo "✖  cmt not found"; exit 1; }

# pick install location: prefer /usr/local/bin if writable, else ~/.local/bin
if [[ -z "${INSTALL_DIR:-}" ]]; then
  if [[ -w "/usr/local/bin" ]]; then
    INSTALL_DIR="/usr/local/bin"
  else
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "$INSTALL_DIR"
  fi
fi

install -m 755 "$SRC" "${INSTALL_DIR}/cmt"
printf "✔ Installed cmt → ${INSTALL_DIR}/cmt\n"

if ! echo ":${PATH}:" | grep -q ":${INSTALL_DIR}:"; then
  printf "\n⚠  ${INSTALL_DIR} is not in your PATH. Add it for your shell:\n\n"
  printf "   bash/zsh  →  echo 'export PATH=\"%s:\$PATH\"' >> ~/.bashrc\n" "${INSTALL_DIR}"
  printf "   fish      →  fish_add_path %s\n\n" "${INSTALL_DIR}"
fi

printf "\n  cmt init       # set up a repo\n  cmt help       # all commands\n\n"
