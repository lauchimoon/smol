use crate::ast::Stmt;
use crate::ast::Expr;
use crate::token::Token;
use crate::environ::Environment;
use std::mem;
use std::fmt;

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(x) => write!(f, "{x}"),
            Value::Int(x) => write!(f, "{x}"),
            Value::Float(x) => write!(f, "{x}"),
            Value::Str(x) => write!(f, "{x}"),
        }
    }
}

pub struct Interpreter {
    stmts: Vec<Stmt>,
    environment: Environment,
}

impl Interpreter {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Interpreter {
            stmts: stmts,
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self) {
        // We do this so compiler won't complain about E0502
        // (borrowing as mutable and immutable)
        let stmts = mem::take(&mut self.stmts);
        for stmt in &stmts {
            self.execute(stmt);
        }
        self.stmts = stmts;
    }

    fn execute(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression(e) => {
                if let Expr::Assignment(expr1, expr2) = e {
                    self.execute_assignment(expr1, expr2);
                }
            },
            Stmt::Let(name, typ, expr) => self.execute_let(name, typ, expr),
            Stmt::Print(expr, newline) => self.execute_print(expr, newline),
            Stmt::Block(stmts) => self.execute_block(stmts),
            Stmt::While(cond, block) => self.execute_while(cond, block),
            _ => todo!(),
        }
    }

    fn eval(&mut self, expression: &Expr) -> Value {
        match expression {
            Expr::Literal(v) => self.eval_literal(v),
            Expr::Unary(v, op) => self.eval_unary(v, op),
            Expr::Binary(left, op, right) => self.eval_binary(left, op, right),
            Expr::Grouping(exp) => self.eval(exp),
            Expr::Variable(sym) => self.eval_variable(sym),
            _ => todo!()
        }
    }

    fn eval_literal(&mut self, v: &Token) -> Value {
        match v {
            Token::False => Value::Bool(false),
            Token::True => Value::Bool(true),
            Token::Number(n) => self.eval_number_literal(n),
            Token::Str(s) => self.eval_string_literal(s),
            _ => todo!(),
        }
    }

    fn eval_number_literal(&mut self, nstring: &String) -> Value {
        let s = nstring.as_str();
        if let Ok(x) = s.parse::<i64>() {
            Value::Int(x)
        } else if let Ok(x) = s.parse::<f64>() {
            Value::Float(x)
        } else {
            panic!("invalid number literal: {}", nstring);
        }
    }

    fn eval_string_literal(&mut self, string: &String) -> Value {
        let s = &string[1..string.len()-1]; // Remove trailing "
        Value::Str(s.to_string())
    }

    fn eval_unary(&mut self, v: &Token, e: &Expr) -> Value {
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

    fn eval_binary(&mut self, left: &Expr, op: &Token, right: &Expr) -> Value {
        let left_value = self.eval(left);
        let right_value = self.eval(right);
        self.perform_op(left_value, op, right_value)
    }

    fn perform_op(&mut self, left: Value, op: &Token, right: Value) -> Value {
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

    fn eval_variable(&mut self, sym_tk: &Token) -> Value {
        let name = match sym_tk {
            Token::Symbol(s) => s.clone(),
            _ => panic!("expected symbol"),
        };
        self.environment.get(name)
    }

    fn execute_assignment(&mut self, expr1: &Expr, expr2: &Expr) {
        let rhs = self.eval(expr2);
        if let Expr::Variable(name_tk) = expr1 {
            let name = match name_tk {
                Token::Symbol(s) => s.clone(),
                _ => panic!("expected symbol"),
            };
            self.environment.update(name, rhs);
        } else {
            panic!("expected variable on lhs");
        }
    }

    // TODO: implement type checking
    fn execute_let(&mut self, name_tk: &Token, _typ_tk: &Token, expr: &Expr) {
        let name = match name_tk {
            Token::Symbol(s) => s.clone(),
            _ => panic!("expected symbol"),
        };
        let val = self.eval(expr);

        // This function panics when name is already defined
        // To redefine, we'd have a Environment::update function
        self.environment.insert(name, val);
    }

    fn execute_print(&mut self, expr: &Expr, newline: &bool) {
        let value = self.eval(expr);
        if *newline {
            println!("{}", value);
        } else {
            print!("{}", value);
        }
    }

    fn execute_block(&mut self, block: &Vec<Stmt>) {
        for stmt in block {
            self.execute(stmt);
        }
    }

    fn execute_while(&mut self, cond: &Expr, block_stmt: &Stmt) {
        let value = match self.eval(cond) {
            Value::Bool(x) => x,
            _ => panic!("expected boolean expression for while condition"),
        };

        let block = match block_stmt {
            Stmt::Block(stmts) => stmts,
            _ => panic!("expected one or more statements inside while body"),
        };
        while value {
            self.execute_block(block);
        }
    }
}
