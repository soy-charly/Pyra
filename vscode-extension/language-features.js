const BUILTINS = {
  print: {
    signatures: [["*values"]],
    summary: "Print values separated by spaces, followed by a newline."
  },
  input: {
    signatures: [["prompt"], []],
    summary: "Read a line from standard input. An optional prompt is printed first."
  },
  len: {
    signatures: [["value"]],
    summary: "Return the length of a string, list, or dictionary."
  },
  range: {
    signatures: [["stop"], ["start", "stop"], ["start", "stop", "step"]],
    summary: "Create an integer sequence for use in a for loop. The stop value is excluded."
  },
  str: {
    signatures: [["value"]],
    summary: "Convert a value to a string."
  },
  int: {
    signatures: [["value"]],
    summary: "Convert a value to an integer."
  },
  float: {
    signatures: [["value"]],
    summary: "Convert a value to a floating-point number."
  },
  bool: {
    signatures: [["value"]],
    summary: "Convert a value to a boolean."
  },
  type: {
    signatures: [["value"]],
    summary: "Return the type name of a value."
  }
};

const KEYWORDS = {
  def: "Define a top-level function with fixed positional parameters.",
  if: "Run a block when its condition is true.",
  elif: "Test another condition after an if or elif branch.",
  else: "Run the alternative block when no preceding if/elif condition matched.",
  for: "Iterate over each value in an iterable.",
  in: "Test membership or introduce the iterable in a for loop.",
  while: "Repeat a block while its condition is true.",
  return: "Return a value from the current function.",
  break: "Exit the innermost loop.",
  continue: "Skip to the next iteration of the innermost loop.",
  and: "Short-circuit logical AND; returns one of its operands.",
  or: "Short-circuit logical OR; returns one of its operands.",
  not: "Negate the truth value of an expression.",
  True: "Boolean true value.",
  False: "Boolean false value.",
  None: "The absence of a value."
};

function userVariables(document) {
  const variables = new Set();
  const source = document.getText();
  const assignment = /^\s*([A-Za-z_][A-Za-z0-9_]*)\s*(?:=|\+=|-=|\*=|\/=|%=)/gm;
  const loopVariable = /^\s*for\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\b/gm;
  for (const match of source.matchAll(assignment)) variables.add(match[1]);
  for (const match of source.matchAll(loopVariable)) variables.add(match[1]);
  for (const definition of userFunctions(document).values()) {
    for (const parameter of definition.parameters) variables.add(parameter);
  }
  for (const name of [...variables]) {
    if (BUILTINS[name] || KEYWORDS[name]) variables.delete(name);
  }
  return variables;
}

function userFunctions(document) {
  const functions = new Map();
  const source = document.getText();
  const definition = /^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)/gm;
  for (const match of source.matchAll(definition)) {
    const parameters = match[2].trim()
      ? match[2].split(",").map((parameter) => parameter.trim()).filter(Boolean)
      : [];
    functions.set(match[1], { parameters, line: source.slice(0, match.index).split("\n").length });
  }
  return functions;
}

function signatureText(name, parameters) {
  return `${name}(${parameters.join(", ")})`;
}

function functionDocumentation(name, parameters, userFunction) {
  const signature = signatureText(name, parameters);
  return userFunction
    ? `\`${signature}\`\n\nUser-defined Pyra function (line ${userFunction.line}).`
    : `\`${signature}\`\n\n${BUILTINS[name].summary}`;
}

function wordAt(document, position) {
  const range = document.getWordRangeAtPosition(position, /[A-Za-z_][A-Za-z0-9_]*/);
  if (!range) return undefined;
  return { range, name: document.getText(range) };
}

function completionProvider(vscode) {
  return {
    provideCompletionItems(document) {
      const items = [];
      for (const [name, detail] of Object.entries(BUILTINS)) {
        const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Function);
        item.detail = "Pyra built-in";
        item.documentation = `${signatureText(name, detail.signatures[0])}\n\n${detail.summary}`;
        items.push(item);
      }
      for (const [name, detail] of Object.entries(KEYWORDS)) {
        const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Keyword);
        item.detail = "Pyra keyword";
        item.documentation = detail;
        items.push(item);
      }
      for (const [name, definition] of userFunctions(document)) {
        const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Function);
        item.detail = "User-defined function";
        item.documentation = functionDocumentation(name, definition.parameters, definition);
        item.insertText = `${name}($0)`;
        items.push(item);
      }
      for (const name of userVariables(document)) {
        const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Variable);
        item.detail = "Variable in this file";
        items.push(item);
      }
      return items;
    }
  };
}

