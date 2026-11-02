const assert = require("node:assert/strict");
const fs = require("node:fs");
const Module = require("node:module");
const path = require("node:path");
const test = require("node:test");

const root = path.join(__dirname, "..");
const manifest = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));

test("manifest registers the Pyra language, grammar, and requested commands", () => {
  assert.ok(manifest.contributes.languages.some(
    (language) => language.id === "pyra" && language.extensions.includes(".pyra")
  ));
  assert.ok(manifest.contributes.grammars.some(
    (grammar) => grammar.language === "pyra" && grammar.path === "./syntaxes/pyra.tmLanguage.json"
  ));
  assert.deepEqual(
    manifest.contributes.commands.map((command) => command.command),
    ["pyra.compileFile", "pyra.runExecutable", "pyra.checkFile"]
  );
});

test("extension activation registers all three commands", () => {
  const registered = [];
  const disposable = { dispose() {} };
  const diagnostics = { ...disposable, set() {} };
  const output = { ...disposable, appendLine() {}, show() {} };
  const vscode = {
    languages: { createDiagnosticCollection: () => diagnostics },
    window: { createOutputChannel: () => output },
    commands: {
      registerCommand(command, handler) {
        registered.push([command, handler]);
        return disposable;
      }
    }
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
  extension.deactivate();
});
