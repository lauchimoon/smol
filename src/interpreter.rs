use crate::ast::Stmt;
use crate::ast::Expr;
use crate::token::Token;
use crate::token::TokenKind;
use crate::environ::Environment;
use crate::format;
use std::mem;
use std::fmt;
use std::process;

#[derive(Clone, Debug)]
pub enum Value {
    Nil,
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
            Value::Nil => write!(f, "nil"),
            Value::Bool(x) => write!(f, "{x}"),
            Value::Int(x) => write!(f, "{x}"),
            Value::Float(x) => write!(f, "{x}"),
            Value::Str(x) => write!(f, "{x}"),
            Value::Function(name, ..) => write!(f, "<fn {name}>"),
        }
    }
}

impl Value {
    pub fn typ(&self) -> String {
        match &self {
            Value::Nil => "nil".to_string(),
            Value::Bool(_) => "bool".to_string(),
            Value::Int(_) => "int".to_string(),
            Value::Float(_) => "float".to_string(),
            Value::Str(_) => "string".to_string(),
            Value::Function(..) => "function".to_string(),
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
            match stmt {
                Stmt::Func(..) | Stmt::Let(..) => {
                    let _ = self.execute(stmt, &mut globals);
                }
                _ => {
                    semantic_error("expected function or variable declarations at top-level", stmt.token());
                    unreachable!()
                }
            }
        }

        if !globals.exists("main".to_string()) {
            error(format!("'{}' function not found", format::bold("main")).as_str());
        }

        let main_func = globals.get("main".to_string());
        if let Value::Function(_, _, body) = main_func {
            let mut main_env = Environment::from(&globals);
            let _ = self.execute(&body, &mut main_env);
        } else {
            error(format!("found '{}', but it's not a function", format::bold("main")).as_str());
        }

        self.stmts = stmts;
        self.globals = globals;
    }

