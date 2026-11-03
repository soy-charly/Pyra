# Build and install

## Requirements

- Rust and Cargo (edition 2024).
- Clang or GCC with a C++20 standard library (`clang++` is the default backend).

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

The current backend and packaging instructions target the host OS. A host C++20 compiler must be installed; cross-compilation, Windows executable suffix handling, and macOS notarization are not yet automated.
