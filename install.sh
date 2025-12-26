#!/usr/bin/env bash
set -euo pipefail

REPO="DennySORA/Auto-Video-Organize"
REF="${REF:-main}"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"
BINARY_NAME="auto_video_organize"

if ! command -v git >/dev/null 2>&1; then
  echo "git is required but was not found." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required but was not found." >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

git clone --depth 1 --branch "$REF" "https://github.com/$REPO.git" "$tmp_dir/repo"

cd "$tmp_dir/repo"

cargo build --release --locked

install -d "$BIN_DIR"
install -m 0755 "target/release/$BINARY_NAME" "$BIN_DIR/$BINARY_NAME"

echo "Installed $BINARY_NAME to $BIN_DIR/$BINARY_NAME"
