use crate::token::Token;
use crate::token::TokenKind;
use crate::format;
use std::process;

#[derive(Debug)]
pub struct Lexer {
    filename: String,
    source: Vec<char>,
    cursor: usize,
    line: usize,
    chr: usize,
    source_len: usize,
}

impl Lexer {
    pub fn new(filename: &str, source: String) -> Self {
        Lexer {
            filename: filename.to_string(),
            source: source.chars().collect(),
            cursor: 0usize,
            line: 1usize,
            chr: 0usize,
            source_len: source.len(),
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        while self.cursor < self.source_len {
            let mut c = self.chop();
            if c.is_alphabetic() {
                let start = self.chr;
                let mut value = String::new();
                value.push(c);
                c = self.chop();
                while c.is_alphabetic() || c.is_digit(10) || c == '_' {
                    value.push(c);
                    c = self.chop();
                }
                let tok = self.identify_symbol_or_keyword(value);
                tokens.push(Token {origin_file: self.filename.clone(), kind: tok, pos: (self.line, start)});
            }

            if c.is_digit(10) {
                let start = self.chr;
                let mut value = String::new();
                value.push(c);
                c = self.chop();
                while c.is_digit(10) || c == '.' {
                    value.push(c);
                    c = self.chop();
                }
                tokens.push(Token {origin_file: self.filename.clone(), kind: TokenKind::Number(value), pos: (self.line, start)});
            }

            if c == '"' {
                let start = self.chr;
                let mut value = String::from("\"");
                c = self.chop();
                while c != '"' {
                    value.push(c);
                    c = self.chop();
                }
                value.push(c);
                tokens.push(Token {origin_file: self.filename.clone(), kind: TokenKind::Str(value), pos: (self.line, start)});
                continue;
            }

            if c == '\'' {
                let start = self.chr;
                let mut value = String::from("'");
                c = self.chop();
                value.push(c);
                c = self.chop();
                if c != '\'' {
                    println!("{}:{}:{}: {}: expected one character, got '{c}'", self.filename, self.line, self.chr, format::error("error"));
                    process::exit(1);
                }
                value.push(c);
                tokens.push(Token {origin_file: self.filename.clone(), kind: TokenKind::Char(value), pos: (self.line, start)});
                continue;
            }

            if c.is_whitespace() && c != '\n' {
                continue;
            }

            if c == '\n' {
                self.line += 1;
                self.chr = 0;
            } else if c == '(' {
                tokens.push(self.token(TokenKind::OpenParen));
            } else if c == ')' {
                tokens.push(self.token(TokenKind::CloseParen));
            } else if c == '{' {
                tokens.push(self.token(TokenKind::OpenCurly));
            } else if c == '}' {
                tokens.push(self.token(TokenKind::CloseCurly));
            } else if c == ',' {
                tokens.push(self.token(TokenKind::Comma));
            } else if c == ';' {
                tokens.push(self.token(TokenKind::Semicolon));
            } else if c == ':' {
                tokens.push(self.token(TokenKind::Colon));
            } else if c == '=' {
                if self.current() == '=' {
                    tokens.push(self.token(TokenKind::Equals));
                    _ = self.chop();
                    continue;
                }
                tokens.push(self.token(TokenKind::Equal));
            } else if c == '+' {
                if self.current() == '=' {
                    tokens.push(self.token(TokenKind::PlusEq));
                    _ = self.chop();
                    continue;
                }
                tokens.push(self.token(TokenKind::Plus));
            } else if c == '-' {
                if self.current() == '=' {
                    tokens.push(self.token(TokenKind::MinusEq));
                    _ = self.chop();
                    continue;
                }
                tokens.push(self.token(TokenKind::Minus));
            } else if c == '*' {
                if self.current() == '=' {
                    tokens.push(self.token(TokenKind::MulEq));
                    _ = self.chop();
                    continue;
                }
                tokens.push(self.token(TokenKind::Mul));
            } else if c == '/' {
                if self.current() == '=' {
                    tokens.push(self.token(TokenKind::DivEq));
                    _ = self.chop();
                    continue;
                }
                tokens.push(self.token(TokenKind::Div));
            } else if c == '%' {
                if self.current() == '=' {
                    tokens.push(self.token(TokenKind::ModuloEq));
                    _ = self.chop();
                    continue;
                }
                tokens.push(self.token(TokenKind::Modulo));
            } else if c == '<' {
                if self.current() == '=' {
                    tokens.push(self.token(TokenKind::LessEq));
                    _ = self.chop();
                    continue;
                }
                tokens.push(self.token(TokenKind::Less));
            } else if c == '>' {
                if self.current() == '=' {
                    tokens.push(self.token(TokenKind::GreaterEq));
                    _ = self.chop();
                    continue;
                }
                tokens.push(self.token(TokenKind::Greater));
            } else if c == '&' {
                if self.current() == '&' {
                    tokens.push(self.token(TokenKind::And));
                    _ = self.chop();
                    continue;
                }
            } else if c == '|' {
                if self.current() == '|' {
                    tokens.push(self.token(TokenKind::Or));
                    _ = self.chop();
                    continue;
                }
            } else if c == '!' {
                if self.current() == '=' {
                    tokens.push(self.token(TokenKind::NotEq));
                    _ = self.chop();
                    continue;
                }
                tokens.push(self.token(TokenKind::Not));
            } else {
                tokens.push(self.token(TokenKind::Unknown(String::from(c))));
            }
        }
        tokens.push(self.token(TokenKind::EOF));
        tokens
    }

    fn token(&self, kind: TokenKind) -> Token {
        Token {origin_file: self.filename.clone(), kind: kind, pos: (self.line, self.chr)}
    }

    fn chop(&mut self) -> char {
        if self.cursor >= self.source_len {
            return ' ';
        }
        let c = self.source[self.cursor];
        self.cursor += 1;
        self.chr += 1;
        c
    }

    fn current(&mut self) -> char {
        if self.cursor >= self.source_len {
            return ' ';
        }
        self.source[self.cursor]
    }

    fn identify_symbol_or_keyword(&self, value: String) -> TokenKind {
        match value.as_str() {
            "fn" => TokenKind::Func,
            "let" => TokenKind::Let,
            "false" => TokenKind::False,
            "true" => TokenKind::True,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "print" => TokenKind::Print,
            "println" => TokenKind::Println,
            "int" | "float" | "bool" | "string" | "void" | "char" => TokenKind::PrimitiveType(value),
            _ => TokenKind::Symbol(value)
        }
    }
}
