# Pyra Language for VS Code

<p align="center">
  <img src="https://raw.githubusercontent.com/soy-charly/Pyra/HEAD/vscode-extension/icon.png" alt="Pyra logo" width="180">
</p>

The extension recognizes `.pyra` files, provides TextMate syntax highlighting, snippets for functions/branches/loops/collections, built-in and local-function completion, hover documentation, function signature help, `#` line comments, automatic delimiter pairs, brace indentation, and three commands:

- **Pyra: Compile File** — compiles the active file beside its source.
- **Pyra: Check File** — runs the compiler's syntax/semantic-only mode.
- **Pyra: Run Executable** — opens a picker to run a compiled executable in the integrated terminal.

Set `pyraLanguage.compilerPath` to the compiler executable if `pyra` is not available on `PATH`. Compiler source diagnostics are published to VS Code's Problems panel. Completion, hover, and signature help use the built-ins and top-level function definitions in the current file; variable completion is inferred from simple assignments, loop variables, and function parameters. These features do not require an LSP server.

Package and install from the repository root:

```sh
(cd vscode-extension && npx --yes @vscode/vsce package --no-dependencies --allow-missing-repository)
code --install-extension vscode-extension/pyra-language-0.1.0.vsix
```

The compiler currently does not implement interpolated strings, so the grammar does not highlight them as a language feature. Cross-file navigation, type inference, and real-time diagnostics from an LSP server are not implemented.
