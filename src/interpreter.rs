use crate::ast::Stmt;
use crate::ast::Expr;
use crate::token::Token;

#[derive(Debug)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

pub struct Interpreter {
    stmts: Vec<Stmt>
}

impl Interpreter {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Interpreter {
            stmts: stmts,
        }
    }

    pub fn interpret(&self) {
        for stmt in &self.stmts {
            self.execute(stmt);
        }
    }

    fn execute(&self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression(e) => {
                let v = self.eval(e);
                println!("{:#?}", v);
            },
            _ => todo!(),
        }
    }

    fn eval(&self, expression: &Expr) -> Value {
        match expression {
            Expr::Literal(v) => self.eval_literal(v),
            Expr::Unary(v, op) => self.eval_unary(v, op),
            Expr::Binary(left, op, right) => self.eval_binary(left, op, right),
            Expr::Grouping(exp) => self.eval(exp),
            _ => todo!()
        }
    }

    fn eval_literal(&self, v: &Token) -> Value {
        match v {
            Token::False => Value::Bool(false),
            Token::True => Value::Bool(true),
            Token::Number(n) => self.eval_number_literal(n),
            Token::Str(s) => self.eval_string_literal(s),
            _ => todo!(),
        }
    }

    fn eval_number_literal(&self, nstring: &String) -> Value {
        let s = nstring.as_str();
        if let Ok(x) = s.parse::<i64>() {
            Value::Int(x)
        } else if let Ok(x) = s.parse::<f64>() {
            Value::Float(x)
        } else {
            panic!("invalid number literal: {}", nstring);
        }
    }

    fn eval_string_literal(&self, string: &String) -> Value {
        let s = &string[1..string.len()-1]; // Remove trailing "
        Value::Str(s.to_string())
    }

    fn eval_unary(&self, v: &Token, e: &Expr) -> Value {
        let value = self.eval(e);
        match v {
            Token::Minus => {
                if let Value::Int(i) = value {
                    return Value::Int(-i);
                } else if let Value::Float(f) = value {
                    return Value::Float(-f);
                } else {
                    panic!("invalid number type to negate: {:#?}", value);
                }
            }
            Token::Not => {
                if let Value::Bool(x) = value {
                    return Value::Bool(!x);
                } else {
                    panic!("invalid bool type to negate: {:#?}", value);
                }
            }
            _ => panic!("not an unary expression: {:#?}", v)
        }
    }

    fn eval_binary(&self, left: &Expr, op: &Token, right: &Expr) -> Value {
        let left_value = self.eval(left);
        let right_value = self.eval(right);
        self.perform_op(left_value, op, right_value)
    }

    fn perform_op(&self, left: Value, op: &Token, right: Value) -> Value {
        match (left, right) {
            (Value::Int(lv), Value::Int(rv)) => {
                match op {
                    Token::Plus => Value::Int(lv + rv),
                    Token::Minus => Value::Int(lv - rv),
                    Token::Mul => Value::Int(lv*rv),
                    Token::Div => Value::Int(lv/rv),
                    Token::Modulo => Value::Int(lv%rv),
                    _ => panic!("unknown operator {:#?}", op),
                }
            }
            (Value::Float(lv), Value::Float(rv)) => {
                match op {
                    Token::Plus => Value::Float(lv + rv),
                    Token::Minus => Value::Float(lv - rv),
                    Token::Mul => Value::Float(lv*rv),
                    Token::Div => Value::Float(lv/rv),
                    Token::Modulo => Value::Float(lv%rv),
                    _ => panic!("unknown operator {:#?}", op),
                }
            }
            (Value::Str(lv), Value::Str(rv)) => {
                match op {
                    Token::Plus => Value::Str(lv + &rv),
                    _ => panic!("operator {:#?} not valid for string", op),
                }
            }
            _ => panic!("invalid types to operate on"),
        }
    }
}
