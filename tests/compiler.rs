use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "pyra-e2e-{}-{sequence}-with spaces",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn pyra(arguments: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pyra"))
        .args(arguments)
        .output()
        .unwrap()
}

fn arg(path: &std::path::Path) -> &std::ffi::OsStr {
    path.as_os_str()
}

#[test]
fn compiles_a_standalone_program_with_functions_and_collections() {
    let directory = TestDirectory::new();
    let source = directory.file("factorial and lists.pyra");
    let executable = directory.file("standalone program");
    fs::write(
        &source,
        r#"
def factorial(number) {
    if number <= 1 {
        return 1;
    }
    return number * factorial(number - 1);
}
values = [1, 2, 3];
scores = {"answer": 5};
total = 0;
for value in values {
    total += value;
}
if total == 6 and scores["answer"] in [5] {
    print(factorial(5), total, values[-1], values[0:2]);
} else {
    print("wrong");
}
"#,
    )
    .unwrap();

    let compilation = pyra(&[arg(&source), OsStrExt::as_os_str("-o"), arg(&executable)]);
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
    assert!(executable.exists());
    let execution = Command::new(&executable).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8_lossy(&execution.stdout).trim(),
        "120 6 3 [1, 2]"
    );
}

#[test]
fn preserves_precision_for_integer_sums_beyond_64_bits() {
    let directory = TestDirectory::new();
    let source = directory.file("quadratic-sum.pyra");
    let executable = directory.file("quadratic-sum");
    fs::write(
        &source,
        r#"
def quadratic_sum(n) {
    total = 0;
    for i in range(n) {
        total = total + (i * i);
    }
    return total;
}
print(quadratic_sum(10000000));
"#,
    )
    .unwrap();

    let compilation = pyra(&[arg(&source), OsStrExt::as_os_str("-o"), arg(&executable)]);
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
    let execution = Command::new(executable).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8_lossy(&execution.stdout).trim(),
        "333333283333335000000"
    );
}

#[test]
fn supports_syntax_only_object_and_assembly_modes() {
    let directory = TestDirectory::new();
    let source = directory.file("simple.pyra");
    fs::write(&source, "print(2 + 3);\n").unwrap();
    assert!(
        pyra(&[OsStrExt::as_os_str("-fsyntax-only"), arg(&source)])
            .status
            .success()
    );

    let object = directory.file("simple.o");
    assert!(
        pyra(&[
            OsStrExt::as_os_str("-c"),
            arg(&source),
            OsStrExt::as_os_str("-o"),
            arg(&object)
        ])
        .status
        .success()
    );
    assert!(object.exists());

    let assembly = directory.file("simple.s");
    assert!(
        pyra(&[
            OsStrExt::as_os_str("-S"),
            arg(&source),
            OsStrExt::as_os_str("-o"),
            arg(&assembly)
        ])
        .status
        .success()
    );
    assert!(assembly.exists());
}

#[test]
fn reports_source_locations_and_unknown_functions() {
    let directory = TestDirectory::new();
    let source = directory.file("bad.pyra");
    fs::write(&source, "\nprint(missing_function());\n").unwrap();
    let output = pyra(&[arg(&source)]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("bad.pyra:2:7: error: unknown function `missing_function`"));
}

#[test]
fn checks_short_circuiting_and_runtime_exit_status() {
    let directory = TestDirectory::new();
    let source = directory.file("short-circuit.pyra");
    let executable = directory.file("short-circuit");
    fs::write(
        &source,
        "if False and missing_name {\n    print(1);\n}\nprint(True or missing_name);\n",
    )
    .unwrap();
    let compilation = pyra(&[arg(&source), OsStrExt::as_os_str("-o"), arg(&executable)]);
    assert!(compilation.status.success());
    let execution = Command::new(&executable).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8_lossy(&execution.stdout).trim(), "True");
}

#[test]
fn compiles_multiple_source_files_together() {
    let directory = TestDirectory::new();
    let library = directory.file("math.pyra");
    let program = directory.file("main.pyra");
    let executable = directory.file("multiple");
    fs::write(&library, "def double(value) {\n    return value * 2;\n}\n").unwrap();
    fs::write(&program, "print(double(21));\n").unwrap();
    let compilation = pyra(&[
        arg(&program),
        arg(&library),
        OsStrExt::as_os_str("-o"),
        arg(&executable),
    ]);
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
    let execution = Command::new(executable).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(String::from_utf8_lossy(&execution.stdout).trim(), "42");
}

#[test]
fn evaluates_operands_left_to_right_and_compares_types_correctly() {
    let directory = TestDirectory::new();
    let source = directory.file("evaluation.pyra");
    let executable = directory.file("evaluation");
    fs::write(
        &source,
        r#"
def left() {
    print("left");
    return 8;
}
def right() {
    print("right");
    return 2;
}
print(left() / right());
print("1" == 1);
print(5.5 % 2.0);
"#,
    )
    .unwrap();
    let compilation = pyra(&[arg(&source), OsStrExt::as_os_str("-o"), arg(&executable)]);
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
    let execution = Command::new(executable).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8_lossy(&execution.stdout).trim(),
        "left\nright\n4\nFalse\n1.5"
    );
}

#[test]
fn requires_semicolons_after_simple_statements() {
    let directory = TestDirectory::new();
    let source = directory.file("missing-semicolon.pyra");
    fs::write(&source, "value = 1\nprint(value);\n").unwrap();
    let output = pyra(&[arg(&source)]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "missing-semicolon.pyra:1:10: error: expected `;` at the end of the statement"
        )
    );

    fs::write(
        &source,
        "value = 1;\nif value == 1 {\n    print(value);\n}\n",
    )
    .unwrap();
    let output = pyra(&[OsStrExt::as_os_str("-fsyntax-only"), arg(&source)]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

struct OsStrExt;
impl OsStrExt {
    fn as_os_str(value: &str) -> &std::ffi::OsStr {
        std::ffi::OsStr::new(value)
    }
}
