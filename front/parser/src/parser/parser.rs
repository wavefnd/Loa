use std::collections::HashSet;
use std::iter::Peekable;
use std::slice::Iter;
use ::lexer::*;
use error::{LoaError, LoaErrorKind};
use parser::ast::*;
use crate::*;
use crate::parser::format::*;

pub fn parse(tokens: &Vec<Token>) -> Option<Vec<ASTNode>> {
    let mut iter = tokens.iter().peekable();
    let mut nodes = vec![];

    while let Some(token) = iter.peek() {
        if token.token_type == TokenType::Eof {
            break;
        }

        if let Some(node) = parse_statement(&mut iter) {
            nodes.push(node);
        } else {
            println!("❌ Failed to parse statement");
            return None;
        }
    }

    Some(nodes)
}

pub fn param(parameter: String, initial_value: Option<Value>) -> ParameterNode {
    ParameterNode {
        name: parameter,
        initial_value,
    }
}

pub fn parse_parameters(tokens: &mut Peekable<Iter<Token>>) -> Vec<ParameterNode> {
    let mut params = vec![];

    loop {
        let Some(token) = tokens.peek() else {
            break;
        };

        match &token.token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                tokens.next(); // consume identifier

                if !matches!(tokens.peek().map(|t| &t.token_type), Some(TokenType::Colon)) {
                    println!("Error: Expected ':' after parameter name '{}'", name);
                    break;
                }
                tokens.next(); // consume ':'

                let initial_value = if matches!(tokens.peek().map(|t| &t.token_type), Some(TokenType::Equal)) {
                    tokens.next(); // consume '='
                    match tokens.next() {
                        Some(Token { token_type: TokenType::Number(n), .. }) => Some(Value::Int(*n)),
                        Some(Token { token_type: TokenType::Float(f), .. }) => Some(Value::Float(*f)),
                        Some(Token { token_type: TokenType::String(s), .. }) => Some(Value::Text(s.clone())),
                        _ => None,
                    }
                } else {
                    None
                };

                params.push(ParameterNode {
                    name,
                    initial_value,
                });

                match tokens.peek().map(|t| &t.token_type) {
                    Some(TokenType::SemiColon) => {
                        tokens.next(); // consume ';'
                        continue;
                    }
                    Some(TokenType::Rparen) => {
                        tokens.next();
                        break;
                    }
                    Some(TokenType::Comma) => {
                        println!("Error: use `;` instead of `,` to separate parameters");
                        break;
                    }
                    _ => break,
                }
            }

            TokenType::Rparen => {
                tokens.next();
                break;
            }

            _ => break,
        }
    }

    params
}

pub fn extract_body(tokens: &mut Peekable<Iter<Token>>) -> Option<Vec<ASTNode>> {
    let mut body = vec![];

    if tokens.peek()?.token_type != TokenType::Colon {
        println!("Error: Expected ':' before function body");
        return None;
    }
    tokens.next(); // consume '{'

    if tokens.peek()?.token_type != TokenType::Indent {
        println!("Error: Expected Indent after ':' for function body");
        return None;
    }

    while let Some(token) = tokens.peek() {
        match &token.token_type {
            TokenType::Dedent => {
                tokens.next(); // consume Dedent
                break;
            }
            TokenType::Eof => {
                println!("Error: Unexpected EOF inside function body");
                return None;
            }
            _ => {
                if let Some(node) = parse_statement(tokens) {
                    body.push(node);
                } else {
                    println!("Error: Failed to parse statement inside function body");
                    return None;
                }
            }
        }
    }

    Some(body)
}

pub fn parse_function_call(name: Option<String>, tokens: &mut Peekable<Iter<Token>>) -> Option<Expression> {
    let name = name?;

    if tokens.peek()?.token_type != TokenType::Lparen {
        println!("❌ Expected '(' after function name '{}'", name);
        return None;
    }
    tokens.next(); // consume '('

    let mut args = vec![];

    while let Some(token) = tokens.peek() {
        if token.token_type == TokenType::Rparen {
            tokens.next(); // consume ')'
            break;
        }

        let arg = parse_expression(tokens)?;
        args.push(arg);

        match tokens.peek().map(|t| &t.token_type) {
            Some(TokenType::Comma) => {
                tokens.next(); // consume ','
            }
            Some(TokenType::Rparen) => continue,
            _ => {
                println!("❌ Unexpected token in function arguments: {:?}", tokens.peek());
                return None;
            }
        }
    }

    Some(Expression::FunctionCall {
        name,
        args,
    })
}

