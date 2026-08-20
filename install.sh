#!/usr/bin/env bash
set -euo pipefail

REPO="phasehumans/spacetime"
INSTALL_DIR="${SPACETIME_INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="spacetime"

ORANGE="\033[38;2;251;146;60m"
GREEN="\033[38;2;110;231;183m"
RED="\033[38;2;252;165;165m"
WHITE="\033[38;2;228;228;231m"
GREY="\033[38;2;113;113;122m"
TRUNK="\033[38;2;63;63;70m"
RESET="\033[0m"

log_step() {
    echo -e "${ORANGE}✱${RESET}  ${WHITE}$1${RESET}"
}

log_tree() {
    echo -e "${TRUNK}│${RESET}  ${GREY}$1${RESET}"
}

log_error() {
    echo -e "${ORANGE}✱${RESET}  ${RED}$1${RESET}"
}

# 1. Verify OS (Linux / WSL)
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    log_error "This installer supports Linux and Windows Subsystem for Linux (WSL). Detected OS: $OS"
    exit 1
fi

# 2. Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    aarch64|arm64)
        TARGET="aarch64-unknown-linux-gnu"
        ;;
    *)
        log_error "Unsupported architecture: $ARCH (Spacetime supports x86_64 and aarch64)"
        exit 1
        ;;
esac

log_step "installing spacetime for Linux/WSL (${TARGET})..."

# 3. Resolve version & download URL
VERSION="${SPACETIME_VERSION:-latest}"
ARCHIVE="spacetime-${TARGET}.tar.gz"

if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ARCHIVE}"
else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION#v}/${ARCHIVE}"
fi

TMP_DIR="$(mktemp -d)"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

log_tree "downloading pre-compiled release archive..."
if ! curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARCHIVE}"; then
    echo -e "${TRUNK}│${RESET}"
    log_error "Failed to download ${DOWNLOAD_URL}"
    log_tree "Make sure the release tag exists on https://github.com/${REPO}/releases"
    exit 1
fi

log_tree "extracting binary..."
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"

if [ ! -f "${TMP_DIR}/${BINARY_NAME}" ]; then
    log_error "Extracted archive did not contain '${BINARY_NAME}' executable."
    exit 1
fi

mkdir -p "$INSTALL_DIR"
mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo -e "${TRUNK}│${RESET}"
log_step "${GREEN}spacetime successfully installed${RESET} to ${WHITE}${INSTALL_DIR}/${BINARY_NAME}${RESET}"

# 4. Check PATH
if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
    echo -e "${TRUNK}│${RESET}"
    log_tree "Note: ${INSTALL_DIR} is not in your current PATH."
    log_tree "Add it to your shell configuration file (~/.bashrc or ~/.zshrc):"
    echo -e "${TRUNK}│${RESET}  ${WHITE}export PATH=\"\$PATH:${INSTALL_DIR}\"${RESET}"
fi

echo -e "${TRUNK}│${RESET}"
log_step "run ${WHITE}spacetime${RESET} to launch the interactive benchmark wizard"
