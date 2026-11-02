# Pyra Language for VS Code

The extension recognizes `.pyra` files, provides TextMate syntax highlighting, `#` line comments, automatic delimiter pairs, brace indentation, and three commands:

- **Pyra: Compile File** — compiles the active file beside its source.
- **Pyra: Check File** — runs the compiler's syntax/semantic-only mode.
- **Pyra: Run Executable** — opens a picker to run a compiled executable in the integrated terminal.

Set `pyraLanguage.compilerPath` to the compiler executable if `pyra` is not available on `PATH`. Compiler source diagnostics are published to VS Code's Problems panel.

Package and install from the repository root:

```sh
(cd vscode-extension && npx --yes @vscode/vsce package --no-dependencies --allow-missing-repository)
code --install-extension vscode-extension/pyra-language-0.1.0.vsix
```

The compiler currently does not implement interpolated strings, so the grammar does not highlight them as a language feature. There is no language server or real-time diagnostics yet.
