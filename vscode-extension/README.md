# Pyra Language for VS Code

<p align="center">
  <img src="https://raw.githubusercontent.com/soy-charly/Pyra/HEAD/vscode-extension/icons/icon.png" alt="Pyra logo" width="180">
</p>

The extension recognizes `.pyra` files, provides TextMate syntax highlighting, snippets for functions/branches/loops/collections, built-in and local-function completion, hover documentation, function signature help, `#` line comments, automatic delimiter pairs, brace indentation, and three commands:

- **Pyra: Compile File** — compiles the active file beside its source.
- **Pyra: Check File** — runs the compiler's syntax/semantic-only mode.
- **Pyra: Run Executable** — opens a picker to run a compiled executable in the integrated terminal.

The Explorer shows a **Pyra** badge beside `.pyra` files while preserving the user's selected file icon theme. VS Code extensions cannot overlay a custom image on arbitrary file icon themes; badges are the supported additive decoration.

### Use a custom SVG with Material Icon Theme

Material Icon Theme supports custom SVG file icons. The Pyra SVG is [`icons/pyra.svg`](./icons/pyra.svg). Copy it to an `icons` folder inside VS Code's user extensions directory:

```text
<VS Code extensions directory>/
└── icons/
    └── pyra.svg
```

On Linux/macOS, the extensions directory is usually `~/.vscode/extensions`; on Windows, it is usually `%USERPROFILE%\.vscode\extensions`. If you use VS Code Insiders or a portable installation, use that installation's extensions directory.

Install **Material Icon Theme**, open **Preferences: Open User Settings (JSON)**, and add this association:

```json
"material-icon-theme.files.associations": {
  "*.pyra": "../../icons/pyra"
}
```

The association path is relative to Material Icon Theme's `dist` directory, and the `.svg` suffix is omitted. If `material-icon-theme.files.associations` already exists in your settings, add `"*.pyra": "../../icons/pyra"` to that object rather than replacing your existing associations. Select **Material Icon Theme** as your file icon theme if it is not already active.

Alternatively, to use a recolored Python icon instead of a custom SVG, add a clone under `material-icon-theme.languages.customClones`:

```json
"material-icon-theme.languages.customClones": [
  {
    "name": "pyra-clone",
    "base": "python",
    "color": "purple-400",
    "lightColor": "purple-700",
    "ids": ["pyra"]
  }
]
```

If that setting already exists, add the object to its array without removing existing clones.

Set `pyraLanguage.compilerPath` to the compiler executable if `pyra` is not available on `PATH`. Compiler source diagnostics are published to VS Code's Problems panel. Completion, hover, and signature help use the built-ins and top-level function definitions in the current file; variable completion is inferred from simple assignments, loop variables, and function parameters. These features do not require an LSP server.

Package and install from the repository root:

```sh
(cd vscode-extension && npx --yes @vscode/vsce package --no-dependencies --allow-missing-repository)
code --install-extension vscode-extension/pyra-language-0.1.0.vsix
```

The repository installer can optionally package and install the extension and the custom SVG asset along with the compiler:

```sh
./scripts/install.sh --vscode
```

It also installs Material Icon Theme and copies the existing `icons/pyra.svg` into the VS Code extensions icon directory. To activate the SVG, add the `material-icon-theme.files.associations` entry above to User Settings (JSON); the installer prints the snippet and leaves settings unchanged.

The compiler currently does not implement interpolated strings, so the grammar does not highlight them as a language feature. Cross-file navigation, type inference, and real-time diagnostics from an LSP server are not implemented.
