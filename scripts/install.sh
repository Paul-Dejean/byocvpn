#!/usr/bin/env bash
set -euo pipefail

HELPER_TOOLS_DIR="/Library/PrivilegedHelperTools"
LAUNCH_DAEMONS_DIR="/Library/LaunchDaemons"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ---------------------------------------------------------------------------
# Parse flags
# ---------------------------------------------------------------------------

DEV=false
COMMAND="install"

for arg in "$@"; do
    case "$arg" in
        --dev) DEV=true ;;
        install|uninstall|status) COMMAND="$arg" ;;
        *) echo "Unknown argument: $arg"; exit 1 ;;
    esac
done

if [[ "$DEV" == "true" ]]; then
    DAEMON_BINARY_NAME="byocvpn-daemon-dev"
    DAEMON_LABEL="com.byocvpn.daemon.dev"
    BUILD_DIR="debug"
else
    DAEMON_BINARY_NAME="byocvpn-daemon"
    DAEMON_LABEL="com.byocvpn.daemon"
    BUILD_DIR="release"
fi

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

is_installed() {
    [[ -f "$HELPER_TOOLS_DIR/$DAEMON_BINARY_NAME" ]] \
        && [[ -f "$LAUNCH_DAEMONS_DIR/$DAEMON_LABEL.plist" ]]
}

unload_if_running() {
    if launchctl list "$DAEMON_LABEL" &>/dev/null; then
        echo "→ Stopping $DAEMON_LABEL..."
        launchctl unload "$LAUNCH_DAEMONS_DIR/$DAEMON_LABEL.plist" 2>/dev/null || true
    fi
}

# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

cmd_install() {
    local daemon_binary_path=""
    local daemon_plist_path="$SCRIPT_DIR/$DAEMON_LABEL.plist"
    # Cargo always outputs byocvpn_daemon (underscores); we install it as $DAEMON_BINARY_NAME.
    local source_binary_name="byocvpn_daemon"

    if [[ -f "$SCRIPT_DIR/$source_binary_name" ]]; then
        daemon_binary_path="$SCRIPT_DIR/$source_binary_name"
    elif [[ -f "$SCRIPT_DIR/../target/$BUILD_DIR/$source_binary_name" ]]; then
        daemon_binary_path="$(cd "$SCRIPT_DIR/../target/$BUILD_DIR" && pwd)/$source_binary_name"
    else
        echo "❌  Could not find $source_binary_name in target/$BUILD_DIR/."
        if [[ "$DEV" == "true" ]]; then
            echo "    Build first:  cargo build -p byocvpn_daemon"
        else
            echo "    Build first:  cargo build --release -p byocvpn_daemon"
        fi
        exit 1
    fi

    if [[ ! -f "$daemon_plist_path" ]]; then
        echo "❌  Could not find $DAEMON_LABEL.plist in scripts/."
        exit 1
    fi

    echo "→ Installing $DAEMON_BINARY_NAME from $daemon_binary_path..."

    unload_if_running

    install -o root -g wheel -m 544 \
        "$daemon_binary_path" "$HELPER_TOOLS_DIR/$DAEMON_BINARY_NAME"

    install -o root -g wheel -m 644 \
        "$daemon_plist_path" "$LAUNCH_DAEMONS_DIR/$DAEMON_LABEL.plist"

    launchctl load "$LAUNCH_DAEMONS_DIR/$DAEMON_LABEL.plist"

    echo "✅  $DAEMON_LABEL installed and started."
}

cmd_uninstall() {
    echo "→ Uninstalling $DAEMON_LABEL..."
    unload_if_running
    rm -f "$HELPER_TOOLS_DIR/$DAEMON_BINARY_NAME"
    rm -f "$LAUNCH_DAEMONS_DIR/$DAEMON_LABEL.plist"
    echo "✅  $DAEMON_LABEL removed."
}

cmd_status() {
    if is_installed; then
        echo "✅  $DAEMON_LABEL is installed."
    else
        echo "❌  $DAEMON_LABEL is not installed."
    fi

    if launchctl list "$DAEMON_LABEL" &>/dev/null; then
        echo "✅  $DAEMON_LABEL is running."
    else
        echo "❌  $DAEMON_LABEL is not running."
    fi
}

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root. Try:  sudo $0 $*"
    exit 1
fi

case "$COMMAND" in
    install)   cmd_install ;;
    uninstall) cmd_uninstall ;;
    status)    cmd_status ;;
esac
