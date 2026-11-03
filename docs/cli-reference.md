# CLI reference

```text
zenvik [options] file.zenvik [file.zenvik ...]
```

- `-o PATH`: output path. By default, the first input's extension is removed.
- `-c`: compile to an object file (`.o` by default).
- `-S`: emit native assembly (`.s` by default).
- `-fsyntax-only`: lex, parse, check, and validate generated C++ without writing a final artifact.
- `-O0`, `-O1`, `-O2`, `-O3`: pass the selected optimization level to the C++ backend.
- `-v`: show concise backend invocation details.
- `--help`, `--version`: print help or version.

`-c`, `-S`, and `-fsyntax-only` are mutually exclusive. `-o` is not valid with `-fsyntax-only`. Multiple source files are parsed and compiled together. Paths are passed as separate process arguments, so spaces are supported. Set `ZENVIK_CXX` to select the C++20 compiler; the default is `clang++`.

The Zenvik front end uses exit code 2 for invalid input or source diagnostics, and propagates the native backend's exit status for backend errors.
