use crate::ast::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Name(String),
    Number(String),
    String(String),
    Newline,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Operator(String),
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
}

pub fn lex(source: &str, file: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer {
        chars: source.chars().collect(),
        file,
        index: 0,
        line: 1,
        column: 1,
        grouping: 0,
        tokens: Vec::new(),
    };
    lexer.scan()?;
    Ok(lexer.tokens)
}

struct Lexer<'a> {
    chars: Vec<char>,
    file: &'a str,
    index: usize,
    line: usize,
    column: usize,
    grouping: usize,
    tokens: Vec<Token>,
}

impl Lexer<'_> {
    fn scan(&mut self) -> Result<(), String> {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    let position = self.position();
                    self.advance();
                    if self.grouping == 0 {
                        self.push(TokenKind::Newline, position);
                    }
                }
                '#' => {
                    while !matches!(self.peek(), None | Some('\n')) {
                        self.advance();
                    }
                }
                '0'..='9' => self.number(),
                'a'..='z' | 'A'..='Z' | '_' => self.name(),
                '"' | '\'' => self.string(ch)?,
                '(' => self.open(TokenKind::LeftParen),
                '[' => self.open(TokenKind::LeftBracket),
                ')' => self.close(TokenKind::RightParen)?,
                ']' => self.close(TokenKind::RightBracket)?,
                '{' => self.single(TokenKind::LeftBrace),
                '}' => self.single(TokenKind::RightBrace),
                ',' => self.single(TokenKind::Comma),
                ':' => self.single(TokenKind::Colon),
                ';' => self.single(TokenKind::Semicolon),
                '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' => self.operator(),
                _ => return Err(self.error(format!("unexpected character `{ch}`"))),
            }
        }
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            position: self.position(),
        });
        Ok(())
    }

    fn number(&mut self) {
        let position = self.position();
        let start = self.index;
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }
        if self.peek() == Some('.') && matches!(self.peek_next(), Some('0'..='9')) {
            self.advance();
            while matches!(self.peek(), Some('0'..='9')) {
                self.advance();
            }
        }
        self.push(
            TokenKind::Number(self.chars[start..self.index].iter().collect()),
            position,
        );
    }

    fn name(&mut self) {
        let position = self.position();
        let start = self.index;
        while matches!(self.peek(), Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_')) {
            self.advance();
        }
        self.push(
            TokenKind::Name(self.chars[start..self.index].iter().collect()),
            position,
        );
    }

    fn string(&mut self, quote: char) -> Result<(), String> {
        let position = self.position();
        self.advance();
        let mut value = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => {
                    return Err(self.error_at(position, "unterminated string".into()));
                }
                Some(ch) if ch == quote => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    let escaped = match self.peek() {
                        Some('n') => '\n',
                        Some('r') => '\r',
                        Some('t') => '\t',
                        Some('0') => '\0',
                        Some('\\') => '\\',
                        Some('"') => '"',
                        Some('\'') => '\'',
                        Some(other) => {
                            return Err(self.error(format!("unsupported escape `\\{other}`")));
                        }
                        None => return Err(self.error_at(position, "unterminated escape".into())),
                    };
                    self.advance();
                    value.push(escaped);
                }
                Some(ch) => {
                    self.advance();
                    value.push(ch);
                }
            }
        }
        self.push(TokenKind::String(value), position);
        Ok(())
    }

    fn operator(&mut self) {
        let position = self.position();
        let first = self.peek().expect("operator starts with a character");
        self.advance();
        let mut operator = first.to_string();
        if first == '*' && self.peek() == Some('*') {
            operator.push('*');
            self.advance();
        } else if matches!(self.peek(), Some('='))
            && matches!(first, '=' | '!' | '<' | '>' | '+' | '-' | '*' | '/' | '%')
        {
            operator.push('=');
            self.advance();
        }
        self.push(TokenKind::Operator(operator), position);
    }

    fn open(&mut self, kind: TokenKind) {
        let position = self.position();
        self.advance();
        self.grouping += 1;
        self.push(kind, position);
    }

    fn close(&mut self, kind: TokenKind) -> Result<(), String> {
        if self.grouping == 0 {
            return Err(self.error("unmatched closing delimiter".into()));
        }
        let position = self.position();
        self.advance();
        self.grouping -= 1;
        self.push(kind, position);
        Ok(())
    }

    fn single(&mut self, kind: TokenKind) {
        let position = self.position();
        self.advance();
        self.push(kind, position);
    }

    fn push(&mut self, kind: TokenKind, position: Position) {
        self.tokens.push(Token { kind, position });
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.index += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn position(&self) -> Position {
        Position {
            file: self.file.to_owned(),
            line: self.line,
            column: self.column,
        }
    }

    fn error(&self, message: String) -> String {
        self.error_at(self.position(), message)
    }

    fn error_at(&self, position: Position, message: String) -> String {
        format!(
            "{}:{}:{}: error: {}",
            position.file, position.line, position.column, message
        )
    }
}
