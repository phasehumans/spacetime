#!/usr/bin/env sh
set -e

REPO="phasehumans/spacetime"
INSTALL_DIR="${SPACETIME_INSTALL_DIR:-$HOME/.spacetime/bin}"

os_type() {
  case "$(uname -s)" in
    Linux*)     echo "linux";;
    Darwin*)    echo "darwin";;
    MINGW*|MSYS*|CYGWIN*) echo "windows";;
    *)          echo "unknown";;
  esac
}

arch_type() {
  case "$(uname -m)" in
    x86_64|amd64) echo "x86_64";;
    aarch64|arm64) echo "aarch64";;
    *)             echo "unknown";;
  esac
}

OS=$(os_type)
ARCH=$(arch_type)

if [ "$OS" = "unknown" ] || [ "$ARCH" = "unknown" ]; then
  echo "Error: Unsupported operating system or architecture ($OS / $ARCH)"
  exit 1
fi

BINARY_NAME="spacetime"
if [ "$OS" = "windows" ]; then
  BINARY_NAME="spacetime.exe"
  TARGET="${ARCH}-pc-windows-msvc"
elif [ "$OS" = "darwin" ]; then
  TARGET="${ARCH}-apple-darwin"
else
  TARGET="${ARCH}-unknown-linux-gnu"
fi

echo "⚡ Installing Spacetime ($OS / $ARCH)..."

LATEST_RELEASE_URL="https://api.github.com/repos/${REPO}/releases/latest"
DOWNLOAD_URL=$(curl -s "$LATEST_RELEASE_URL" | grep "browser_download_url.*${TARGET}" | cut -d '"' -f 4 || true)

if [ -z "$DOWNLOAD_URL" ]; then
  DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/spacetime-${TARGET}.tar.gz"
fi

mkdir -p "$INSTALL_DIR"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading ${DOWNLOAD_URL}..."
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/spacetime.tar.gz" || {
  echo "Error downloading Spacetime binary."
  exit 1
}

tar -xzf "$TMP_DIR/spacetime.tar.gz" -C "$TMP_DIR"
mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "✅ Spacetime successfully installed to ${INSTALL_DIR}/${BINARY_NAME}"
echo ""
echo "To add Spacetime to your PATH, add this line to your shell config (.bashrc / .zshrc):"
echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
