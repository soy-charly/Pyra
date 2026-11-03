# Zenvik

![Zenvik logo](icons/Zenvik-Logo.png)

Zenvik is a small, brace-delimited, Python-inspired compiled language. Its compiler is written in Rust and emits C++20; Clang or GCC then builds a standalone native executable. Compiled programs do not load or invoke the Zenvik compiler, Python, or another interpreter.

Zenvik is an early-stage project, and its language and runtime may change. It is not Python-compatible. The supported syntax, runtime model, build requirements, and current limitations are described in the [language specification](docs/language-spec.md), [architecture notes](docs/architecture.md), and [installation guide](docs/installation.md).

## Requirements

- Rust and Cargo (edition 2024).
- A C++20 compiler and standard library. The default is `clang++`; set `ZENVIK_CXX` to select another compiler, such as `g++`.
- Boost headers, including Boost.Multiprecision.

No Python installation is needed.

## Build and try it

```sh
git clone https://github.com/soy-charly/Pyra.git zenvik
cd zenvik
```

Linux/macOS:

```sh
cargo build --release
cargo test
./target/release/zenvik examples/factorial.zenvik -o out/factorial
./out/factorial
```

To install the release binary to `/usr/local/bin/zenvik`, run:

```sh
./scripts/install.sh
```

The installer may use `sudo` to write to `/usr/local/bin`.
For development, run the compiler without installing it:

```sh
cargo run -- examples/hello.zenvik -o out/hello
./out/hello
```

Windows:

On Windows, install the prerequisites and install Zenvik with `cargo install --path . --locked`. See the [installation guide](docs/installation.md#windows) for the C++ toolchain setup and PowerShell example.

See the [CLI reference](docs/cli-reference.md) for available options and [examples/](examples/) for more programs.

## Example

```zenvik
def factorial(n) {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

for i in range(6) {
    print(factorial(i));
}
```

## Project layout

- `src/`: Rust lexer, parser, semantic checks, C++ code generator, and command-line interface.
- `examples/`: runnable `.zenvik` programs.
- `tests/`: compiler-to-native-executable integration tests.
- `docs/`: language, backend, CLI, and installation documentation.

## Contributing

Contributions are welcome. For a substantial change, open an issue first to discuss the approach. For bug reports, include the Zenvik version, operating system, C++ compiler, a small reproducing program, and the actual and expected behavior. Please do not include secrets or private data in issues.

Before opening a pull request:

1. Create a focused branch and keep the change scoped to one fix or feature.
2. Run `cargo fmt --check` and `cargo test`.
3. If you change language syntax or runtime behavior, add or update tests and the relevant documentation, especially [docs/language-spec.md](docs/language-spec.md).
4. Update other affected docs and examples, and describe the user-visible change and validation in the pull request.

Keep generated binaries and build artifacts out of commits. Use clear commit messages; there is no required commit-message format.

## License

Zenvik is distributed under the [MIT License](LICENSE).
