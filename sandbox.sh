#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="spacetime-sandbox:latest"
DOCKERFILE="Dockerfile"

ORANGE="\033[38;2;251;146;60m"
GREEN="\033[38;2;110;231;183m"
RED="\033[38;2;252;165;165m"
WHITE="\033[38;2;228;228;231m"
GREY="\033[38;2;113;113;122m"
TRUNK="\033[38;2;63;63;70m"
RESET="\033[0m"

log_info() {
    echo -e "${ORANGE}✱${RESET}  ${WHITE}$1${RESET}"
}

log_tree() {
    echo -e "${TRUNK}│${RESET}  ${GREY}$1${RESET}"
}

log_error() {
    echo -e "${ORANGE}✱${RESET}  ${RED}$1${RESET}"
}

cmd_start() {
    log_info "starting spacetime sandbox image build/verification..."
    if [[ ! -f "$DOCKERFILE" ]]; then
        log_error "Dockerfile not found in current directory ($(pwd))"
        exit 1
    fi

    log_tree "building ${IMAGE_NAME}..."
    docker build -t "$IMAGE_NAME" -f "$DOCKERFILE" .
    echo -e "${TRUNK}│${RESET}"
    log_info "sandbox image ${WHITE}${IMAGE_NAME}${RESET} ready"
}

cmd_stop() {
    log_info "stopping and removing all active spacetime sandbox containers..."
    
    CONTAINERS=$(docker ps -q --filter "name=spacetime-")
    if [[ -n "$CONTAINERS" ]]; then
        echo "$CONTAINERS" | xargs -r docker rm -f >/dev/null
        log_tree "removed containers:"
        echo "$CONTAINERS" | while read -r cid; do
            echo -e "${TRUNK}│${RESET}  ${RED}●${RESET} ${GREY}${cid}${RESET}"
        done
        echo -e "${TRUNK}│${RESET}"
        log_info "all spacetime containers stopped"
    else
        log_tree "no active spacetime containers found"
        echo -e "${TRUNK}│${RESET}"
        log_info "clean state verified"
    fi
}

cmd_clean() {
    log_info "purging all stopped and dangling spacetime containers..."
    ALL_CONTAINERS=$(docker ps -a -q --filter "name=spacetime-")
    if [[ -n "$ALL_CONTAINERS" ]]; then
        echo "$ALL_CONTAINERS" | xargs -r docker rm -f >/dev/null
        log_tree "purged $(echo "$ALL_CONTAINERS" | wc -l) container(s)"
    else
        log_tree "no spacetime containers to clean"
    fi
    echo -e "${TRUNK}│${RESET}"
    log_info "cleanup complete"
}

cmd_status() {
    log_info "spacetime container status:"
    echo -e "${TRUNK}│${RESET}"
    
    RUNNING=$(docker ps --filter "name=spacetime-" --format "table {{.ID}}\t{{.Image}}\t{{.Status}}\t{{.Names}}")
    if [[ $(echo "$RUNNING" | wc -l) -gt 1 ]]; then
        echo "$RUNNING" | while read -r line; do
            echo -e "${TRUNK}│${RESET}  ${WHITE}${line}${RESET}"
        done
    else
        log_tree "no active containers running"
    fi
    echo -e "${TRUNK}│${RESET}"
}

cmd_shell() {
    log_info "launching interactive debug shell inside sandbox..."
    CONTAINER_NAME="spacetime-debug-$(date +%s)"
    docker run --rm -it \
        --name "$CONTAINER_NAME" \
        -v "$(pwd):/workspace:ro" \
        -w /workspace \
        "$IMAGE_NAME" /bin/bash
}

ACTION="${1:-status}"

case "$ACTION" in
    start)
        cmd_start
        ;;
    stop)
        cmd_stop
        ;;
    clean)
        cmd_clean
        ;;
    status)
        cmd_status
        ;;
    shell)
        cmd_shell
        ;;
    -h|--help|help)
        echo -e "${ORANGE}✱${RESET}  ${WHITE}SPACETIME Sandbox Manager${RESET}"
        echo -e "${TRUNK}│${RESET}"
        echo -e "${TRUNK}│${RESET}  ${WHITE}Usage:${RESET} ./sandbox.sh [command]"
        echo -e "${TRUNK}│${RESET}"
        echo -e "${TRUNK}│${RESET}  ${WHITE}Commands:${RESET}"
        echo -e "${TRUNK}│${RESET}    ${ORANGE}start${RESET}   - Build/verify the sandbox docker image"
        echo -e "${TRUNK}│${RESET}    ${ORANGE}stop${RESET}    - Force-stop & remove all running spacetime containers"
        echo -e "${TRUNK}│${RESET}    ${ORANGE}clean${RESET}   - Purge all dangling/stopped spacetime containers"
        echo -e "${TRUNK}│${RESET}    ${ORANGE}status${RESET}  - Show currently running spacetime containers"
        echo -e "${TRUNK}│${RESET}    ${ORANGE}shell${RESET}   - Open an interactive bash shell in sandbox"
        echo -e "${TRUNK}│${RESET}"
        ;;
    *)
        log_error "unknown command: '$ACTION'"
        echo -e "${TRUNK}│${RESET}  Run ${WHITE}./sandbox.sh help${RESET} for available commands"
        exit 1
        ;;
esac
