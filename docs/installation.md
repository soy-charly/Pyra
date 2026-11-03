# Build and install

## Requirements

- Rust and Cargo (edition 2024).
- Clang or GCC with a C++20 standard library (`clang++` is the default backend).
- Boost headers, including Boost.Multiprecision.

No Python installation is needed.
Ensure the Boost headers are available on the C++ compiler's include path; Zenvik uses the header-only Boost.Multiprecision library for integer values.

## Build, test, install

```sh
cargo test
cargo build --release
./scripts/install.sh
zenvik examples/hello.zenvik -o out/hello
./out/hello
```

To select another C++ compiler, set `ZENVIK_CXX` for the compiler invocation, for example `ZENVIK_CXX=g++ zenvik hello.zenvik`.

`./scripts/install.sh` builds the release compiler and installs it to `/usr/local/bin/zenvik`, using `sudo` only for the installation step when needed. Run it from any directory; it finds the repository relative to the script. During development, use `cargo run -- examples/hello.zenvik` rather than assuming a global install.

## Windows

Install the stable Rust toolchain with `rustup`, and install LLVM Clang plus Visual Studio Build Tools with the **Desktop development with C++** workload and a Windows SDK. Ensure `clang++` is available in PowerShell's `PATH`; Zenvik uses it as the default C++20 backend. Alternatively, set `ZENVIK_CXX` to the command or path for another compatible C++20 compiler.

From the repository directory, install Zenvik into Cargo's user binary directory:

```powershell
cargo install --path . --locked
zenvik --version
```

To update an existing installation, run the same `cargo install` command again. If PowerShell cannot find `zenvik`, add Cargo's binary directory (`$env:USERPROFILE\.cargo\bin`) to your user `PATH`, then open a new terminal.

Compile and run a program from PowerShell, specifying the `.exe` output path:

```powershell
zenvik examples/hello.zenvik -o .\out\hello.exe
.\out\hello.exe
```

The compiler emits a native executable for the host OS. Cross-compilation is not supported.
