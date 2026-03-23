use crate::token::Token;

#[derive(Clone, Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    FuncCall(Box<Expr>, Vec<Expr>),
    Literal(Token),
    Variable(Token),
    Grouping(Box<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
}

// stmt ::= ("print" | "println") expr ";" |
//          return expr? ";" |
//          break ";" |
//          continue ";" |
//          let symbol ":" (symbol | PrimitiveType) = expr ";" |
//          while "(" expr ")" { stmt* } |
//          if "(" expr ")" { stmt* } (else { stmt* })? |
//          "{" stmt* "}" |
//          fn symbol "(" param ("," param)* ")" (symbol | PrimitiveType) { stmt* }, where param ::= symbol: symbol |
//          expr-stmt
#[derive(Clone, Debug)]
pub enum Stmt {
    Print(Expr, bool),
    Return(Token, Option<Expr>),
    Break(Token),
    Continue(Token),
    Let(Token, Token, Expr),
    While(Expr, Box<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Block(Vec<Stmt>),
    Func(Token, Vec<(Token, Token)>, Token, Box<Stmt>),
    Expression(Expr),
}

impl Expr {
    pub fn token(&self) -> &Token {
        match &self {
            Expr::Binary(_, t, _) => t,
            Expr::Unary(t, _) => t,
            Expr::FuncCall(name, _) => name.token(),
            Expr::Literal(t) => t,
            Expr::Variable(t) => t,
            Expr::Grouping(expr) => expr.token(),
            Expr::Assignment(expr1, _) => expr1.token(),
            Expr::Logical(_, t, _) => t,
        }
    }
}

impl Stmt {
    pub fn token(&self) -> &Token {
        match &self {
            Stmt::Print(expr, _) => expr.token(),
            Stmt::Return(t, _) => t,
            Stmt::Let(t, ..) => t,
            Stmt::While(expr, _) => expr.token(),
            Stmt::If(expr, ..) => expr.token(),
            Stmt::Block(stmts) => stmts[0].token(),
            Stmt::Func(t, ..) => t,
            Stmt::Expression(expr) => expr.token(),
            Stmt::Break(t) => t,
            Stmt::Continue(t) => t,
        }
    }
}
