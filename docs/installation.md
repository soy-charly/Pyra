# Build and install

## Requirements

- Rust and Cargo (edition 2024).
- Clang or GCC with a C++20 standard library (`clang++` is the default backend).
- Node.js/npm and the VS Code CLI (`code` or `code-insiders`) to install the optional editor extension and icon.

No Python installation is needed.

## Build, test, install

```sh
cargo test
cargo build --release
./scripts/install.sh  
pyra examples/hello.pyra -o out/hello
./out/hello
```

To select another C++ compiler, set `PYRA_CXX` for the compiler invocation, for example `PYRA_CXX=g++ pyra hello.pyra`.

`./scripts/install.sh` builds the release compiler and installs it to `/usr/local/bin/pyra`, using `sudo` only for the installation step when needed. Run it from any directory; it finds the repository relative to the script. During development, use `cargo run -- examples/hello.pyra` rather than assuming a global install.

The VS Code extension and custom Material Icon Theme SVG are optional. The installer asks about them in an interactive terminal; the default is **No**. Use `--vscode` to opt in without a prompt, or `--no-vscode` to skip explicitly:

```sh
./scripts/install.sh --vscode
```

Opting in packages and installs the Pyra extension, installs Material Icon Theme, and copies the existing [`pyra.svg`](../vscode-extension/icons/pyra.svg) from the extension's `icons` directory into the VS Code extensions icon directory. The script prints a `settings.json` association to enable the SVG; add it to your User Settings (JSON), merging it into any existing `material-icon-theme.files.associations` object:

```json
"material-icon-theme.files.associations": {
  "*.pyra": "../../icons/pyra"
}
```

The `.svg` suffix is intentionally omitted from the association. The installer does not edit VS Code settings, to avoid overwriting user configuration.

## VS Code

From the repository root, package and install the extension with:

```sh
(cd vscode-extension && npx --yes @vscode/vsce package --no-dependencies --allow-missing-repository)
code --install-extension vscode-extension/pyra-language-0.1.0.vsix
```

The package manifest is inside `vscode-extension/`, and the `.vsix` is created there; running `vsce package` from the repository root fails because that directory has no extension manifest. Configure `pyraLanguage.compilerPath` if `pyra` is not on `PATH`.

The current backend and packaging instructions target the host OS. A host C++20 compiler must be installed; cross-compilation, Windows executable suffix handling, and macOS notarization are not yet automated.
