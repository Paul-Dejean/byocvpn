#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
daemon_binary="$repository_root/target/debug/byocvpn_daemon"

cd "$repository_root"
cargo build -p byocvpn_daemon

echo "The daemon needs administrator rights to manage routes, DNS and the firewall."
sudo -v

stop_daemon() {
  sudo -n pkill -f "$daemon_binary" 2>/dev/null || true
}
trap stop_daemon EXIT

stop_daemon
sudo RUST_LOG="${RUST_LOG:-info}" RUST_LOG_STYLE=always "$daemon_binary" 2>&1 \
  | awk '{ print "[daemon] " $0; fflush() }' &

pnpm --dir crates/ui run tauri:dev
