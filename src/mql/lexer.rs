use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub(crate) struct Span(usize, usize);

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self(start, end)
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self(self.start().min(other.start()), self.end().max(other.end()))
    }

    pub fn start(&self) -> usize {
        self.0
    }

    pub fn end(&self) -> usize {
        self.1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Colon,
    RBracket,
    LBracket,
    RParen,
    LParen,
    RBrace,
    LBrace,
    Dot,
    Comma,
    Semicolon,
    Asterisk,
    ArrowRight,
    ArrowLeft,
    GreaterThan,
    SmallerThan,
    Equals,
    GreaterThanOrEquals,
    SmallerThanOrEquals,
    Identifier(String),
    StringLiteral(String),
    IntLiteral(usize),
    FloatLiteral(f64),
    Select,
    Create,
    Where,
    Like,
    Link,
    From,
    And,
    Or,
    Unknown(char),
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Colon => TokenKind::Colon,
            Token::RBracket => TokenKind::RBracket,
            Token::LBracket => TokenKind::LBracket,
            Token::RParen => TokenKind::RParen,
            Token::LParen => TokenKind::LParen,
            Token::RBrace => TokenKind::RBrace,
            Token::LBrace => TokenKind::LBrace,
            Token::Dot => TokenKind::Dot,
            Token::Comma => TokenKind::Comma,
            Token::Semicolon => TokenKind::Semicolon,
            Token::Asterisk => TokenKind::Asterisk,
            Token::ArrowRight => TokenKind::ArrowRight,
            Token::ArrowLeft => TokenKind::ArrowLeft,
            Token::GreaterThan => TokenKind::GreaterThan,
            Token::SmallerThan => TokenKind::SmallerThan,
            Token::Equals => TokenKind::Equals,
            Token::GreaterThanOrEquals => TokenKind::GreaterThanOrEquals,
            Token::SmallerThanOrEquals => TokenKind::SmallerThanOrEquals,
            Token::Identifier(_) => TokenKind::Identifier,
            Token::StringLiteral(_) => TokenKind::StringLiteral,
            Token::IntLiteral(_) => TokenKind::IntLiteral,
            Token::FloatLiteral(_) => TokenKind::FloatLiteral,
            Token::Select => TokenKind::Select,
            Token::Create => TokenKind::Create,
            Token::Where => TokenKind::Where,
            Token::Like => TokenKind::Like,
            Token::Link => TokenKind::Link,
            Token::From => TokenKind::From,
            Token::And => TokenKind::And,
            Token::Or => TokenKind::Or,
            Token::Unknown(_) => TokenKind::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenKind {
    Colon,
    RBracket,
    LBracket,
    RParen,
    LParen,
    RBrace,
    LBrace,
    Dot,
    Comma,
    Semicolon,
    Asterisk,
    ArrowRight,
    ArrowLeft,
    GreaterThan,
    SmallerThan,
    Equals,
    GreaterThanOrEquals,
    SmallerThanOrEquals,
    Identifier,
    StringLiteral,
    IntLiteral,
    FloatLiteral,
    Select,
    Create,
    Where,
    Like,
    Link,
    From,
    And,
    Or,
    Unknown,
}

pub(crate) struct Lexer<'c> {
    input: Peekable<Chars<'c>>,
    position: usize,
}

impl<'c> Lexer<'c> {
    pub fn new(input: &'c str) -> Self {
        Self {
            input: input.chars().peekable(),
            position: 0,
        }
    }

