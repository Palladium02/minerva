use std::iter::Peekable;
use crate::mql::lexer::{Lexer, Span, Token, TokenKind};

#[derive(Debug)]
pub(crate) enum ParseError {
    UnexpectedEOF,
    UnexpectedToken(Token),
    InvalidProjection,
}

#[derive(Debug)]
pub(crate) enum AST {
    // TODO: insert actual type for filter expression
    Select(Projection, PathExpression, Option<()>)
}

#[derive(Debug)]
pub(crate) enum Projection {
    All,
    Fields(Vec<String>),
}

#[derive(Debug)]
pub(crate) enum EntityDescription {
    NoId(String),
    WithId(String, String),
}

#[derive(Debug)]
pub(crate) struct PathExpression(EntityDescription, Vec<EntityDescription>);

pub(crate) struct Parser<'t> {
    input: Peekable<Lexer<'t>>,
}

impl<'t> Parser<'t> {
    pub fn new(input: Lexer<'t>) -> Self {
        Self { input: input.peekable() }
    }

    pub fn parse(&mut self) -> Result<AST, ParseError> {
        let (first, _) = self.input.peek().ok_or(ParseError::UnexpectedEOF)?;

        match first {
            Token::Select => self.expect_select_statement(),
            Token::Create => self.expect_create_statement(),
            Token::Link => self.expect_link_statement(),
            _ => Err(ParseError::UnexpectedToken(first.clone())),
        }
    }

    fn expect_select_statement(&mut self) -> Result<AST, ParseError> {
        self.expect_token_type(TokenKind::Select)?;
        let projection = self.expect_projection()?;
        self.expect_token_type(TokenKind::From)?;
        let path_expression = self.expect_path_expression()?;

        let peeked_next = self.input.peek();

        if let Some((token, span)) = peeked_next {
            match token {
                Token::Semicolon => {
                    self.input.next();
                    return Ok(AST::Select(projection, path_expression, None));
                },
                _ => {
                    let _ = self.expect_filter_condition()?;
                    // TODO: actually pass in what expect_filter_condition returns
                    return Ok(AST::Select(projection, path_expression, Some(())));
                },
            }
        }

        Err(ParseError::UnexpectedEOF)

    }

    fn expect_create_statement(&mut self) -> Result<AST, ParseError> {
        todo!()
    }

    fn expect_link_statement(&mut self) -> Result<AST, ParseError> {
        todo!()
    }

    fn expect_projection(&mut self) -> Result<Projection, ParseError> {
        let peeked_next = self.input.peek();

        match peeked_next {
            Some((token, _)) => {
                if let Token::Asterisk = token {
                    self.input.next();
                    return Ok(Projection::All)
                }

                let mut fields = Vec::new();

                loop {
                    let (token, _) = self.input.next().ok_or(ParseError::UnexpectedEOF)?;
                    if let Token::Identifier(field) = token {
                        fields.push(field);
                        if let Some((peeked_next, _)) = self.input.peek() {
                            if *peeked_next == Token::Comma {
                                self.input.next();
                                continue;
                            }
                        }
                    } else {
                        break;
                    }
                }

                if fields.is_empty() {
                    return Err(ParseError::InvalidProjection);
                }

                Ok(Projection::Fields(fields))
            },
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    fn expect_path_expression(&mut self) -> Result<PathExpression, ParseError> {
        let source = self.expect_entity_description()?;
        let mut path = Vec::new();

        loop {
            if let Some((peeked_next, _)) = self.input.peek() {
                match peeked_next {
                    Token::ArrowRight => {
                        self.input.next();
                        let step = self.expect_entity_description()?;
                        path.push(step);
                        continue;
                    }
                    _ => break,
                }
            } else {
                return Err(ParseError::UnexpectedEOF);
            }
        }

        Ok(PathExpression(source, path))
    }

    fn expect_entity_description(&mut self) -> Result<EntityDescription, ParseError> {
        let typename = self.expect_token_type(TokenKind::Identifier)?;
        if let Some((peeked_next, _)) = self.input.peek() {
            if *peeked_next == Token::Colon {
                self.input.next();
                let id = self.expect_token_type(TokenKind::Identifier)?;

                if let Token::Identifier(id) = id && let Token::Identifier(typename) = typename {
                    return Ok(EntityDescription::WithId(id, typename));
                }

                unreachable!()
            }
        }

        if let Token::Identifier(typename) = typename {
            return Ok(EntityDescription::NoId(typename.to_string()))
        }

        unreachable!()
    }

    fn expect_filter_condition(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn expect_token_type(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        match self.input.next() {
            Some((token, _)) if token.kind() == kind => Ok(token),
            Some((token, _)) => Err(ParseError::UnexpectedToken(token)),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}
