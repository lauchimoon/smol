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
//          let symbol ":" (symbol | PrimitiveType) = expr ";" |
//          while "(" expr ")" { stmt* } |
//          if "(" expr ")" { stmt* } (else { stmt* })? |
//          "{" stmt* "}" |
//          fn symbol "(" param ("," param)* ")" (symbol | PrimitiveType) { stmt* }, where param ::= symbol: symbol |
//          expr-stmt
#[derive(Clone, Debug)]
pub enum Stmt {
    Print(Expr, bool),
    Return(Option<Expr>),
    Let(Token, Token, Expr),
    While(Expr, Box<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Block(Vec<Stmt>),
    Func(Token, Vec<(Token, Token)>, Token, Box<Stmt>),
    Expression(Expr),
}
