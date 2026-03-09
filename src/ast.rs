use crate::token::Token;

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Literal(Token),
    Variable(Token),
    Grouping(Box<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
}

// stmt ::= return expr? ";" |
//          let symbol ":" type = expr ";" |
//          while "(" expr ")" { stmt* } |
//          if "(" expr ")" { stmt* } ((elif "(" expr ")" { stmt* })*)? (else { stmt* })? |
//          "{" stmt* "}" |
//          expr-stmt
#[derive(Debug)]
pub enum Stmt {
    Return(Option<Expr>),
    Let(Token, Token, Expr),
    While(Expr, Vec<Stmt>),
    If(Expr, Vec<Stmt>, Option<Box<Stmt>>),
    Block(Vec<Stmt>),
    Expression(Expr),
}
