use std::collections::HashMap;
use parser::ast::*;
use ::error::{LoaError, LoaErrorKind};

pub struct Interpreter {
    pub variables: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
        }
    }

    pub fn execute(&mut self, ast: &[ASTNode]) {
        for node in ast {
            self.execute_node(node);
        }
    }

    fn execute_node(&mut self, node: &ASTNode) {
        match node {
            ASTNode::Statement(stmt) => self.execute_statement(stmt),
            _ => {}
        }
    }

    fn execute_statement(&mut self, stmt: &StatementNode) {
        match stmt {
            StatementNode::PrintArgs(args) => {
                for expr in args {
                    let value = self.evaluate_expression(expr);
                    match value {
                        Value::Number(n) => println!("{}", n),
                        Value::Float(f) => println!("{}", f),
                        Value::String(s) => println!("{}", s),
                        Value::Bool(b) => println!("{}", b),
                        Value::None => println!("None"),
                    }
                }
            }
            StatementNode::Assign { variable, value } => {
                let val = self.evaluate_expression(value);
                self.variables.insert(variable.clone(), val);
            }
            StatementNode::While { condition, body } => {
                while let Value::Bool(true) = self.evaluate_expression(condition) {
                    self.execute(body);
                }
            }
            StatementNode::If { condition, body, else_if_blocks, else_block } => {
                if self.evaluate_condition(condition) {
                    self.execute(body);
                } else if let Some(else_ifs) = else_if_blocks {
                    let mut executed = false;

                    for else_if in else_ifs.iter() {
                        if let ASTNode::Statement(StatementNode::If { condition: else_if_condition, body, else_if_blocks: _, else_block: inner_else_block }) = else_if {
                            if self.evaluate_condition(else_if_condition) {
                                self.execute(body);
                                executed = true;
                                break;
                            } else if let Some(inner_else_block) = inner_else_block {
                                let warning = LoaError::new(
                                    LoaErrorKind::SyntaxError("Unused else block".to_string()),
                                    "Warning: An else block inside an else-if was ignored",
                                    "unknown",
                                    0,
                                    0,
                                );
                                warning.display();
                            }
                        }
                    }

                    if !executed {
                        if let Some(else_if) = else_ifs.first() {
                            if let ASTNode::Statement(StatementNode::If { else_block: Some(inner_else_block), .. }) = else_if {
                                self.execute(inner_else_block);
                            }
                        }
                    }
                } else if let Some(else_block) = else_block {
                    self.execute(else_block);
                }
            }
            StatementNode::Break => {}
            StatementNode::Continue => {}
            StatementNode::Return(_) => {}
            _ => {}
        }
    }

    fn evaluate_condition(&mut self, expr: &Expression) -> bool {
        match self.evaluate_expression(expr) {
            Value::Bool(b) => b,
            Value::Number(n) => n != 0,
            _ => false,
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Number(n) => Value::Number(*n),
                Literal::Float(f) => Value::Float(*f),
                Literal::String(s) => Value::String(s.clone()),
            },
            Expression::Variable(name) => {
                self.variables.get(name).cloned().unwrap_or(Value::None)
            }
            Expression::BinaryExpression { left, operator, right } => {
                let l = self.evaluate_expression(left);
                let r = self.evaluate_expression(right);
                self.evaluate_binary_op(l, operator, r)
            }
            _ => Value::None,
        }
    }

    fn evaluate_binary_op(&self, l: Value, op: &Operator, r: Value) -> Value {
        match (l, r) {
            (Value::Number(a), Value::Number(b)) => match op {
                Operator::Add => Value::Number(a + b),
                Operator::Subtract => Value::Number(a - b),
                Operator::Multiply => Value::Number(a * b),
                Operator::Divide => Value::Number(a / b),
                Operator::Less => Value::Bool(a < b),
                Operator::Greater => Value::Bool(a > b),
                Operator::Equal => Value::Bool(a == b),
                Operator::NotEqual => Value::Bool(a != b),
                _ => Value::None,
            },
            _ => Value::None,
        }
    }
}
