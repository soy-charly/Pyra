#!/usr/bin/env bash
set -euo pipefail

repo_root=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
binary="$repo_root/target/release/pyra"
destination=/usr/local/bin/pyra

usage() {
    printf 'Usage: %s\n' "$0"
    printf 'Build and install Pyra to %s.\n' "$destination"
}

if [[ $# -gt 0 ]]; then
    if [[ $# -eq 1 && ( "$1" == "-h" || "$1" == "--help" ) ]]; then
        usage
        exit 0
    fi

    printf 'install.sh: unexpected argument: %s\n' "$1" >&2
    usage >&2
    exit 2
fi

if ! command -v cargo >/dev/null 2>&1; then
    printf 'install.sh: cargo is required to build Pyra.\n' >&2
    exit 1
fi

printf 'Building Pyra in release mode...\n'
cargo build --release --manifest-path "$repo_root/Cargo.toml"

if [[ ! -x "$binary" ]]; then
    printf 'install.sh: expected executable was not produced: %s\n' "$binary" >&2
    exit 1
fi

printf 'Installing Pyra to %s...\n' "$destination"

if [[ $(id -u) -eq 0 ]]; then
    install -D -m 755 "$binary" "$destination"
else
    if ! command -v sudo >/dev/null 2>&1; then
        printf 'install.sh: sudo is required to install into /usr/local/bin.\n' >&2
        printf 'Run this script as root or install sudo, then retry.\n' >&2
        exit 1
    fi

    sudo install -D -m 755 "$binary" "$destination"
fi

printf 'Installed Pyra. Verify with: pyra --version\n'
