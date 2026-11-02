use crate::ast::{Expr, ExprKind, Stmt, StmtKind};
use std::collections::HashSet;

const BUILTINS: &[&str] = &[
    "print", "input", "len", "range", "str", "int", "float", "bool", "type",
];

pub fn check(statements: &[Stmt]) -> Result<(), String> {
    let mut functions = HashSet::new();
    for statement in statements {
        if let StmtKind::Function {
            name, parameters, ..
        } = &statement.kind
        {
            if BUILTINS.contains(&name.as_str()) || !functions.insert(name) {
                return Err(diagnostic(
                    &statement.position,
                    format!("function `{name}` is already defined or reserved"),
                ));
            }
            let mut seen = HashSet::new();
            if parameters.iter().any(|parameter| !seen.insert(parameter)) {
                return Err(diagnostic(
                    &statement.position,
                    format!("function `{name}` has a duplicate parameter"),
                ));
            }
        }
    }

    let mut checker = Checker {
        functions: &functions,
        function_depth: 0,
        loop_depth: 0,
    };
    checker.statements(statements)
}

struct Checker<'a> {
    functions: &'a HashSet<&'a String>,
    function_depth: usize,
    loop_depth: usize,
}

impl Checker<'_> {
    fn statements(&mut self, statements: &[Stmt]) -> Result<(), String> {
        for statement in statements {
            match &statement.kind {
                StmtKind::Expr(expression) => self.expression(expression)?,
                StmtKind::Assign { value, .. } => self.expression(value)?,
                StmtKind::If {
                    branches,
                    otherwise,
                } => {
                    for (condition, body) in branches {
                        self.expression(condition)?;
                        self.statements(body)?;
                    }
                    self.statements(otherwise)?;
                }
                StmtKind::While { condition, body } => {
                    self.expression(condition)?;
                    self.loop_depth += 1;
                    self.statements(body)?;
                    self.loop_depth -= 1;
                }
                StmtKind::For { iterable, body, .. } => {
                    self.expression(iterable)?;
                    self.loop_depth += 1;
                    self.statements(body)?;
                    self.loop_depth -= 1;
                }
                StmtKind::Function { body, .. } => {
                    if self.function_depth > 0 {
                        return Err(diagnostic(
                            &statement.position,
                            "nested function definitions are not supported".into(),
                        ));
                    }
                    self.function_depth += 1;
                    self.statements(body)?;
                    self.function_depth -= 1;
                }
                StmtKind::Return(value) => {
                    if self.function_depth == 0 {
                        return Err(diagnostic(
                            &statement.position,
                            "`return` is only valid inside a function".into(),
                        ));
                    }
                    if let Some(value) = value {
                        self.expression(value)?;
                    }
                }
                StmtKind::Break | StmtKind::Continue if self.loop_depth == 0 => {
                    let keyword = if matches!(statement.kind, StmtKind::Break) {
                        "break"
                    } else {
                        "continue"
                    };
                    return Err(diagnostic(
                        &statement.position,
                        format!("`{keyword}` is only valid inside a loop"),
                    ));
                }
                StmtKind::Break | StmtKind::Continue => {}
            }
        }
        Ok(())
    }

    fn expression(&self, expression: &Expr) -> Result<(), String> {
        match &expression.kind {
            ExprKind::Unary { operand, .. } => self.expression(operand)?,
            ExprKind::Binary { left, right, .. } => {
                self.expression(left)?;
                self.expression(right)?;
            }
            ExprKind::Call { callee, arguments } => {
                let ExprKind::Name(name) = &callee.kind else {
                    return Err(diagnostic(
                        &callee.position,
                        "only named functions can be called".into(),
                    ));
                };
                if !BUILTINS.contains(&name.as_str())
                    && !self
                        .functions
                        .iter()
                        .any(|function| function.as_str() == name)
                {
                    return Err(diagnostic(
                        &callee.position,
                        format!("unknown function `{name}`"),
                    ));
                }
                for argument in arguments {
                    self.expression(argument)?;
                }
            }
            ExprKind::List(items) => {
                for item in items {
                    self.expression(item)?;
                }
            }
            ExprKind::Dict(items) => {
                for (key, value) in items {
                    self.expression(key)?;
                    self.expression(value)?;
                }
            }
            ExprKind::Index { value, index } => {
                self.expression(value)?;
                self.expression(index)?;
            }
            ExprKind::Slice {
                value,
                start,
                end,
                step,
            } => {
                self.expression(value)?;
                for expression in [start, end, step].into_iter().flatten() {
                    self.expression(expression)?;
                }
            }
            ExprKind::None
            | ExprKind::Bool(_)
            | ExprKind::Number(_)
            | ExprKind::String(_)
            | ExprKind::Name(_) => {}
        }
        Ok(())
    }
}

fn diagnostic(position: &crate::ast::Position, message: String) -> String {
    format!(
        "{}:{}:{}: error: {}",
        position.file, position.line, position.column, message
    )
}
