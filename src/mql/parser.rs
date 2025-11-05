use std::iter::Peekable;
use std::collections::HashMap;
use crate::mql::lexer::{Lexer, Token, TokenKind};

#[derive(Debug)]
pub(crate) enum ParseError {
    UnexpectedEOF,
    UnexpectedToken(Token),
    InvalidProjection,
}

#[derive(Debug)]
pub(crate) enum AST {
    // TODO: insert actual type for filter expression
    Select(Projection, PathExpression, Option<FilterExpression>),
    Create(EntityDescription, HashMap<String, Value>),
    Link(EntityDescription, EntityDescription),
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

#[derive(Debug)]
pub(crate) enum FilterExpression {
    Plain(String, Operator, Value),
    And(Box<FilterExpression>, Box<FilterExpression>),
    Or(Box<FilterExpression>, Box<FilterExpression>),
}

#[derive(Debug)]
pub(crate) enum Operator {
    Equals,
    NotEquals,
    SmallerThan,
    GreaterThan,
    SmallerThanOrEqual,
    GreaterThanOrEqual,
    Like,
}

impl TryFrom<Token> for Operator {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Equals => Ok(Operator::Equals),
            Token::NotEquals => Ok(Operator::NotEquals),
            Token::SmallerThan => Ok(Operator::SmallerThan),
            Token::GreaterThan => Ok(Operator::GreaterThan),
            Token::SmallerThanOrEquals => Ok(Operator::SmallerThanOrEqual),
            Token::GreaterThanOrEquals => Ok(Operator::GreaterThanOrEqual),
            Token::Like => Ok(Operator::Like),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Value {
    String(String),
    Int(usize),
    Float(f64),
}

impl TryFrom<Token> for Value {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::StringLiteral(string) => Ok(Value::String(string)),
            Token::IntLiteral(int) => Ok(Value::Int(int)),
            Token::FloatLiteral(float) => Ok(Value::Float(float)),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
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
                    let filter_expression = self.expect_filter_condition()?;
                    // TODO: actually pass in what expect_filter_condition returns
                    return Ok(AST::Select(projection, path_expression, Some(filter_expression)));
                },
            }
        }

        Err(ParseError::UnexpectedEOF)

    }

    fn expect_create_statement(&mut self) -> Result<AST, ParseError> {
        let _ = self.expect_token_type(TokenKind::Create)?;
        let entity_description = self.expect_entity_description()?;
        let values = self.expect_key_value_pairs()?;
        let _ = self.expect_token_type(TokenKind::Semicolon);

        Ok(AST::Create(entity_description, values))
    }

    fn expect_link_statement(&mut self) -> Result<AST, ParseError> {
        let _ = self.expect_token_type(TokenKind::Link);
        let lhs_entity_description = self.expect_entity_description()?;
        let _ = self.expect_token_type(TokenKind::ArrowRight);
        let rhs_entity_description = self.expect_entity_description()?;

        Ok(AST::Link(lhs_entity_description, rhs_entity_description))
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
                    // peek first then consume if identifier to prevent the parser to consume the
                    // from token that comes after a projection
                    let (token, _) = self.input.peek().ok_or(ParseError::UnexpectedEOF)?;
                    if let Token::Identifier(field) = token {
                        fields.push(field.to_string());
                        self.input.next();
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

    fn expect_filter_condition(&mut self) -> Result<FilterExpression, ParseError> {
        let _ = self.expect_token_type(TokenKind::Where)?;
        let filter_expression = self.expect_expression()?;
        let _ = self.expect_token_type(TokenKind::Semicolon);

        Ok(filter_expression)
    }

    fn expect_expression(&mut self) -> Result<FilterExpression, ParseError> {
        let expression = match self.input.next() {
            Some((Token::LParen, _)) => {
                let expression = self.expect_expression()?;
                let _ = self.expect_token_type(TokenKind::RParen)?;

                Ok(expression)
            },
            Some((token, _)) => {
                if let Token::Identifier(identifier) = token.clone() {
                    let operator = self.expect_operator()?;
                    let value = self.expect_value()?;
                    Ok(FilterExpression::Plain(identifier, operator, value))
                } else {
                    Err(ParseError::UnexpectedToken(token))
                }
            },
            None => Err(ParseError::UnexpectedEOF),
        };

        let mut cloned_input = self.input.clone();
        let peeked_next = cloned_input.peek();
        match peeked_next {
            Some((token @ (Token::And | Token::Or), _)) => {
                let rhs = self.expect_expression()?;
                match token {
                    Token::And => Ok(FilterExpression::And(Box::new(expression?), Box::new(rhs))),
                    Token::Or => Ok(FilterExpression::Or(Box::new(expression?), Box::new(rhs))),
                    _ => unreachable!(),
                }
            },
            // TODO: done here?
            _ => expression,
        }
    }

    fn expect_operator(&mut self) -> Result<Operator, ParseError> {
        match self.input.next() {
            Some((token, _)) => Operator::try_from(token.clone()).map_err(|_| ParseError::UnexpectedToken(token)),
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    fn expect_value(&mut self) -> Result<Value, ParseError> {
        match self.input.next() {
            Some((token, _)) => Value::try_from(token.clone()).map_err(|_| ParseError::UnexpectedToken(token)),
            None => Err(ParseError::UnexpectedEOF)
        }
    }

    fn expect_key_value_pairs(&mut self) -> Result<HashMap<String, Value>, ParseError> {
        let mut values = HashMap::new();

        let _ = self.expect_token_type(TokenKind::RBrace);

        loop {
            let peeked_next = self.input.peek();
            match peeked_next {
                Some((Token::LBrace, _)) => return Ok(values),
                Some((_, _)) => {
                    let pair = self.expect_key_value_pair()?;
                    values.insert(pair.0, pair.1);
                    if let Some((Token::Comma, _)) = self.input.peek() {
                        self.input.next();
                        continue;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        self.expect_token_type(TokenKind::LBrace)?;

        Ok(values)
    }

    fn expect_key_value_pair(&mut self) -> Result<(String, Value), ParseError> {
        let key = if let Token::Identifier(identifier) = self.expect_token_type(TokenKind::Identifier)? {
            identifier
        } else {
            unreachable!()
        };
        let _ = self.expect_token_type(TokenKind::Equals);
        let value = self.expect_value()?;

        Ok((key, value))
    }

    fn expect_token_type(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        match self.input.next() {
            Some((token, _)) if token.kind() == kind => Ok(token),
            Some((token, _)) => Err(ParseError::UnexpectedToken(token)),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}
