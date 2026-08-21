#!/bin/bash
set -e
REPO="frostre1997/prep"
BIN_NAME="prep"
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
      armv7l|armhf) TARGET="armv7-unknown-linux-gnueabihf" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  darwin)
    case "$ARCH" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      aarch64|arm64) TARGET="aarch64-apple-darwin" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

LATEST_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url.*$TARGET" | cut -d '"' -f 4)
if [ -z "$LATEST_URL" ]; then
  echo "Could not find binary for target $TARGET"
  exit 1
fi

curl -sSL "$LATEST_URL" -o /tmp/$BIN_NAME
chmod +x /tmp/$BIN_NAME
mv /tmp/$BIN_NAME "$INSTALL_DIR/"
echo "$BIN_NAME installed to $INSTALL_DIR"
echo "Add to PATH: export PATH=\"$INSTALL_DIR:\$PATH\""
