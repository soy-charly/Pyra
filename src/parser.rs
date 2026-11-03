use crate::ast::{Expr, ExprKind, Position, Stmt, StmtKind};
use crate::lexer::{Token, TokenKind};

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, String> {
    Parser { tokens, index: 0 }.program()
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn program(mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        self.separators();
        while !self.at(&TokenKind::Eof) {
            statements.push(self.statement()?);
            self.separators();
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        let position = self.current().position.clone();
        if self.keyword("def") {
            return self.function(position);
        }
        if self.keyword("if") {
            return self.if_statement(position);
        }
        if self.keyword("while") {
            self.advance();
            let condition = self.expression(0)?;
            let body = self.block()?;
            return Ok(Stmt {
                kind: StmtKind::While { condition, body },
                position,
            });
        }
        if self.keyword("for") {
            self.advance();
            let name = self.expect_name("expected a loop variable")?;
            self.expect_keyword("in")?;
            let iterable = self.expression(0)?;
            let body = self.block()?;
            return Ok(Stmt {
                kind: StmtKind::For {
                    name,
                    iterable,
                    body,
                },
                position,
            });
        }
        if self.keyword("return") {
            self.advance();
            let value = if self.at_statement_end() {
                None
            } else {
                Some(self.expression(0)?)
            };
            self.finish_simple_statement()?;
            return Ok(Stmt {
                kind: StmtKind::Return(value),
                position,
            });
        }
        if self.keyword("break") {
            self.advance();
            self.finish_simple_statement()?;
            return Ok(Stmt {
                kind: StmtKind::Break,
                position,
            });
        }
        if self.keyword("continue") {
            self.advance();
            self.finish_simple_statement()?;
            return Ok(Stmt {
                kind: StmtKind::Continue,
                position,
            });
        }
        if let TokenKind::Name(name) = &self.current().kind
            && let TokenKind::Operator(operator) = &self.peek(1).kind
            && matches!(operator.as_str(), "=" | "+=" | "-=" | "*=" | "/=" | "%=")
        {
            let name = name.clone();
            let operator = operator.clone();
            self.advance();
            self.advance();
            let value = self.expression(0)?;
            self.finish_simple_statement()?;
            return Ok(Stmt {
                kind: StmtKind::Assign {
                    name,
                    operator,
                    value,
                },
                position,
            });
        }
        let expression = self.expression(0)?;
        self.finish_simple_statement()?;
        Ok(Stmt {
            kind: StmtKind::Expr(expression),
            position,
        })
    }

    fn function(&mut self, position: Position) -> Result<Stmt, String> {
        self.advance();
        let name = self.expect_name("expected a function name")?;
        self.expect(&TokenKind::LeftParen, "expected `(` after function name")?;
        let mut parameters = Vec::new();
        if !self.at(&TokenKind::RightParen) {
            loop {
                parameters.push(self.expect_name("expected a parameter name")?);
                if !self.take(&TokenKind::Comma) {
                    break;
                }
                if self.at(&TokenKind::RightParen) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RightParen, "expected `)` after parameters")?;
        let body = self.block()?;
        Ok(Stmt {
            kind: StmtKind::Function {
                name,
                parameters,
                body,
            },
            position,
        })
    }

    fn if_statement(&mut self, position: Position) -> Result<Stmt, String> {
        self.advance();
        let condition = self.expression(0)?;
        let body = self.block()?;
        let mut branches = vec![(condition, body)];
        self.separators();
        while self.keyword("elif") {
            self.advance();
            let condition = self.expression(0)?;
            let body = self.block()?;
            branches.push((condition, body));
            self.separators();
        }
        let otherwise = if self.keyword("else") {
            self.advance();
            self.block()?
        } else {
            Vec::new()
        };
        Ok(Stmt {
            kind: StmtKind::If {
                branches,
                otherwise,
            },
            position,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(&TokenKind::LeftBrace, "expected `{` to begin block")?;
        let mut statements = Vec::new();
        self.separators();
        while !self.at(&TokenKind::RightBrace) && !self.at(&TokenKind::Eof) {
            statements.push(self.statement()?);
            self.separators();
        }
        self.expect(&TokenKind::RightBrace, "expected `}` to close block")?;
        Ok(statements)
    }

    fn expression(&mut self, minimum_precedence: u8) -> Result<Expr, String> {
        let mut left = self.prefix()?;
        loop {
            if self.at(&TokenKind::LeftParen) && 9 >= minimum_precedence {
                let position = left.position.clone();
                self.advance();
                let mut arguments = Vec::new();
                if !self.at(&TokenKind::RightParen) {
                    loop {
                        arguments.push(self.expression(0)?);
                        if !self.take(&TokenKind::Comma) {
                            break;
                        }
                        if self.at(&TokenKind::RightParen) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RightParen, "expected `)` after arguments")?;
                left = Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(left),
                        arguments,
                    },
                    position,
                };
                continue;
            }
            if self.at(&TokenKind::LeftBracket) && 9 >= minimum_precedence {
                left = self.index_or_slice(left)?;
                continue;
            }
            let Some((operator, precedence)) = self.infix() else {
                break;
            };
            if precedence < minimum_precedence {
                break;
            }
            let position = left.position.clone();
            self.advance();
            let right = self.expression(precedence + 1)?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                position,
            };
        }
        Ok(left)
    }

    fn prefix(&mut self) -> Result<Expr, String> {
        let token = self.advance().clone();
        let kind = match token.kind {
            TokenKind::Number(number) => ExprKind::Number(number),
            TokenKind::String(value) => ExprKind::String(value),
            TokenKind::Name(name) if name == "True" => ExprKind::Bool(true),
            TokenKind::Name(name) if name == "False" => ExprKind::Bool(false),
            TokenKind::Name(name) if name == "None" => ExprKind::None,
            TokenKind::Name(name) if name == "not" => {
                let operand = self.expression(8)?;
                ExprKind::Unary {
                    operator: name,
                    operand: Box::new(operand),
                }
            }
            TokenKind::Operator(operator) if matches!(operator.as_str(), "-" | "+") => {
                let operand = self.expression(8)?;
                ExprKind::Unary {
                    operator,
                    operand: Box::new(operand),
                }
            }
            TokenKind::Name(name) => ExprKind::Name(name),
            TokenKind::LeftParen => {
                let expression = self.expression(0)?;
                self.expect(&TokenKind::RightParen, "expected `)` after expression")?;
                return Ok(expression);
            }
            TokenKind::LeftBracket => {
                let mut items = Vec::new();
                if !self.at(&TokenKind::RightBracket) {
                    loop {
                        items.push(self.expression(0)?);
                        if !self.take(&TokenKind::Comma) {
                            break;
                        }
                        if self.at(&TokenKind::RightBracket) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RightBracket, "expected `]` after list")?;
                ExprKind::List(items)
            }
            TokenKind::LeftBrace => {
                let mut items = Vec::new();
                self.newlines();
                if !self.at(&TokenKind::RightBrace) {
                    loop {
                        let key = self.expression(0)?;
                        self.expect(&TokenKind::Colon, "expected `:` between key and value")?;
                        let value = self.expression(0)?;
                        items.push((key, value));
                        self.newlines();
                        if !self.take(&TokenKind::Comma) {
                            break;
                        }
                        self.newlines();
                        if self.at(&TokenKind::RightBrace) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RightBrace, "expected `}` after dictionary")?;
                ExprKind::Dict(items)
            }
            _ => {
                return Err(self.error_at(token.position, "expected an expression".into()));
            }
        };
        Ok(Expr {
            kind,
            position: token.position,
        })
    }

    fn index_or_slice(&mut self, value: Expr) -> Result<Expr, String> {
        let position = value.position.clone();
        self.advance();
        let first = if self.at(&TokenKind::Colon) {
            None
        } else {
            Some(Box::new(self.expression(0)?))
        };
        if self.take(&TokenKind::Colon) {
            let end = if self.at(&TokenKind::Colon) || self.at(&TokenKind::RightBracket) {
                None
            } else {
                Some(Box::new(self.expression(0)?))
            };
            let step = if self.take(&TokenKind::Colon) {
                if self.at(&TokenKind::RightBracket) {
                    None
                } else {
                    Some(Box::new(self.expression(0)?))
                }
            } else {
                None
            };
            self.expect(&TokenKind::RightBracket, "expected `]` after slice")?;
            Ok(Expr {
                kind: ExprKind::Slice {
                    value: Box::new(value),
                    start: first,
                    end,
                    step,
                },
                position,
            })
        } else {
            let index = first.ok_or_else(|| self.error("expected an index".into()))?;
            self.expect(&TokenKind::RightBracket, "expected `]` after index")?;
            Ok(Expr {
                kind: ExprKind::Index {
                    value: Box::new(value),
                    index,
                },
                position,
            })
        }
    }

    fn infix(&self) -> Option<(String, u8)> {
        let operator = match &self.current().kind {
            TokenKind::Operator(operator) => operator.as_str(),
            TokenKind::Name(word) => word.as_str(),
            _ => return None,
        };
        let precedence = match operator {
            "or" => 1,
            "and" => 2,
            "==" | "!=" | "<" | "<=" | ">" | ">=" | "in" => 3,
            "+" | "-" => 4,
            "*" | "/" | "%" => 5,
            "**" => 6,
            _ => return None,
        };
        Some((operator.to_owned(), precedence))
    }

    fn expect_name(&mut self, message: &str) -> Result<String, String> {
        if let TokenKind::Name(name) = &self.current().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.error(message.into()))
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        if self.keyword(keyword) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("expected `{keyword}`")))
        }
    }

    fn expect(&mut self, expected: &TokenKind, message: &str) -> Result<(), String> {
        if self.at(expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(message.into()))
        }
    }

    fn take(&mut self, expected: &TokenKind) -> bool {
        if self.at(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn at(&self, expected: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(expected)
    }

    fn keyword(&self, keyword: &str) -> bool {
        matches!(&self.current().kind, TokenKind::Name(name) if name == keyword)
    }

    fn at_statement_end(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Newline | TokenKind::Semicolon | TokenKind::RightBrace | TokenKind::Eof
        )
    }

    fn finish_simple_statement(&mut self) -> Result<(), String> {
        self.expect(
            &TokenKind::Semicolon,
            "expected `;` at the end of the statement",
        )
    }

    fn separators(&mut self) {
        while matches!(
            self.current().kind,
            TokenKind::Newline | TokenKind::Semicolon
        ) {
            self.advance();
        }
    }

    fn newlines(&mut self) {
        while self.at(&TokenKind::Newline) {
            self.advance();
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn peek(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.index + offset)
            .unwrap_or_else(|| self.tokens.last().expect("lexer always emits EOF"))
    }

    fn advance(&mut self) -> &Token {
        let index = self.index;
        if !self.at(&TokenKind::Eof) {
            self.index += 1;
        }
        &self.tokens[index]
    }

    fn error(&self, message: String) -> String {
        self.error_at(self.current().position.clone(), message)
    }

    fn error_at(&self, position: Position, message: String) -> String {
        format!(
            "{}:{}:{}: error: {}",
            position.file, position.line, position.column, message
        )
    }
}