fn parse_parentheses(tokens: &mut Peekable<Iter<Token>>) -> Vec<Token> {
    let mut param_tokens = vec![];
    let mut paren_depth = 1;

    while let Some(token) = tokens.next() {
        match token.token_type {
            TokenType::Lparen => paren_depth += 1,
            TokenType::Rparen => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        param_tokens.push(token.clone());
    }
    param_tokens
}

// FUN parsing
fn parse_function(tokens: &mut Peekable<Iter<Token>>) -> Option<ASTNode> {
    tokens.next(); // consume 'fun'

    let name = match tokens.next() {
        Some(Token { token_type: TokenType::Identifier(name), .. }) => name.clone(),
        _ => return None,
    };

    if tokens.peek()?.token_type != TokenType::Lparen {
        println!("Error: Expected '(' after function name '{}'", name);
        return None;
    }
    tokens.next(); // consume '('

    let parameters = parse_parameters(tokens);

    let mut param_names = HashSet::new();
    for param in &parameters {
        if !param_names.insert(param.name.clone()) {
            println!("Error: Parameter '{}' is declared multiple times", param.name);
            return None;
        }
    }

    if tokens.peek()?.token_type != TokenType::Colon {
        println!("Error: Expected ':' after function parameters");
        return None;
    }
    tokens.next(); // consume ':'

    let body = extract_body(tokens)?;

    Some(ASTNode::Function(FunctionNode {
        name,
        parameters,
        body,
    }))
}

// VAR parsing
fn parse_var(tokens: &mut Peekable<Iter<'_, Token>>) -> Option<ASTNode> {
    let name = match tokens.next() {
        Some(Token { token_type: TokenType::Identifier(name), .. }) => name.clone(),
        _ => {
            println!("Expected identifier after 'var'");
            return None;
        }
    };

    if tokens.peek()?.token_type != TokenType::Equal {
        println!("Expected '=' after variable name '{}'", name);
        return None;
    }
    tokens.next(); // consume '='

    let initial_value = parse_expression(tokens)?;

    if let Some(Token { token_type: TokenType::SemiColon, .. }) = tokens.peek() {
        tokens.next(); // consume ';'
    }

    Some(ASTNode::Statement(StatementNode::Assign {
        variable: name,
        value: initial_value,
    }))
}

// PRINT parsing
fn parse_print(tokens: &mut Peekable<Iter<Token>>) -> Option<ASTNode> {
    if tokens.peek()?.token_type != TokenType::Lparen {
        println!("Error: Expected '(' after 'print'");
        return None;
    }
    tokens.next(); // consume '('

    let mut args = Vec::new();

    while let Some(token) = tokens.peek() {
        if token.token_type == TokenType::Rparen {
            tokens.next(); // consume ')'
            break;
        }

        if let Some(expr) = parse_expression(tokens) {
            args.push(expr);
        } else {
            println!("Error: Failed to parse expression in 'print'");
            return None;
        }

        if let Some(Token { token_type: TokenType::Comma, .. }) = tokens.peek() {
            tokens.next(); // consume ','
        }
    }

    Some(ASTNode::Statement(StatementNode::PrintArgs(args)))
}

fn skip_whitespace(tokens: &mut Peekable<Iter<Token>>) {
    while let Some(token) = tokens.peek() {
        if token.token_type == TokenType::Whitespace {
            tokens.next();
        } else {
            break;
        }
    }
}

// IF parsing
fn parse_if(tokens: &mut Peekable<Iter<Token>>) -> Option<ASTNode> {
    if tokens.peek()?.token_type != TokenType::Lparen {
        let token = tokens.peek().unwrap();
        LoaError::new(
            LoaErrorKind::ExpectedToken("(".to_string()),
            "Expected '(' after 'if'".to_string(),
            "unknown",
            token.line,
            0,
        ).display();
        return None;
    }
    tokens.next(); // Consume '('

    let condition = parse_expression(tokens)?;

    if tokens.peek()?.token_type != TokenType::Rparen {
        println!("Error: Expected ')' after 'if' condition");
        return None;
    }
    tokens.next(); // Consume ')'

    if tokens.peek()?.token_type != TokenType::Colon {
        println!("Error: Expected ':' after 'if' condition");
        return None;
    }
    tokens.next(); // Consume ':'

    let body = parse_block(tokens)?;

    let mut else_if_blocks: Vec<ASTNode> = Vec::new();
    let mut else_block = None;

    while let Some(token) = tokens.peek() {
        if token.token_type != TokenType::Else {
            break;
        }
        tokens.next(); // Consume 'else'

        if let Some(Token { token_type: TokenType::If, .. }) = tokens.peek() {
            tokens.next(); // consume 'if'
            let parsed = parse_if(tokens);

            match parsed {
                Some(ASTNode::Statement(stmt @ StatementNode::If { .. })) => {
                    else_if_blocks.push(ASTNode::Statement(stmt));
                }
                Some(_) | None => {
                    return None;
                }
            }
        } else {
            if tokens.peek()?.token_type != TokenType::Colon {
                println!("Error: Expected ':' after 'else'");
                return None;
            }
            tokens.next(); // Consume ':'
            else_block = Some(Box::new(parse_block(tokens)?));
            break;
        }
    }

    Some(ASTNode::Statement(StatementNode::If {
        condition,
        body,
        else_if_blocks: if else_if_blocks.is_empty() {
            None
        } else {
            Some(Box::new(else_if_blocks))
        },
        else_block,
    }))
}

// FOR parsing
fn parse_for(tokens: &mut Peekable<Iter<Token>>) -> Option<ASTNode> {
    /*
    // Check 'for' keyword and see if there is '()
    if tokens.peek()?.token_type != TokenType::Lparen {
        println!("Error: Expected '(' after 'if'");
        return None;
    }
    tokens.next(); // '(' Consumption

    // Conditional parsing (where condition must be made ASTNode)
    let initialization = parse_expression(tokens)?; // Parsing conditions with expressions
    let condition = parse_expression(tokens)?;
    let increment = parse_expression(tokens)?;
    let body = parse_expression(tokens)?;

    if tokens.peek()?.token_type != TokenType::Rparen {
        println!("Error: Expected ')' after condition");
        return None;
    }
    tokens.next(); // ')' Consumption

    Some(ASTNode::Statement(StatementNode::For {
        initialization,
        condition,
        increment,
        body,
    }))
     */
    None
}

// WHILE parsing
fn parse_while(tokens: &mut Peekable<Iter<Token>>) -> Option<ASTNode> {
    if tokens.peek()?.token_type != TokenType::Lparen {
        println!("Error: Expected '(' after 'while'");
        return None;
    }
    tokens.next(); // consume '('

    let condition = parse_expression(tokens)?;

    if tokens.peek()?.token_type != TokenType::Rparen {
        println!("Error: Expected ')' after 'while' condition");
        return None;
    }
    tokens.next(); // consume ')'

    if tokens.peek()?.token_type != TokenType::Colon {
        println!("Error: Expected ':' after 'while' condition");
        return None;
    }
    tokens.next(); // consume ':'

    let body = parse_block(tokens)?;

    Some(ASTNode::Statement(StatementNode::While { condition, body }))
}

fn parse_return(tokens: &mut Peekable<Iter<Token>>) -> Option<ASTNode> {
    let expr = if let Some(Token { token_type: TokenType::SemiColon, .. }) = tokens.peek() {
        tokens.next(); // consume ';'
        None
    } else {
        let value = parse_expression(tokens)?;
        if let Some(Token { token_type: TokenType::SemiColon, .. }) = tokens.peek() {
            tokens.next(); // consume ';'
        }
        Some(value)
    };

    Some(ASTNode::Statement(StatementNode::Return(expr)))
}

fn parse_assignment(tokens: &mut Peekable<Iter<Token>>, first_token: &Token) -> Option<ASTNode> {
    let left_expr = parse_expression_from_token(first_token, tokens)?;

    if tokens.peek()?.token_type != TokenType::Equal {
        println!("Error: Expected '=' in assignment");
        return None;
    }
    tokens.next(); // consume '='

    let right_expr = parse_expression(tokens)?;

    if let Expression::Variable(name) = left_expr {
        if let Some(Token { token_type: TokenType::SemiColon, .. }) = tokens.peek() {
            tokens.next(); // consume ';'
        }
        return Some(ASTNode::Statement(StatementNode::Assign {
            variable: name,
            value: right_expr,
        }));
    }

    println!("Error: Left side of assignment must be a variable");
    None
}

// block parsing
fn parse_block(tokens: &mut Peekable<Iter<Token>>) -> Option<Vec<ASTNode>> {
    let mut body = vec![];

    if tokens.peek()?.token_type != TokenType::Indent {
        println!("Error: Expected Indent to start a block");
        return None;
    }
    tokens.next(); // consume Indent

    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::Dedent => {
                tokens.next(); // consume Dedent
                break;
            }
            TokenType::Eof => {
                println!("Error: Unexpected EOF inside block");
                return None;
            }
            _ => {
                if let Some(node) = parse_statement(tokens) {
                    body.push(node);
                } else {
                    println!("Error: Failed to parse statement inside block");
                    return None;
                }
            }
        }
    }

    Some(body)
}

fn parse_statement(tokens: &mut Peekable<Iter<Token>>) -> Option<ASTNode> {
    let token = tokens.peek()?.clone();

    match token.token_type {
        TokenType::Print => {
            tokens.next(); // consume 'print'
            parse_print(tokens)
        }
        TokenType::If => {
            tokens.next(); // consume 'if'
            parse_if(tokens)
        }
        TokenType::While => {
            tokens.next(); // consume 'while'
            parse_while(tokens)
        }
        TokenType::For => {
            tokens.next(); // consume 'for'
            parse_for(tokens)
        }
        TokenType::Return => {
            tokens.next(); // consume 'return'
            parse_return(tokens)
        }
        TokenType::Break => {
            tokens.next(); // consume 'break'
            Some(ASTNode::Statement(StatementNode::Break))
        }
        TokenType::Continue => {
            tokens.next(); // consume 'continue'
            Some(ASTNode::Statement(StatementNode::Continue))
        }
        TokenType::Identifier(_) => {
            let first = tokens.next()?; // consume identifier
            parse_assignment(tokens, first)
        }
        _ => {
            println!("Error: Unknown token in block: {:?}", token);
            None
        }
    }
}
