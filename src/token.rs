#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    Comma,
    Semicolon,
    Colon,

    Equal,
    Plus,
    Minus,
    Mul,
    Div,
    Modulo,
    PlusEq,
    MinusEq,
    MulEq,
    DivEq,
    ModuloEq,

    Equals,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    And,
    Or,
    Not,
    NotEq,

    Symbol(String),
    Str(String),
    Number(String),

    Func,
    Let,
    False,
    True,
    If,
    Else,
    While,
    Return,
    Print,
    Println,
    PrimitiveType(String),

    EOF,
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: (usize, usize),
}
