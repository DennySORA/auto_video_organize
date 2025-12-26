#!/usr/bin/env bash
set -e

REPO_OWNER="DennySORA"
REPO_NAME="Auto-Video-Organize"
BIN_NAME="auto_video_organize"

# Default Install Directory
INSTALL_DIR="${HOME}/.local/bin"

# Help message
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    echo "Usage: install.sh [options]"
    echo ""
    echo "Options:"
    echo "  --version <tag>    Install a specific version (default: latest)"
    echo "  --to <dir>         Install to a specific directory (default: $INSTALL_DIR)"
    echo "  --force            Force installation even if verification fails"
    echo "  -h, --help         Show this help message"
    exit 0
fi

# Parse arguments
VERSION="latest"
FORCE=false
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --version) VERSION="$2"; shift ;;
        --to) INSTALL_DIR="$2"; shift ;;
        --force) FORCE=true ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

# Detect OS and Arch
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        OS_TYPE="unknown-linux-gnu"
        ;;
    Darwin)
        OS_TYPE="apple-darwin"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        ARCH_TYPE="x86_64"
        ;;
    aarch64|arm64)
        ARCH_TYPE="aarch64"
        ;;
    *)
        echo "Unsupported Architecture: $ARCH"
        exit 1
        ;;
esac

TARGET="${ARCH_TYPE}-${OS_TYPE}"

# Resolve Version
if [[ "$VERSION" == "latest" ]]; then
    echo "Fetching latest version..."
    RELEASE_URL="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"
    TAG_NAME=$(curl -sL "$RELEASE_URL" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [[ -z "$TAG_NAME" ]]; then
        echo "Error: Unable to find latest release. Rate limit might be exceeded."
        exit 1
    fi
else
    TAG_NAME="$VERSION"
fi

echo "Installing ${REPO_NAME} ${TAG_NAME} for ${TARGET}..."

# Construct Asset URL
ASSET_NAME="${BIN_NAME}-${TAG_NAME}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${TAG_NAME}/${ASSET_NAME}"

# Setup Temp Directory
TMP_DIR=$(mktemp -d)
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Download
echo "Downloading $DOWNLOAD_URL..."
if ! curl -sL -o "$TMP_DIR/asset.tar.gz" "$DOWNLOAD_URL"; then
    echo "Error: Download failed."
    exit 1
fi

# Extract
echo "Extracting..."
tar -xzf "$TMP_DIR/asset.tar.gz" -C "$TMP_DIR"

# Install
echo "Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
mv "$TMP_DIR/${BIN_NAME}" "$INSTALL_DIR/${BIN_NAME}"
chmod +x "$INSTALL_DIR/${BIN_NAME}"

echo "Successfully installed ${BIN_NAME} to ${INSTALL_DIR}/${BIN_NAME}"

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Warning: $INSTALL_DIR is not in your PATH."
    echo "Add the following to your shell config (.bashrc, .zshrc, etc.):"
    echo "  export PATH=\"$INSTALL_DIR:$PATH\""
fi