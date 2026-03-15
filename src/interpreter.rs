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
    Function(String, Vec<(Token, Token)>, Box<Stmt>),
}

enum ControlFlow {
    Return(Value),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(x) => write!(f, "{x}"),
            Value::Int(x) => write!(f, "{x}"),
            Value::Float(x) => write!(f, "{x}"),
            Value::Str(x) => write!(f, "{x}"),
            Value::Function(name, ..) => write!(f, "<fn {name}>"),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(x) => *x,
            _ => panic!("expected bool expression"),
        }
    }
}

pub struct Interpreter {
    stmts: Vec<Stmt>,
    globals: Environment,
}

impl Interpreter {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Interpreter {
            stmts: stmts,
            globals: Environment::new(),
        }
    }

    pub fn interpret(&mut self) {
        // We do this so compiler won't complain about E0502
        // (borrowing as mutable and immutable)
        let stmts = mem::take(&mut self.stmts);
        let mut globals = mem::take(&mut self.globals);
        for stmt in &stmts {
            let _ = self.execute(stmt, &mut globals);
        }
        self.stmts = stmts;
        self.globals = globals;
    }

    fn execute(&mut self, stmt: &Stmt, environ: &mut Environment) -> Result<(), ControlFlow> {
        match stmt {
            Stmt::Return(expr) => {
                let value = match expr {
                    Some(e) => self.eval(e, environ),
                    None => Value::Bool(false),
                };
                return Err(ControlFlow::Return(value));
            },
            Stmt::Expression(e) => {
                if let Expr::Assignment(expr1, expr2) = e {
                    self.execute_assignment(expr1, expr2, environ)
                } else {
                    self.eval(e, environ);
                    return Ok(());
                }
            },
            Stmt::Let(name, typ, expr) => self.execute_let(name, typ, expr, environ),
            Stmt::Print(expr, newline) => self.execute_print(expr, newline, environ),
            Stmt::Block(stmts) => self.execute_block(stmts, environ),
            Stmt::While(cond, block) => self.execute_while(cond, block, environ),
            Stmt::If(cond, then_branch, else_branch) => self.execute_if(cond, then_branch, else_branch, environ),
            Stmt::Func(name, params, ret_type, body) => self.execute_func(name, params, ret_type, body),
        }
    }

    fn eval(&mut self, expression: &Expr, environ: &mut Environment) -> Value {
        match expression {
            Expr::Literal(v) => self.eval_literal(v),
            Expr::Unary(v, op) => self.eval_unary(v, op, environ),
            Expr::Binary(left, op, right) => self.eval_binary(left, op, right, environ),
            Expr::Grouping(exp) => self.eval(exp, environ),
            Expr::Variable(sym) => self.eval_variable(sym, environ),
            Expr::Logical(left, op, right) => self.eval_logical(left, op, right, environ),
            Expr::FuncCall(name, args) => self.eval_func_call(name, args, environ),
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

    fn eval_unary(&mut self, v: &Token, e: &Expr, environ: &mut Environment) -> Value {
        let value = self.eval(e, environ);
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

    fn eval_binary(&mut self, left: &Expr, op: &Token, right: &Expr, environ: &mut Environment) -> Value {
        let left_value = self.eval(left, environ);
        let right_value = self.eval(right, environ);
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
                    Token::Less => Value::Bool(lv < rv),
                    Token::LessEq => Value::Bool(lv <= rv),
                    Token::Greater => Value::Bool(lv > rv),
                    Token::GreaterEq => Value::Bool(lv >= rv),
                    Token::Equals => Value::Bool(lv == rv),
                    Token::NotEq => Value::Bool(lv != rv),
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
                    Token::Less => Value::Bool(lv < rv),
                    Token::LessEq => Value::Bool(lv <= rv),
                    Token::Greater => Value::Bool(lv > rv),
                    Token::GreaterEq => Value::Bool(lv >= rv),
                    Token::Equals => Value::Bool(lv == rv),
                    Token::NotEq => Value::Bool(lv != rv),
                    _ => panic!("unknown operator {:#?}", op),
                }
            }
            (Value::Str(lv), Value::Str(rv)) => {
                match op {
                    Token::Plus => Value::Str(lv + &rv),
                    Token::Equals => Value::Bool(lv == rv),
                    Token::NotEq => Value::Bool(lv != rv),
                    _ => panic!("operator {:#?} not valid for string", op),
                }
            }
            _ => panic!("invalid types to operate on"),
        }
    }

    fn eval_logical(&mut self, left: &Expr, op: &Token, right: &Expr, environ: &mut Environment) -> Value {
        let left_value = self.eval(left, environ);
        let right_value = self.eval(right, environ);
        match (left_value, right_value) {
            (Value::Bool(lv), Value::Bool(rv)) => {
                match op {
                    Token::Or => Value::Bool(lv || rv),
                    Token::And => Value::Bool(lv && rv),
                    _ => panic!("operator {:#?} not valid for logical expression", op),
                }
            }
            _ => panic!("expected two boolean expressions to evaluate"),
        }
    }

    fn eval_func_call(&mut self, name: &Expr, args: &Vec<Expr>, environ: &mut Environment) -> Value {
        let mut env = Environment::from(environ);
        let name_string = match name {
            Expr::Variable(s) => {
                match s {
                    Token::Symbol(sym) => sym.clone(),
                    _ => panic!("expected symbol for function name"),
                }
            },
            _ => panic!("expected variable for function name"),
        };

        let func = self.globals.get(name_string.clone());
        if let Value::Function(_, params, body) = func {
            let arity = params.len();
            if args.len() != arity {
                panic!("expected {} arguments, got {}", arity, args.len());
            }

            let mut arguments: Vec<Value> = Vec::new();
            for arg in args {
                arguments.push(self.eval(arg, &mut env));
            }

            for (param, arg) in params.iter().zip(arguments.into_iter()) {
                if let Token::Symbol(name) = &param.0 {
                    env.insert(name.clone(), arg);
                }
            }

            // TODO: use internal globals and bindings for functions
            match self.execute(&body, &mut env) {
                Err(ControlFlow::Return(v)) => v,
                Ok(_) => Value::Bool(false),
            }
        } else {
            panic!("{name_string} is not a function");
        }
    }

    fn eval_variable(&mut self, sym_tk: &Token, environ: &mut Environment) -> Value {
        let name = match sym_tk {
            Token::Symbol(s) => s.clone(),
            _ => panic!("expected symbol"),
        };
        environ.get(name)
    }

    fn execute_assignment(&mut self, expr1: &Expr, expr2: &Expr, environ: &mut Environment) -> Result<(), ControlFlow> {
        let rhs = self.eval(expr2, environ);
        if let Expr::Variable(name_tk) = expr1 {
            let name = match name_tk {
                Token::Symbol(s) => s.clone(),
                _ => panic!("expected symbol"),
            };
            environ.update(name, rhs);
            Ok(())
        } else {
            panic!("expected variable on lhs");
        }
    }

    // TODO: implement type checking
    fn execute_let(&mut self, name_tk: &Token, _typ_tk: &Token, expr: &Expr, environ: &mut Environment) -> Result<(), ControlFlow> {
        let name = match name_tk {
            Token::Symbol(s) => s.clone(),
            _ => panic!("expected symbol"),
        };
        let val = self.eval(expr, environ);
        environ.insert(name, val);
        Ok(())
    }

    fn execute_print(&mut self, expr: &Expr, newline: &bool, environ: &mut Environment) -> Result<(), ControlFlow> {
        let value = self.eval(expr, environ);
        if *newline {
            println!("{}", value);
        } else {
            print!("{}", value);
        }
        Ok(())
    }

    fn execute_block(&mut self, block: &Vec<Stmt>, environ: &mut Environment) -> Result<(), ControlFlow> {
        for stmt in block {
            self.execute(stmt, environ)?;
        }
        Ok(())
    }

    fn execute_while(&mut self, cond: &Expr, block: &Stmt, environ: &mut Environment) -> Result<(), ControlFlow> {
        while self.eval(cond, environ).is_truthy() {
            self.execute(block, environ)?;
        }
        Ok(())
    }

    fn execute_if(&mut self, cond: &Expr, then_branch: &Stmt, else_branch: &Option<Box<Stmt>>, environ: &mut Environment) -> Result<(), ControlFlow> {
        if self.eval(cond, environ).is_truthy() {
            self.execute(then_branch, environ)?;
        } else if let Some(els) = else_branch {
            self.execute(els, environ)?;
        }
        Ok(())
    }

    fn execute_func(&mut self, name: &Token, params: &Vec<(Token, Token)>, _ret_type: &Token, body: &Stmt) -> Result<(), ControlFlow> {
        let name_str = match name {
            Token::Symbol(s) => s.clone(),
            _ => panic!("expected symbol for function name"),
        };
        let func = Value::Function(name_str.clone(), params.clone(), Box::new(body.clone()));
        self.globals.insert(name_str, func);
        Ok(())
    }
}
