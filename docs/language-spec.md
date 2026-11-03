# Pyra language specification (0.1)

Pyra is Python-inspired, not Python-compatible. `.pyra` is the conventional source extension. Blocks use `{` and `}`; indentation is insignificant. Every simple statement must end with `;`. Newlines and semicolons separate statements outside parentheses and brackets. Block headers and closing braces do not need a semicolon. `#` begins a line comment.

## Values and names

Values are dynamically represented as `None`, booleans (`True`, `False`), signed integers, floating-point numbers, strings, lists, and dictionaries with stringified keys. Names use ASCII letters or `_` initially, followed by letters, digits, or `_`. Variables are introduced by assignment and live in the current function's environment. Functions have their own local environment; global-variable lookup and closures are not implemented.

Strings use single or double quotes and support `\\`, `\"`, `\'`, `\n`, `\r`, `\t`, and `\0`. Triple-quoted and interpolated strings are not implemented.

## Statements

```pyra
value = 2;
value += 3;

if value > 3 {
    print("large");
} elif value == 3 {
    print("three");
} else {
    print("small");
}

while value > 0 {
    value -= 1;
}

for item in range(3) {
    print(item);
}

def twice(number) {
    return number * 2;
}
```

Supported statements: expression statements, `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `if`/`elif`/`else`, `while`, `for name in iterable`, function definitions, `return`, `break`, and `continue`. Simple statements (expressions, assignments, `return`, `break`, and `continue`) require a trailing `;`; compound statements (blocks and their headers) do not. Functions are top-level, have fixed positional parameters, and may recurse. Imports, exceptions, classes, keyword/default arguments, nested functions, and assignment to an indexed element are not supported.

## Expressions and precedence

From lowest to highest: `or`; `and`; comparisons (`==`, `!=`, `<`, `<=`, `>`, `>=`, `in`); `+`, `-`; `*`, `/`, `%`; `**`; unary `not`, `+`, `-`; calls and indexing. Parentheses group expressions. `and` and `or` short-circuit and return one of their operands.

Lists use `[item, ...]`; dictionaries use `{key: value, ...}`. Indexing and slicing (`value[index]`, `value[start:end:step]`) are supported for strings and lists. Iteration and `in` support strings, lists, and dictionaries; iterating a dictionary yields its keys. `range`, `len`, `print`, `input`, `str`, `int`, `float`, `bool`, and `type` are built-ins.

## Runtime and errors

The runtime is embedded in each generated native program, uses Boost.Multiprecision for arbitrary-precision integers, and uses C++ standard-library values with shared ownership for collection storage. Collection literals, concatenation, indexing, slicing, membership, conversions, and iteration are implemented. List/dictionary mutation methods and a module system are not.

Lexical, parse, and semantic errors report the source path, line, and column. Runtime errors are printed to standard error and exit with status 1. Numeric and collection behavior is intentionally a small subset; do not rely on exact Python coercion, formatting, Unicode indexing, or overflow behavior.
