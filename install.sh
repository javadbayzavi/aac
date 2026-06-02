#!/usr/bin/env bash
set -e

REPO="https://github.com/javadbayzavi/aac"
BINARY="3t-scaffold-mcp"
INSTALL_DIR="$HOME/.3t-scaffold/bin"

echo "Installing $BINARY..."

# Detect OS and arch
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
  darwin) TARGET="${ARCH}-apple-darwin" ;;
  linux)  TARGET="${ARCH}-unknown-linux-gnu" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Download latest release
DOWNLOAD_URL="$REPO/releases/latest/download/$BINARY-$TARGET"
echo "Downloading from $DOWNLOAD_URL..."
curl -fsSL "$DOWNLOAD_URL" -o "/tmp/$BINARY"
chmod +x "/tmp/$BINARY"

# Install
mkdir -p "$INSTALL_DIR"
mv "/tmp/$BINARY" "$INSTALL_DIR/$BINARY"

echo "$BINARY installed to $INSTALL_DIR/$BINARY"

# Register with Claude Code
echo "Registering MCP server with Claude Code..."
claude mcp add --scope user 3t-scaffold "$BINARY" 2>/dev/null && \
  echo "MCP server registered. Restart Claude Code to use it." || \
  echo "Could not register automatically. Run: claude mcp add --scope user 3t-scaffold $BINARY"