function hoverProvider(vscode) {
  return {
    provideHover(document, position) {
      const word = wordAt(document, position);
      if (!word) return undefined;
      const { name, range } = word;
      let contents;
      if (BUILTINS[name]) {
        contents = functionDocumentation(name, BUILTINS[name].signatures[0]);
      } else if (KEYWORDS[name]) {
        contents = `**${name}**\n\n${KEYWORDS[name]}`;
      } else {
        const definition = userFunctions(document).get(name);
        if (!definition) return undefined;
        contents = functionDocumentation(name, definition.parameters, definition);
      }
      const markdown = new vscode.MarkdownString(contents);
      markdown.isTrusted = false;
      return new vscode.Hover(markdown, range);
    }
  };
}

function callContext(source, offset) {
  const stack = [];
  let quote;
  let escaped = false;
  let comment = false;
  for (let index = 0; index < offset; index += 1) {
    const character = source[index];
    if (comment) {
      if (character === "\n") comment = false;
      continue;
    }
    if (quote) {
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === quote) quote = undefined;
      continue;
    }
    if (character === "#") {
      comment = true;
      continue;
    }
    if (character === "'" || character === "\"") {
      quote = character;
      continue;
    }
    if (character === "(" || character === "[" || character === "{") {
      const prefix = source.slice(0, index);
      const match = character === "(" ? /([A-Za-z_][A-Za-z0-9_]*)\s*$/.exec(prefix) : undefined;
      stack.push({
        delimiter: character,
        name: match?.[1],
        argument: 0
      });
    } else if (character === ")" || character === "]" || character === "}") {
      const delimiter = character === ")" ? "(" : character === "]" ? "[" : "{";
      for (let stackIndex = stack.length - 1; stackIndex >= 0; stackIndex -= 1) {
        if (stack[stackIndex].delimiter === delimiter) {
          stack.length = stackIndex;
          break;
        }
      }
    } else if (character === "," && stack.length > 0) {
      const current = stack[stack.length - 1];
      if (current.delimiter === "(" && current.name) current.argument += 1;
    }
  }
  const current = stack.at(-1);
  return current?.delimiter === "(" && current.name ? current : undefined;
}

function signatureProvider(vscode) {
  return {
    provideSignatureHelp(document, position) {
      const source = document.getText();
      const context = callContext(source, document.offsetAt(position));
      if (!context) return undefined;

      const userFunction = userFunctions(document).get(context.name);
      const candidates = userFunction
        ? [userFunction.parameters]
        : BUILTINS[context.name]?.signatures;
      if (!candidates) return undefined;

      const help = new vscode.SignatureHelp();
      help.signatures = candidates.map((parameters) => {
        const signature = new vscode.SignatureInformation(signatureText(context.name, parameters));
        signature.parameters = parameters.map((parameter) => (
          new vscode.ParameterInformation(parameter)
        ));
        if (BUILTINS[context.name]) {
          signature.documentation = BUILTINS[context.name].summary;
        } else {
          signature.documentation = `User-defined function (line ${userFunction.line}).`;
        }
        return signature;
      });
      help.activeSignature = candidates.findIndex(
        (parameters) => parameters.length > context.argument
      );
      if (help.activeSignature < 0) help.activeSignature = candidates.length - 1;
      help.activeParameter = Math.min(
        context.argument,
        Math.max(0, candidates[help.activeSignature].length - 1)
      );
      return help;
    }
  };
}

function registerLanguageFeatures(vscode, context) {
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider("pyra", completionProvider(vscode)),
    vscode.languages.registerHoverProvider("pyra", hoverProvider(vscode)),
    vscode.languages.registerSignatureHelpProvider(
      "pyra",
      signatureProvider(vscode),
      "(",
      ","
    )
  );
}

module.exports = {
  registerLanguageFeatures,
  callContext
};