    fn execute(&mut self, stmt: &Stmt, environ: &mut Environment) -> Result<(), ControlFlow> {
        match stmt {
            Stmt::Return(_, expr) => {
                let value = match expr {
                    Some(e) => self.eval(e, environ),
                    None => Value::Nil,
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
            Stmt::Func(name, params, ret_type, body) => self.execute_func(name, params, ret_type, body, environ),
        }
    }

    fn eval(&mut self, expression: &Expr, environ: &mut Environment) -> Value {
        match expression {
            Expr::Binary(left, op, right) => self.eval_binary(left, op, right, environ),
            Expr::Unary(v, op) => self.eval_unary(v, op, environ),
            Expr::FuncCall(name, args) => self.eval_func_call(name, args, environ),
            Expr::Literal(v) => self.eval_literal(v),
            Expr::Variable(sym) => self.eval_variable(sym, environ),
            Expr::Grouping(exp) => self.eval(exp, environ),
            Expr::Logical(left, op, right) => self.eval_logical(left, op, right, environ),
            _ => unreachable!()
        }
    }

    fn eval_literal(&mut self, v: &Token) -> Value {
        match &v.kind {
            TokenKind::False => Value::Bool(false),
            TokenKind::True => Value::Bool(true),
            TokenKind::Number(_) => self.eval_number_literal(v),
            TokenKind::Str(_) => self.eval_string_literal(v),
            _ => unreachable!(),
        }
    }

    fn eval_number_literal(&mut self, token: &Token) -> Value {
        let mut nstring = String::new();
        if let TokenKind::Number(n) = &token.kind {
            nstring = n.clone();
        };
        let s = nstring.as_str();
        if let Ok(x) = s.parse::<i64>() {
            Value::Int(x)
        } else if let Ok(x) = s.parse::<f64>() {
            Value::Float(x)
        } else {
            semantic_error(format!("invalid number literal '{nstring}'").as_str(), token);
            unreachable!()
        }
    }

    fn eval_string_literal(&mut self, token: &Token) -> Value {
        let mut string = String::new();
        if let TokenKind::Str(s) = &token.kind {
            string = s.clone();
        }
        let s = &string[1..string.len()-1]; // Remove trailing "
        Value::Str(s.to_string())
    }

    fn eval_unary(&mut self, v: &Token, e: &Expr, environ: &mut Environment) -> Value {
        let value = self.eval(e, environ);
        match v.kind {
            TokenKind::Minus => {
                if let Value::Int(i) = value {
                    return Value::Int(-i);
                } else if let Value::Float(f) = value {
                    return Value::Float(-f);
                } else {
                    semantic_error(format!("cannot use '-' on value {value}").as_str(), v);
                    unreachable!()
                }
            }
            TokenKind::Not => {
                if let Value::Bool(x) = value {
                    return Value::Bool(!x);
                } else {
                    semantic_error(format!("cannot use '!' on value {value}").as_str(), v);
                    unreachable!()
                }
            }
            _ => {
                semantic_error(format!("{v} is not a unary expression").as_str(), v);
                unreachable!()
            }
        }
    }

    fn eval_binary(&mut self, left: &Expr, op: &Token, right: &Expr, environ: &mut Environment) -> Value {
        let left_value = self.eval(left, environ);
        let right_value = self.eval(right, environ);
        self.perform_op(left_value, op, right_value)
    }

    fn perform_op(&mut self, left: Value, op: &Token, right: Value) -> Value {
        match (left.clone(), right.clone()) {
            (Value::Int(lv), Value::Int(rv)) => {
                match op.kind {
                    TokenKind::Plus => Value::Int(lv + rv),
                    TokenKind::Minus => Value::Int(lv - rv),
                    TokenKind::Mul => Value::Int(lv*rv),
                    TokenKind::Div => Value::Int(lv/rv),
                    TokenKind::Modulo => Value::Int(lv%rv),
                    TokenKind::Less => Value::Bool(lv < rv),
                    TokenKind::LessEq => Value::Bool(lv <= rv),
                    TokenKind::Greater => Value::Bool(lv > rv),
                    TokenKind::GreaterEq => Value::Bool(lv >= rv),
                    TokenKind::Equals => Value::Bool(lv == rv),
                    TokenKind::NotEq => Value::Bool(lv != rv),
                    _ => {
                        semantic_error(format!("unknown operator '{op}'").as_str(), op);
                        unreachable!()
                    }
                }
            }
            (Value::Float(lv), Value::Float(rv)) => {
                match op.kind {
                    TokenKind::Plus => Value::Float(lv + rv),
                    TokenKind::Minus => Value::Float(lv - rv),
                    TokenKind::Mul => Value::Float(lv*rv),
                    TokenKind::Div => Value::Float(lv/rv),
                    TokenKind::Modulo => Value::Float(lv%rv),
                    TokenKind::Less => Value::Bool(lv < rv),
                    TokenKind::LessEq => Value::Bool(lv <= rv),
                    TokenKind::Greater => Value::Bool(lv > rv),
                    TokenKind::GreaterEq => Value::Bool(lv >= rv),
                    TokenKind::Equals => Value::Bool(lv == rv),
                    TokenKind::NotEq => Value::Bool(lv != rv),
                    _ => {
                        semantic_error(format!("unknown operator '{op}'").as_str(), op);
                        unreachable!()
                    }
                }
            }
            (Value::Str(lv), Value::Str(rv)) => {
                match op.kind {
                    TokenKind::Plus => Value::Str(lv + &rv),
                    TokenKind::Equals => Value::Bool(lv == rv),
                    TokenKind::NotEq => Value::Bool(lv != rv),
                    _ => {
                        semantic_error(format!("operator '{op}' is not usable on string").as_str(), op);
                        unreachable!()
                    }
                }
            }
            _ => {
                semantic_error(format!("cannot use operator '{op}' between '{}' and '{}'", left.typ(), right.typ()).as_str(), op);
                unreachable!()
            }
        }
    }

    fn eval_logical(&mut self, left: &Expr, op: &Token, right: &Expr, environ: &mut Environment) -> Value {
        let left_value = self.eval(left, environ);
        let right_value = self.eval(right, environ);
        match (left_value.clone(), right_value.clone()) {
            (Value::Bool(lv), Value::Bool(rv)) => {
                match op.kind {
                    TokenKind::Or => Value::Bool(lv || rv),
                    TokenKind::And => Value::Bool(lv && rv),
                    _ => {
                        semantic_error(format!("operator '{op}' is not usable in logical expressions").as_str(), op);
                        unreachable!()
                    }
                }
            }
            _ => {
                semantic_error(format!("cannot use operator '{op}' between '{}' and '{}'", left_value.typ(), right_value.typ()).as_str(), op);
                unreachable!()
            }
        }
    }

    fn eval_func_call(&mut self, name: &Expr, args: &Vec<Expr>, environ: &mut Environment) -> Value {
        let mut arguments: Vec<Value> = Vec::new();
        for arg in args {
            arguments.push(self.eval(arg, environ));
        }

        let name_string = match name {
            Expr::Variable(s) => {
                match &s.kind {
                    TokenKind::Symbol(sym) => sym.clone(),
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        };

        if !environ.exists(name_string.clone()) {
            semantic_error(format!("undefined function '{}'", format::bold(name_string.as_str())).as_str(), name.token());
        }
        let func = environ.get(name_string.clone());
        if let Value::Function(_, params, body) = func {
            let arity = params.len();
            if args.len() < arity {
                semantic_error(format!("too few arguments to function '{}'. expected {}, have {}", format::bold(name_string.as_str()), arity, args.len()).as_str(), name.token());
            } else if args.len() > arity {
                semantic_error(format!("too many arguments to function '{}'. expected {}, have {}", format::bold(name_string.as_str()), arity, args.len()).as_str(), name.token());
            }

            let mut env = Environment::new();
            for (param, arg) in params.iter().zip(arguments.into_iter()) {
                if let TokenKind::Symbol(param_name) = &param.0.kind {
                    if let TokenKind::PrimitiveType(param_type) = &param.1.kind {
                        if param_type == "void" {
                            semantic_error(format!("type of parameter '{}' cannot be void.", format::bold(param_name.as_str())).as_str(), &param.0);
                        }
                    }
                    env.insert(param_name.clone(), arg);
                }
            }

            match self.execute(&body, &mut env) {
                Err(ControlFlow::Return(v)) => v,
                Ok(_) => Value::Nil,
            }
        } else {
            semantic_error(format!("'{}' is not a function", format::bold(name_string.as_str())).as_str(), name.token());
            unreachable!()
        }
    }

    fn eval_variable(&mut self, sym_tk: &Token, environ: &mut Environment) -> Value {
        let name = match &sym_tk.kind {
            TokenKind::Symbol(s) => s.clone(),
            _ => unreachable!(),
        };

        if !environ.exists(name.clone()) {
            semantic_error(format!("undefined variable '{}'", format::bold(name.as_str())).as_str(), sym_tk);
        }
        environ.get(name)
    }

    fn execute_assignment(&mut self, expr1: &Expr, expr2: &Expr, environ: &mut Environment) -> Result<(), ControlFlow> {
        let rhs = self.eval(expr2, environ);
        if let Expr::Variable(name_tk) = expr1 {
            let name = match &name_tk.kind {
                TokenKind::Symbol(s) => s.clone(),
                _ => unreachable!(),
            };

            environ.update(name, rhs);
            Ok(())
        } else {
            unreachable!()
        }
    }

    // TODO: implement type checking
    fn execute_let(&mut self, name_tk: &Token, typ_tk: &Token, expr: &Expr, environ: &mut Environment) -> Result<(), ControlFlow> {
        let mut name = String::new();
        if let TokenKind::Symbol(s) = &name_tk.kind {
            name = s.clone();
        };
        let typ = match &typ_tk.kind {
            TokenKind::Symbol(s) | TokenKind::PrimitiveType(s) => s.clone(),
            _ => unreachable!(),
        };
        if typ == "void".to_string() {
            semantic_error(format!("type of '{}' cannot be void", format::bold(name.as_str())).as_str(), typ_tk);
        }

        let val = self.eval(expr, environ);
        if environ.exists(name.clone()) {
            semantic_error(format!("redefinition of variable '{}'", format::bold(name.as_str())).as_str(), name_tk);
        }
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
        let condition = match self.eval(cond, environ) {
            Value::Bool(x) => x,
            _ => {
                semantic_error("expected boolean expression", cond.token());
                unreachable!()
            }
        };
        while condition {
            self.execute(block, environ)?;
        }
        Ok(())
    }

    fn execute_if(&mut self, cond: &Expr, then_branch: &Stmt, else_branch: &Option<Box<Stmt>>, environ: &mut Environment) -> Result<(), ControlFlow> {
        let condition = match self.eval(cond, environ) {
            Value::Bool(x) => x,
            _ => {
                semantic_error("expected boolean expression", cond.token());
                unreachable!()
            }
        };
        if condition {
            self.execute(then_branch, environ)?;
        } else if let Some(els) = else_branch {
            self.execute(els, environ)?;
        }
        Ok(())
    }

    fn execute_func(&mut self, name: &Token, params: &Vec<(Token, Token)>, _ret_type: &Token, body: &Stmt, environ: &mut Environment) -> Result<(), ControlFlow> {
        let name_str = match &name.kind {
            TokenKind::Symbol(s) => s.clone(),
            _ => unreachable!(),
        };
        let func = Value::Function(name_str.clone(), params.clone(), Box::new(body.clone()));

        if environ.exists(name_str.clone()) {
            semantic_error(format!("redefinition of function '{}'", format::bold(name_str.as_str())).as_str(), name);
            unreachable!()
        }
        environ.insert(name_str, func);
        Ok(())
    }
}

fn error(msg: &str) {
    eprintln!("{}: {msg}", format::error("error"));
    process::exit(1);
}

fn semantic_error(msg: &str, token: &Token) {
    eprintln!("{}:{}:{}: {}: {msg}", token.origin_file, token.pos.0, token.pos.1, format::error("error"));
    process::exit(1);
}
