mod ast;
mod codegen;
mod lexer;
mod parser;
mod sema;

use lexer::TokenKind;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, PartialEq)]
enum OutputMode {
    Executable,
    Object,
    Assembly,
    SyntaxOnly,
}

struct Options {
    inputs: Vec<PathBuf>,
    output: Option<PathBuf>,
    mode: OutputMode,
    optimization: Option<String>,
    verbose: bool,
}

fn main() {
    let code = match run(env::args_os().skip(1).collect()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("pyra: error: {error}");
            2
        }
    };
    process::exit(code);
}

fn run(arguments: Vec<OsString>) -> Result<i32, String> {
    if arguments.is_empty() {
        print_help();
        return Ok(0);
    }
    let options = parse_arguments(arguments)?;
    let mut tokens = Vec::new();
    for path in &options.inputs {
        let source = fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let file_name = path.display().to_string();
        let mut file_tokens = lexer::lex(&source, &file_name)?;
        let eof = file_tokens.pop().expect("lexer always emits an EOF token");
        file_tokens.push(lexer::Token {
            kind: TokenKind::Newline,
            position: eof.position.clone(),
        });
        tokens.extend(file_tokens);
    }
    let position = tokens
        .last()
        .map(|token| token.position.clone())
        .unwrap_or_else(|| ast::Position {
            file: "<input>".into(),
            line: 1,
            column: 1,
        });
    tokens.push(lexer::Token {
        kind: TokenKind::Eof,
        position,
    });
    let statements = parser::parse(tokens)?;
    sema::check(&statements)?;
    let generated = codegen::generate(&statements);
    if options.mode == OutputMode::SyntaxOnly {
        return compile(&generated, None, &options);
    }
    let output = options
        .output
        .clone()
        .unwrap_or_else(|| default_output(&options.inputs[0], options.mode));
    compile(&generated, Some(&output), &options)
}

fn parse_arguments(arguments: Vec<OsString>) -> Result<Options, String> {
    let mut inputs = Vec::new();
    let mut output = None;
    let mut mode = OutputMode::Executable;
    let mut mode_was_set = false;
    let mut optimization = None;
    let mut verbose = false;
    let mut index = 0;
    while index < arguments.len() {
        let argument = arguments[index].to_string_lossy();
        match argument.as_ref() {
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            "--version" => {
                println!("pyra {VERSION}");
                process::exit(0);
            }
            "-o" => {
                index += 1;
                let path = arguments
                    .get(index)
                    .ok_or_else(|| "`-o` requires an output path".to_owned())?;
                if output.replace(PathBuf::from(path)).is_some() {
                    return Err("`-o` may only be specified once".into());
                }
            }
            "-c" | "-S" | "-fsyntax-only" => {
                if mode_was_set {
                    return Err("choose only one of `-c`, `-S`, and `-fsyntax-only`".into());
                }
                mode_was_set = true;
                mode = match argument.as_ref() {
                    "-c" => OutputMode::Object,
                    "-S" => OutputMode::Assembly,
                    _ => OutputMode::SyntaxOnly,
                };
            }
            "-O0" | "-O1" | "-O2" | "-O3" => {
                if optimization.replace(argument.to_string()).is_some() {
                    return Err("specify only one optimization level".into());
                }
            }
            "-v" => verbose = true,
            _ if argument.starts_with('-') => {
                return Err(format!("unknown option `{argument}`"));
            }
            _ => inputs.push(PathBuf::from(&arguments[index])),
        }
        index += 1;
    }
    if inputs.is_empty() {
        return Err("no input files; try `pyra --help`".into());
    }
    if mode == OutputMode::SyntaxOnly && output.is_some() {
        return Err("`-o` cannot be used with `-fsyntax-only`".into());
    }
    Ok(Options {
        inputs,
        output,
        mode,
        optimization,
        verbose,
    })
}

