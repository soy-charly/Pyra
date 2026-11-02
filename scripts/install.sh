#!/usr/bin/env bash
set -euo pipefail

repo_root=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
binary="$repo_root/target/release/pyra"
destination=/usr/local/bin/pyra
install_vscode=false
install_vscode_choice_set=false

usage() {
    printf 'Usage: %s [--vscode | --no-vscode]\n' "$0"
    printf '  --vscode     Also install the VS Code extension and Material Icon Theme icon.\n'
    printf '  --no-vscode  Skip the optional VS Code installation.\n'
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --vscode)
            install_vscode=true
            install_vscode_choice_set=true
            ;;
        --no-vscode)
            install_vscode=false
            install_vscode_choice_set=true
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'install.sh: unknown option: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

install_vscode_extension() {
    local vscode_cli extensions_dir tmp_dir
    vscode_cli=$(command -v code || command -v code-insiders || true)
    if [[ -z "$vscode_cli" ]]; then
        printf 'install.sh: VS Code CLI (code or code-insiders) is required for the optional editor install.\n' >&2
        return 1
    fi
    if ! command -v npx >/dev/null 2>&1; then
        printf 'install.sh: npx is required to package the optional VS Code extension.\n' >&2
        return 1
    fi

    if [[ "$vscode_cli" == *code-insiders* ]]; then
        extensions_dir=${VSCODE_EXTENSIONS_DIR:-"$HOME/.vscode-insiders/extensions"}
    else
        extensions_dir=${VSCODE_EXTENSIONS_DIR:-"$HOME/.vscode/extensions"}
    fi

    tmp_dir=$(mktemp -d)
    trap 'rm -rf -- "$tmp_dir"' EXIT
    printf 'Packaging the Pyra VS Code extension...\n'
    (
        cd "$repo_root/vscode-extension"
        npx --yes @vscode/vsce package --no-dependencies --out "$tmp_dir/pyra-language.vsix"
    )
    printf 'Installing the Pyra VS Code extension...\n'
    "$vscode_cli" --extensions-dir "$extensions_dir" --install-extension "$tmp_dir/pyra-language.vsix" --force

    printf 'Installing Material Icon Theme for the custom Pyra file icon...\n'
    "$vscode_cli" --extensions-dir "$extensions_dir" --install-extension PKief.material-icon-theme --force
    install -d "$extensions_dir/icons"
    install -m 644 "$repo_root/vscode-extension/icons/pyra.svg" "$extensions_dir/icons/pyra.svg"

    printf '\nInstalled the extension and Pyra SVG. To enable the icon, add this to VS Code User Settings (JSON):\n'
    printf '  "material-icon-theme.files.associations": {\n'
    printf '    "*.pyra": "../../icons/pyra"\n'
    printf '  }\n'
    printf 'If that setting already exists, add "*.pyra": "../../icons/pyra" to its existing object.\n'
    rm -rf -- "$tmp_dir"
    trap - EXIT
}

if ! command -v cargo >/dev/null 2>&1; then
    printf 'install.sh: cargo is required to build Pyra\n' >&2
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
        printf 'install.sh: sudo is required to install into /usr/local/bin\n' >&2
        printf 'Run this script as root or install sudo, then retry.\n' >&2
        exit 1
    fi
    sudo install -D -m 755 "$binary" "$destination"
fi

printf 'Installed Pyra. Verify with: pyra --version\n'

if [[ "$install_vscode_choice_set" == false && -t 0 ]]; then
    answer=
    read -r -p 'Also install the optional VS Code extension and file icon? [y/N] ' answer || true
    case "$answer" in
        y|Y|yes|YES|Yes)
            install_vscode=true
            ;;
    esac
fi

if [[ "$install_vscode" == true ]]; then
    install_vscode_extension
fi
