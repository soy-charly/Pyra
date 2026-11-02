#[derive(Clone, Debug)]
pub struct Position {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub position: Position,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    None,
    Bool(bool),
    Number(String),
    String(String),
    Name(String),
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Unary {
        operator: String,
        operand: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: String,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Index {
        value: Box<Expr>,
        index: Box<Expr>,
    },
    Slice {
        value: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
    },
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub position: Position,
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Expr(Expr),
    Assign {
        name: String,
        operator: String,
        value: Expr,
    },
    If {
        branches: Vec<(Expr, Vec<Stmt>)>,
        otherwise: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        name: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
}