fn default_output(input: &Path, mode: OutputMode) -> PathBuf {
    let mut path = input.with_extension("");
    match mode {
        OutputMode::Executable => {}
        OutputMode::Object => {
            path.set_extension("o");
        }
        OutputMode::Assembly => {
            path.set_extension("s");
        }
        OutputMode::SyntaxOnly => unreachable!(),
    };
    path
}

fn compile(generated: &str, output: Option<&Path>, options: &Options) -> Result<i32, String> {
    let compiler = env::var_os("PYRA_CXX").unwrap_or_else(|| OsString::from("clang++"));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock error: {error}"))?
        .as_nanos();
    let source_path = env::temp_dir().join(format!("pyra-{}-{nonce}.cpp", process::id()));
    fs::write(&source_path, generated)
        .map_err(|error| format!("cannot write temporary backend source: {error}"))?;

    let mut command = Command::new(&compiler);
    command.arg("-std=c++20");
    if let Some(level) = &options.optimization {
        command.arg(level);
    }
    match options.mode {
        OutputMode::Executable => {}
        OutputMode::Object => {
            command.arg("-c");
        }
        OutputMode::Assembly => {
            command.arg("-S");
        }
        OutputMode::SyntaxOnly => {
            command.arg("-fsyntax-only");
        }
    }
    command.arg(&source_path);
    if let Some(output) = output {
        command.arg("-o").arg(output);
    }
    if options.verbose {
        eprintln!(
            "pyra: invoking {} for {} input file(s)",
            Path::new(&compiler).display(),
            options.inputs.len()
        );
        eprintln!(
            "pyra: mode {:?}",
            match options.mode {
                OutputMode::Executable => "executable",
                OutputMode::Object => "object",
                OutputMode::Assembly => "assembly",
                OutputMode::SyntaxOnly => "syntax-only",
            }
        );
        eprintln!(
            "pyra: output {}",
            output.map_or_else(|| "<none>".into(), |path| path.display().to_string())
        );
    }
    let result = command.status().map_err(|error| {
        format!(
            "cannot run C++ backend `{}`: {error}",
            Path::new(&compiler).display()
        )
    });
    let cleanup = fs::remove_file(&source_path);
    if let Err(error) = cleanup
        && error.kind() != std::io::ErrorKind::NotFound
    {
        eprintln!(
            "pyra: warning: cannot remove temporary file {}: {error}",
            source_path.display()
        );
    }
    let status = result?;
    Ok(status.code().unwrap_or(1))
}

fn print_help() {
    println!(
        "Pyra {VERSION} - brace-delimited compiled language\n\
Usage: pyra [options] file.pyra [file.pyra ...]\n\
\n\
Options:\n\
  -o PATH          Write output to PATH\n\
  -c               Compile to a native object file\n\
  -S               Compile to assembly\n\
  -fsyntax-only    Check syntax and semantics without generating output\n\
  -O0|-O1|-O2|-O3  Select the native backend optimization level\n\
  -v               Print compilation details\n\
  --help           Show this help\n\
  --version        Show compiler version\n\
\n\
Set PYRA_CXX to select the C++20 compiler used by the native backend."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_name_uses_input_stem() {
        assert_eq!(
            default_output(
                Path::new("examples/hello world.pyra"),
                OutputMode::Executable
            ),
            PathBuf::from("examples/hello world")
        );
        assert_eq!(
            default_output(Path::new("hello.pyra"), OutputMode::Object),
            PathBuf::from("hello.o")
        );
    }

    #[test]
    fn rejects_conflicting_output_modes() {
        let args = ["-c", "-S", "sample.pyra"]
            .into_iter()
            .map(OsString::from)
            .collect();
        assert!(parse_arguments(args).is_err());
    }

    #[test]
    fn parses_compound_assignment_tokens() {
        let tokens = lexer::lex("count += 1\n", "test.pyra").unwrap();
        assert!(matches!(
            &tokens[1].kind,
            TokenKind::Operator(operator) if operator == "+="
        ));
    }
}
