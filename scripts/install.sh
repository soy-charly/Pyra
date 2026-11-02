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
    local vscode_cli extensions_dir settings_file icon_theme_dir tmp_dir tmp_settings

    vscode_cli=$(command -v code || command -v code-insiders || true)

    if [[ -z "$vscode_cli" ]]; then
        printf 'install.sh: VS Code CLI (code or code-insiders) is required.\n' >&2
        return 1
    fi

    if ! command -v npx >/dev/null 2>&1; then
        printf 'install.sh: npx is required to package the VS Code extension.\n' >&2
        return 1
    fi

    if ! command -v jq >/dev/null 2>&1; then
        printf 'install.sh: jq is required to update VS Code settings safely.\n' >&2
        printf 'Install it with: sudo pacman -S jq\n' >&2
        return 1
    fi

    if [[ "$vscode_cli" == *code-insiders* ]]; then
        extensions_dir=${VSCODE_EXTENSIONS_DIR:-"$HOME/.vscode-insiders/extensions"}
        settings_file="$HOME/.config/Code - Insiders/User/settings.json"
    else
        extensions_dir=${VSCODE_EXTENSIONS_DIR:-"$HOME/.vscode/extensions"}
        settings_file="$HOME/.config/Code/User/settings.json"
    fi

    local icon_source="$repo_root/vscode-extension/icons/pyra.svg"

    if [[ ! -f "$icon_source" ]]; then
        printf 'install.sh: Pyra SVG not found: %s\n' "$icon_source" >&2
        return 1
    fi

    if [[ ! -f "$repo_root/vscode-extension/package.json" ]]; then
        printf 'install.sh: VS Code extension package.json not found.\n' >&2
        return 1
    fi

    tmp_dir=$(mktemp -d)

    printf 'Packaging the Pyra VS Code extension...\n'

    (
        cd "$repo_root/vscode-extension"
        npx --yes @vscode/vsce package \
            --no-dependencies \
            --out "$tmp_dir/pyra-language.vsix"
    )

    printf 'Installing the Pyra VS Code extension...\n'

    "$vscode_cli" --install-extension "$tmp_dir/pyra-language.vsix" --force

    printf 'Installing Material Icon Theme...\n'

    "$vscode_cli" --install-extension PKief.material-icon-theme --force

    printf 'Locating Material Icon Theme...\n'

    icon_theme_dir=""

    if [[ -d "$extensions_dir" ]]; then
        icon_theme_dir=$(find "$extensions_dir" \
            -mindepth 1 \
            -maxdepth 1 \
            -type d \
            -name 'pkief.material-icon-theme-*' \
            -print -quit)
    fi

    if [[ -z "$icon_theme_dir" || ! -d "$icon_theme_dir" ]]; then
        rm -rf -- "$tmp_dir"
        printf 'install.sh: Could not locate Material Icon Theme in %s\n' "$extensions_dir" >&2
        return 1
    fi

    if [[ ! -d "$icon_theme_dir/icons" ]]; then
        rm -rf -- "$tmp_dir"
        printf 'install.sh: Material Icon Theme icons directory not found.\n' >&2
        return 1
    fi

    printf 'Installing the Pyra SVG...\n'

    install -m 644 "$icon_source" "$icon_theme_dir/icons/pyra.svg"

    printf 'Configuring the Material Icon Theme association...\n'

    mkdir -p "$(dirname -- "$settings_file")"

    if [[ ! -f "$settings_file" || ! -s "$settings_file" ]]; then
        printf '{}\n' > "$settings_file"
    fi

    tmp_settings=$(mktemp)

    if jq '
        .["material-icon-theme.files.associations"] =
        ((.["material-icon-theme.files.associations"] // {}) + {"*.pyra": "pyra"})
    ' "$settings_file" > "$tmp_settings"; then
        if ! jq empty "$tmp_settings" >/dev/null 2>&1; then
            rm -f -- "$tmp_settings"
            rm -rf -- "$tmp_dir"
            printf 'install.sh: Generated settings.json is invalid.\n' >&2
            return 1
        fi

        install -m 644 "$tmp_settings" "$settings_file"
        rm -f -- "$tmp_settings"
    else
        rm -f -- "$tmp_settings"
        rm -rf -- "$tmp_dir"
        printf 'install.sh: Could not update VS Code settings.\n' >&2
        return 1
    fi

    rm -rf -- "$tmp_dir"

    printf '\nPyra VS Code extension and custom icon installed successfully.\n'
    printf 'Settings updated: %s\n' "$settings_file"
    printf 'Restart VS Code or run Developer: Reload Window.\n'
}

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
