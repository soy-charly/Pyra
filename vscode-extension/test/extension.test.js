const assert = require("node:assert/strict");
const fs = require("node:fs");
const Module = require("node:module");
const path = require("node:path");
const test = require("node:test");

const root = path.join(__dirname, "..");
const manifest = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
const snippets = JSON.parse(fs.readFileSync(path.join(root, "snippets/pyra.json"), "utf8"));

test("manifest registers the Pyra language, grammar, and commands", () => {
  assert.ok(manifest.contributes.languages.some(
    (language) => language.id === "pyra" && language.extensions.includes(".pyra")
  ));
  assert.ok(!manifest.contributes.iconThemes, "the extension should not replace the user's icon theme");
  assert.ok(manifest.contributes.grammars.some(
    (grammar) => grammar.language === "pyra" && grammar.path === "./syntaxes/pyra.tmLanguage.json"
  ));
  assert.deepEqual(
    manifest.contributes.commands.map((command) => command.command),
    ["pyra.compileFile", "pyra.runExecutable", "pyra.checkFile"]
  );
  assert.ok(manifest.contributes.snippets.some(
    (snippet) => snippet.language === "pyra" && snippet.path === "./snippets/pyra.json"
  ));
});

test("snippets cover functions, branches, loops, and common built-ins", () => {
  const prefixes = new Set(Object.values(snippets).map((snippet) => snippet.prefix));
  for (const prefix of ["def", "if", "ifel", "for", "forr", "while", "print", "input", "list", "dict"]) {
    assert.ok(prefixes.has(prefix), `missing snippet prefix ${prefix}`);
  }
  for (const snippet of Object.values(snippets)) {
    assert.ok(snippet.body, `snippet ${snippet.prefix} has no body`);
    assert.ok(snippet.description, `snippet ${snippet.prefix} has no description`);
  }
  for (const name of ["Print", "Input", "List literal", "Dictionary literal"]) {
    assert.match(
      Array.isArray(snippets[name].body) ? snippets[name].body.join("\n") : snippets[name].body,
      /;\$0$/,
      `${name} snippet should include a statement terminator`
    );
  }
});

test("extension activation adds Pyra file badges without replacing the icon theme", () => {
  const registered = [];
  const providers = [];
  let fileDecorationProvider;
  const disposable = { dispose() {} };
  const diagnostics = { ...disposable, set() {} };
  const output = { ...disposable, appendLine() {}, show() {} };
  class FileDecoration {
    constructor(badge, tooltip) {
      this.badge = badge;
      this.tooltip = tooltip;
    }
  }
  const vscode = {
    languages: {
      createDiagnosticCollection: () => diagnostics,
      registerCompletionItemProvider(...args) {
        providers.push(["completion", ...args]);
        return disposable;
      },
      registerHoverProvider(...args) {
        providers.push(["hover", ...args]);
        return disposable;
      },
      registerSignatureHelpProvider(...args) {
        providers.push(["signature", ...args]);
        return disposable;
      }
    },
    window: {
      createOutputChannel: () => output,
      registerFileDecorationProvider(provider) {
        fileDecorationProvider = provider;
        return disposable;
      },
      showErrorMessage() {}
    },
    commands: {
      registerCommand(command, handler) {
        registered.push([command, handler]);
        return disposable;
      }
    },
    FileDecoration
  };
  const originalLoad = Module._load;
  Module._load = function (request, parent, isMain) {
    return request === "vscode" ? vscode : originalLoad.call(this, request, parent, isMain);
  };
  let extension;
  try {
    extension = require("../extension");
  } finally {
    Module._load = originalLoad;
  }

  extension.activate({ subscriptions: [] });
  assert.deepEqual(registered.map(([command]) => command), [
    "pyra.compileFile",
    "pyra.runExecutable",
    "pyra.checkFile"
  ]);
  const pyraDecoration = fileDecorationProvider.provideFileDecoration({
    fsPath: "/workspace/examples/conditions.pyra"
  });
  assert.equal(pyraDecoration.badge, "Pyra");
  assert.equal(pyraDecoration.tooltip, "Pyra source file");
  assert.equal(fileDecorationProvider.provideFileDecoration({
    fsPath: "/workspace/README.md"
  }), undefined);
  assert.deepEqual(providers.map(([kind, language]) => [kind, language]), [
    ["completion", "pyra"],
    ["hover", "pyra"],
    ["signature", "pyra"]
  ]);
  extension.deactivate();
});

