# Build and install

## Requirements

- Rust and Cargo (edition 2024).
- Clang or GCC with a C++20 standard library (`clang++` is the default backend).
- Node.js and VS Code to package/install the optional editor extension.

No Python installation is needed.

## Build, test, install

```sh
cargo test
cargo build --release
cargo install --path .
pyra examples/hello.pyra -o hello
./hello
```

To select another C++ compiler, set `PYRA_CXX` for the compiler invocation, for example `PYRA_CXX=g++ pyra hello.pyra`.

`cargo install --path .` installs `pyra` into Cargo's bin directory (usually `~/.cargo/bin`); ensure that directory is on `PATH`. During development, use `cargo run -- examples/hello.pyra` rather than assuming a global install.

## VS Code

From `vscode-extension/`, run `npx @vscode/vsce package --no-dependencies --allow-missing-repository` to create a `.vsix`, then install it with `code --install-extension pyra-language-0.1.0.vsix`. Configure `pyraLanguage.compilerPath` if `pyra` is not on `PATH`.

The current backend and packaging instructions target the host OS. A host C++20 compiler must be installed; cross-compilation, Windows executable suffix handling, and macOS notarization are not yet automated.
