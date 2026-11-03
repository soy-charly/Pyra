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

## Windows

Install the stable Rust toolchain with `rustup`, and install LLVM Clang plus Visual Studio Build Tools with the **Desktop development with C++** workload and a Windows SDK. Ensure `clang++` is available in PowerShell's `PATH`; Pyra uses it as the default C++20 backend. Alternatively, set `PYRA_CXX` to the command or path for another compatible C++20 compiler.

From the repository directory, install Pyra into Cargo's user binary directory:

```powershell
cargo install --path . --locked
pyra --version
```

To update an existing installation, run the same `cargo install` command again. If PowerShell cannot find `pyra`, add Cargo's binary directory (`$env:USERPROFILE\.cargo\bin`) to your user `PATH`, then open a new terminal.

Compile and run a program from PowerShell, specifying the `.exe` output path:

```powershell
pyra examples/hello.pyra -o .\out\hello.exe
.\out\hello.exe
```

The compiler emits a native executable for the host OS. Cross-compilation is not supported.