    fn next_token(&mut self) -> Option<(Token, Span)> {
        let current_position = self.position;
        match self.next_char()? {
            ':' => self.emit_token(current_position, Token::Colon),
            '(' => self.emit_token(current_position, Token::LParen),
            ')' => self.emit_token(current_position, Token::RParen),
            '[' => self.emit_token(current_position, Token::LBracket),
            ']' => self.emit_token(current_position, Token::RBracket),
            '{' => self.emit_token(current_position, Token::RBrace),
            '}' => self.emit_token(current_position, Token::LBrace),
            '.' => self.emit_token(current_position, Token::Dot),
            ',' => self.emit_token(current_position, Token::Comma),
            '*' => self.emit_token(current_position, Token::Asterisk),
            ';' => self.emit_token(current_position, Token::Semicolon),
            '"' => {
                let mut string = String::new();

                while let Some(c) = self.next_char() {
                    match c {
                        '"' => break,
                        '\\' => {
                            if let Some(c) = self.next_char() {
                                match c {
                                    'n' => string.push('\n'),
                                    't' => string.push('\t'),
                                    'r' => string.push('\r'),
                                    '\\' => string.push('\\'),
                                    '"' => string.push('"'),
                                    _ => string.push(c),
                                }
                            }
                        }
                        c => string.push(c),
                    }
                }

                self.emit_token(current_position, Token::StringLiteral(string))
            }
            '-' => {
                if let Some(_) = self.next_char_if(|c| c == '>') {
                    return self.emit_token(current_position, Token::ArrowRight)
                }

                self.emit_token(current_position, Token::Unknown('-'))
            }
            '<' => {
                if let Some(_) = self.next_char_if(|c| c == '=') {
                    self.emit_token(current_position, Token::SmallerThanOrEquals)
                } else {
                    self.emit_token(current_position, Token::SmallerThan)
                }
            }
            '>' => {
                if let Some(_) = self.next_char_if(|c| c == '=') {
                    self.emit_token(current_position, Token::GreaterThanOrEquals)
                } else {
                    self.emit_token(current_position, Token::GreaterThan)
                }
            }
            '=' => self.emit_token(current_position, Token::Equals),
            c if c.is_alphabetic() => {
                let mut identifier = String::from(c);
                while let Some(c) = self.next_char_if(|c| c.is_alphanumeric() || c == '_') {
                    identifier.push(c);
                }

                match identifier.as_str() {
                    "select" => self.emit_token(current_position, Token::Select),
                    "where" => self.emit_token(current_position, Token::Where),
                    "create" => self.emit_token(current_position, Token::Create),
                    "like" => self.emit_token(current_position, Token::Like),
                    "link" => self.emit_token(current_position, Token::Link),
                    "from" => self.emit_token(current_position, Token::From),
                    "and" | "&&" => self.emit_token(current_position, Token::And),
                    "or" | "||" => self.emit_token(current_position, Token::Or),
                    _ => self.emit_token(current_position, Token::Identifier(identifier)),
                }
            }
            c if c.is_numeric() => {
                let mut string = String::from(c);
                while let Some(c) = self.next_char_if(|c| c.is_numeric()) {
                    string.push(c);
                }

                let is_float = if self.peek_nth(0) == Some('.') {
                    if self.peek_nth(1).map_or(false, |c| c.is_numeric()) {
                        string.push('.');
                        self.next_char();

                        while let Some(c) = self.next_char_if(|c| c.is_numeric()) {
                            string.push(c);
                        }

                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_float {
                    self.emit_token(
                        current_position,
                        Token::FloatLiteral(
                            string
                                .parse::<f64>()
                                .expect("This will pass due to previous parsing rules"),
                        ),
                    )
                } else {
                    self.emit_token(
                        current_position,
                        Token::IntLiteral(
                            string
                                .parse::<usize>()
                                .expect("This will pass due to previous parsing rules"),
                        ),
                    )
                }
            }
            c if c.is_whitespace() => {
                while self.next_char_if(|c| c.is_whitespace()).is_some() {}
                self.next_token()
            }
            c => self.emit_token(current_position, Token::Unknown(c)),
        }
    }

    fn emit_token(&self, start: usize, token: Token) -> Option<(Token, Span)> {
        Some((token, Span::new(start, self.position)))
    }

    fn next_char(&mut self) -> Option<char> {
        self.next_char_if(|_| true)
    }

    fn next_char_if(&mut self, f: impl FnOnce(char) -> bool) -> Option<char> {
        self.input.next_if(|&c| f(c)).inspect(|c| {
            self.position += c.len_utf8();
        })
    }

    fn peek_nth(&mut self, n: usize) -> Option<char> {
        self.input.clone().nth(n)
    }
}

impl Iterator for Lexer<'_> {
    type Item = (Token, Span);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
