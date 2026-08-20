#!/usr/bin/env bash
set -euo pipefail

REPO="phasehumans/spacetime"
BINARY_NAME="spacetime"
INSTALL_DIR="${SPACETIME_INSTALL_DIR:-$HOME/.local/bin}"

ORANGE="\033[38;2;251;146;60m"
GREEN="\033[38;2;110;231;183m"
RED="\033[38;2;252;165;165m"
WHITE="\033[38;2;228;228;231m"
GREY="\033[38;2;113;113;122m"
TRUNK="\033[38;2;63;63;70m"
RESET="\033[0m"

log_step() { echo -e "${ORANGE}✱${RESET}  ${WHITE}$1${RESET}"; }
log_tree() { echo -e "${TRUNK}│${RESET}  ${GREY}$1${RESET}"; }
log_error() { echo -e "${ORANGE}✱${RESET}  ${RED}$1${RESET}"; }

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            case "$arch" in
                x86_64|amd64) echo "x86_64-unknown-linux-gnu" ;;
                aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
                *) echo "unsupported" ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                arm64) echo "aarch64-apple-darwin" ;;
                x86_64) echo "x86_64-apple-darwin" ;;
                *) echo "unsupported" ;;
            esac
            ;;
        MINGW64_NT*|MINGW32_NT*|MSYS_NT*|CYGWIN*)
            echo "windows"
            ;;
        *)
            echo "unsupported"
            ;;
    esac
}

TARGET=$(detect_target)

if [ "$TARGET" = "unsupported" ]; then
    log_error "Unsupported platform: $(uname -s) $(uname -m)"
    exit 1
fi

if [ "$TARGET" = "windows" ]; then
    log_step "Detected Windows environment. Launching PowerShell installer..."
    TMP_PS1="$(mktemp "${TEMP:-/tmp}/spacetime-install.XXXXXX.ps1")"
    curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/install.ps1" -o "$TMP_PS1"
    powershell.exe -ExecutionPolicy Bypass -File "$TMP_PS1"
    rm -f "$TMP_PS1"
    exit 0
fi

log_step "installing spacetime for ${TARGET}..."

VERSION="${SPACETIME_VERSION:-latest}"
ARCHIVE="spacetime-${TARGET}.tar.gz"

if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ARCHIVE}"
else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION#v}/${ARCHIVE}"
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

log_tree "downloading pre-compiled release binary..."
if ! curl -fL --progress-bar "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARCHIVE}"; then
    echo -e "${TRUNK}│${RESET}"
    log_error "Failed to download from ${DOWNLOAD_URL}"
    log_tree "Check https://github.com/${REPO}/releases for available versions."
    exit 1
fi

printf "\033[1A\033[2K"

log_tree "extracting binary..."
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"

mkdir -p "$INSTALL_DIR"
mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo -e "${TRUNK}│${RESET}"
log_step "${GREEN}spacetime successfully installed${RESET} to ${WHITE}${INSTALL_DIR}/${BINARY_NAME}${RESET}"

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
    echo -e "${TRUNK}│${RESET}"
    log_tree "Note: ${INSTALL_DIR} is not in your current PATH."
    log_tree "Add it to your shell profile (~/.bashrc or ~/.zshrc):"
    echo -e "${TRUNK}│${RESET}  ${WHITE}export PATH=\"\$PATH:${INSTALL_DIR}\"${RESET}"
fi

echo -e "${TRUNK}│${RESET}"
log_step "run ${WHITE}spacetime${RESET} to launch the interactive benchmark wizard"
