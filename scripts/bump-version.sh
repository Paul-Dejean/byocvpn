#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
workspace_manifest="$repository_root/Cargo.toml"
tauri_configuration="$repository_root/crates/ui/src-tauri/tauri.conf.json"
frontend_manifest="$repository_root/crates/ui/package.json"

main() {
  local requested_version="${1:-}"
  local current_version next_version

  current_version="$(read_current_version)"
  if [[ -z "$current_version" ]]; then
    fail "could not read the current version from $workspace_manifest"
  fi

  next_version="${requested_version:-$(increment_patch_number "$current_version")}"
  if [[ ! "$next_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    fail "version must look like X.Y.Z, got '$next_version'"
  fi
  require_newer_version "$current_version" "$next_version"

  replace_version_line "$workspace_manifest" 'version = "' '"' "$current_version" "$next_version"
  replace_version_line "$tauri_configuration" '  "version": "' '",' "$current_version" "$next_version"
  replace_version_line "$frontend_manifest" '  "version": "' '",' "$current_version" "$next_version"

  cargo update --workspace --manifest-path "$workspace_manifest" >&2

  echo "$next_version"
}

read_current_version() {
  perl -ne 'print $1 if /^version = "(\d+\.\d+\.\d+)"$/' "$workspace_manifest"
}

increment_patch_number() {
  local major minor patch
  IFS=. read -r major minor patch <<< "$1"
  echo "$major.$minor.$((patch + 1))"
}

require_newer_version() {
  local current_version="$1" next_version="$2" highest_version
  highest_version="$(printf '%s\n%s\n' "$current_version" "$next_version" | sort -V | tail -n 1)"
  if [[ "$next_version" == "$current_version" || "$highest_version" != "$next_version" ]]; then
    fail "version $next_version is not newer than the current version $current_version"
  fi
}

replace_version_line() {
  local version_file="$1" line_prefix="$2" line_suffix="$3" current_version="$4" next_version="$5"
  LINE_PREFIX="$line_prefix" LINE_SUFFIX="$line_suffix" \
    CURRENT_VERSION="$current_version" NEXT_VERSION="$next_version" \
    perl -pi -e 's/^\Q$ENV{LINE_PREFIX}$ENV{CURRENT_VERSION}$ENV{LINE_SUFFIX}\E$/$ENV{LINE_PREFIX}$ENV{NEXT_VERSION}$ENV{LINE_SUFFIX}/' \
    "$version_file"
  if ! grep -qxF "$line_prefix$next_version$line_suffix" "$version_file"; then
    fail "could not set the version in $version_file"
  fi
}

fail() {
  echo "error: $1" >&2
  exit 1
}

main "$@"
