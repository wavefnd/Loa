use std::iter::Peekable;
use std::slice::Iter;
use lexer::{Token, TokenType};
use crate::ast::{Operator, Expression, FormatPart, Literal};

pub fn parse_format_string(s: &str) -> Vec<FormatPart> {
    let mut parts = Vec::new();
    let mut buffer = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            if let Some('}') = chars.peek() {
                chars.next();
                if !buffer.is_empty() {
                    parts.push(FormatPart::Literal(buffer.clone()));
                    buffer.clear();
                }
                parts.push(FormatPart::Placeholder);
            } else {
                buffer.push(c);
            }
        } else {
            buffer.push(c);
        }
    }

    if !buffer.is_empty() {
        parts.push(FormatPart::Literal(buffer));
    }

    parts
}

pub fn parse_expression<'a, T>(tokens: &mut Peekable<T>) -> Option<Expression>
where
    T: Iterator<Item = &'a Token>,
{
    let expr = parse_logical_expression(tokens)?;
    Some(expr)
}

pub fn parse_logical_expression<'a, T>(tokens: &mut Peekable<T>) -> Option<Expression>
where
    T: Iterator<Item = &'a Token>,
{
    let mut left = parse_relational_expression(tokens)?;

    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::LogicalAnd | TokenType::LogicalOr => {
                let op = match token.token_type {
                    TokenType::LogicalAnd => Operator::LogicalAnd,
                    TokenType::LogicalOr => Operator::LogicalOr,
                    _ => unreachable!(),
                };
                tokens.next();

                let right = parse_relational_expression(tokens)?;
                left = Expression::BinaryExpression {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }
    Some(left)
}

pub fn parse_relational_expression<'a, T>(tokens: &mut Peekable<T>) -> Option<Expression>
where
    T: Iterator<Item = &'a Token>,
{
    let mut left = parse_additive_expression(tokens)?;

    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::EqualTwo |
            TokenType::NotEqual |
            TokenType::Rchevr |
            TokenType::RchevrEq |
            TokenType::Lchevr |
            TokenType::LchevrEq => {
                let op = match token.token_type {
                    TokenType::EqualTwo => Operator::Equal,
                    TokenType::NotEqual => Operator::NotEqual,
                    TokenType::Rchevr => Operator::Greater,
                    TokenType::RchevrEq => Operator::GreaterEqual,
                    TokenType::Lchevr => Operator::Less,
                    TokenType::LchevrEq => Operator::LessEqual,
                    _ => unreachable!(),
                };
                tokens.next();

                let right = parse_additive_expression(tokens)?;
                left = Expression::BinaryExpression {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }
    Some(left)
}

pub fn parse_additive_expression<'a, T>(tokens: &mut Peekable<T>) -> Option<Expression>
where
    T: Iterator<Item = &'a Token>,
{
    let mut left = parse_multiplicative_expression(tokens)?;

    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::Plus | TokenType::Minus => {
                let op = match token.token_type {
                    TokenType::Plus => Operator::Add,
                    TokenType::Minus => Operator::Subtract,
                    _ => unreachable!(),
                };
                tokens.next();

                let right = parse_multiplicative_expression(tokens)?;
                left = Expression::BinaryExpression {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }
    Some(left)
}

pub fn parse_multiplicative_expression<'a, T>(tokens: &mut Peekable<T>) -> Option<Expression>
where
    T: Iterator<Item = &'a Token>,
{
    let mut left = parse_primary_expression(tokens)?;

    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::Star | TokenType::Div => {
                let op = match token.token_type {
                    TokenType::Star => Operator::Multiply,
                    TokenType::Div => Operator::Divide,
                    _ => unreachable!(),
                };
                tokens.next();

                let right = parse_primary_expression(tokens)?;
                left = Expression::BinaryExpression {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }
    Some(left)
}

pub fn parse_primary_expression<'a, T>(tokens: &mut Peekable<T>) -> Option<Expression>
where
    T: Iterator<Item = &'a Token>,
{
    let token = tokens.peek()?.clone();

    match &token.token_type {
        TokenType::Number(value) => {
            tokens.next();
            Some(Expression::Literal(Literal::Number(*value)))
        }
        TokenType::Float(value) => {
            tokens.next();
            Some(Expression::Literal(Literal::Float(*value)))
        }
        TokenType::Identifier(name) => {
            let name = name.clone();
            tokens.next(); // consume identifier

            if let Some(Token { token_type: TokenType::Lparen, .. }) = tokens.peek() {
                tokens.next(); // consume '('

                let mut args = vec![];
                while let Some(token) = tokens.peek() {
                    if token.token_type == TokenType::Rparen {
                        tokens.next(); // consume ')'
                        break;
                    }

                    let arg = parse_expression(tokens)?;
                    args.push(arg);

                    if let Some(Token { token_type: TokenType::Comma, .. }) = tokens.peek() {
                        tokens.next(); // consume ','
                    }
                }

                Some(Expression::FunctionCall { name, args })
            } else {
                Some(Expression::Variable(name))
            }
        }
        TokenType::Lparen => {
            parse_parenthesized_expression(tokens).map(|expr| Expression::Grouped(Box::new(expr)))
        }
        TokenType::String(value) => {
            tokens.next(); // consume the string token
            Some(Expression::Literal(Literal::String(value.clone())))
        }
        _ => {
            println!("Error: Expected primary expression, found {:?}", token.token_type);
            None
        }
    }
}

pub fn parse_parenthesized_expression<'a, T>(tokens: &mut Peekable<T>) -> Option<Expression>
where
    T: Iterator<Item = &'a Token>,
{
    // Ensure the next token is '('
    if tokens.next()?.token_type != TokenType::Lparen {
        println!("Error: Expected '('");
        return None;
    }

    // Parse the inner expression
    let expr = parse_expression(tokens)?;

    // Ensure the next token is ')'
    if tokens.next()?.token_type != TokenType::Rparen {
        println!("Error: Expected ')'");
        return None;
    }

    Some(expr)
}

pub fn parse_expression_from_token(first_token: &Token, tokens: &mut Peekable<Iter<Token>>) -> Option<Expression> {
    match &first_token.token_type {
        TokenType::Identifier(name) => Some(Expression::Variable(name.clone())),

        _ => None,
    }
}