test("language providers complete built-ins and current-file functions", () => {
  const { registerLanguageFeatures } = require("../language-features");
  const providers = {};
  const vscode = editorApi({
    registerCompletionItemProvider(language, provider) {
      providers.completion = provider;
      return { dispose() {} };
    },
    registerHoverProvider(language, provider) {
      providers.hover = provider;
      return { dispose() {} };
    },
    registerSignatureHelpProvider(language, provider) {
      providers.signature = provider;
      return { dispose() {} };
    }
  });
  registerLanguageFeatures(vscode, { subscriptions: [] });
  const document = textDocument("def greet(name) {\n    result = name\n    print(result)\n}\n");
  const items = providers.completion.provideCompletionItems(document);
  assert.ok(items.some((item) => item.label === "range" && item.detail === "Pyra built-in"));
  const custom = items.find((item) => item.label === "greet");
  assert.ok(custom);
  assert.equal(custom.insertText, "greet($0)");
  assert.match(custom.documentation, /greet\(name\)/);
  assert.ok(items.some((item) => item.label === "result" && item.kind === 3));
  assert.ok(items.some((item) => item.label === "True" && item.kind === 2));
});

test("hover and signature help describe built-ins and user-defined functions", () => {
  const { registerLanguageFeatures } = require("../language-features");
  const providers = {};
  const vscode = editorApi({
    registerCompletionItemProvider(language, provider) {
      providers.completion = provider;
      return { dispose() {} };
    },
    registerHoverProvider(language, provider) {
      providers.hover = provider;
      return { dispose() {} };
    },
    registerSignatureHelpProvider(language, provider) {
      providers.signature = provider;
      return { dispose() {} };
    }
  });
  registerLanguageFeatures(vscode, { subscriptions: [] });
  const source = "def add(left, right) {\n    return left + right\n}\nprint(add(1, ";
  const document = textDocument(source);
  document.getWordRangeAtPosition = () => ({ start: 0, end: 4 });
  document.getText = (range) => range ? "print" : source;
  const hover = providers.hover.provideHover(document, { line: 0, character: 2 });
  assert.match(hover.contents.value, /Print values/);

  const position = { line: 3, character: source.split("\n")[3].length };
  const help = providers.signature.provideSignatureHelp(document, position);
  assert.equal(help.signatures[help.activeSignature].label, "add(left, right)");
  assert.equal(help.activeParameter, 1);
});

function editorApi(languageRegistrars) {
  class CompletionItem {
    constructor(label, kind) {
      this.label = label;
      this.kind = kind;
    }
  }
  class MarkdownString {
    constructor(value) {
      this.value = value;
    }
  }
  class Hover {
    constructor(contents, range) {
      this.contents = contents;
      this.range = range;
    }
  }
  class SignatureHelp {}
  class SignatureInformation {
    constructor(label) {
      this.label = label;
    }
  }
  class ParameterInformation {
    constructor(label) {
      this.label = label;
    }
  }
  return {
    languages: languageRegistrars,
    CompletionItem,
    CompletionItemKind: { Function: 1, Keyword: 2, Variable: 3 },
    MarkdownString,
    Hover,
    SignatureHelp,
    SignatureInformation,
    ParameterInformation
  };
}

function textDocument(source) {
  return {
    getText(range) {
      return range ? "" : source;
    },
    offsetAt(position) {
      return source.split("\n").slice(0, position.line).join("\n").length
        + (position.line > 0 ? 1 : 0)
        + position.character;
    }
  };
}
