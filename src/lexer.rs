use crate::token::Token;

#[derive(Debug)]
pub struct Lexer {
    source: Vec<char>,
    cursor: usize,
    source_len: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Lexer {
            source: source.chars().collect(),
            cursor: 0usize,
            source_len: source.len(),
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        while self.cursor < self.source_len {
            let mut c = self.chop();
            if c.is_alphabetic() {
                let mut value = String::new();
                value.push(c);
                c = self.chop();
                while c.is_alphabetic() || c.is_digit(10) || c == '_' {
                    value.push(c);
                    c = self.chop();
                }
                let tok = self.identify_symbol_or_keyword(value);
                tokens.push(tok);
            }

            if c.is_digit(10) {
                let mut value = String::new();
                value.push(c);
                c = self.chop();
                while c.is_digit(10) || c == '.' {
                    value.push(c);
                    c = self.chop();
                }
                tokens.push(Token::Number(value));
            }

            if c == '"' {
                let mut value = String::from("\"");
                c = self.chop();
                while c != '"' {
                    value.push(c);
                    c = self.chop();
                }
                value.push(c);
                tokens.push(Token::Str(value));
                continue;
            }

            if c.is_whitespace() {
                continue;
            }

            if c == '(' {
                tokens.push(Token::OpenParen);
            } else if c == ')' {
                tokens.push(Token::CloseParen);
            } else if c == '{' {
                tokens.push(Token::OpenCurly);
            } else if c == '}' {
                tokens.push(Token::CloseCurly);
            } else if c == ',' {
                tokens.push(Token::Comma);
            } else if c == ';' {
                tokens.push(Token::Semicolon);
            } else if c == ':' {
                tokens.push(Token::Colon);
            } else if c == '=' {
                if self.current() == '=' {
                    tokens.push(Token::Equals);
                    _ = self.chop();
                    continue;
                }
                tokens.push(Token::Equal);
            } else if c == '+' {
                if self.current() == '=' {
                    tokens.push(Token::PlusEq);
                    _ = self.chop();
                    continue;
                }
                tokens.push(Token::Plus);
            } else if c == '-' {
                if self.current() == '=' {
                    tokens.push(Token::MinusEq);
                    _ = self.chop();
                    continue;
                }
                tokens.push(Token::Minus);
            } else if c == '*' {
                if self.current() == '=' {
                    tokens.push(Token::MulEq);
                    _ = self.chop();
                    continue;
                }
                tokens.push(Token::Asterisk);
            } else if c == '/' {
                if self.current() == '=' {
                    tokens.push(Token::DivEq);
                    _ = self.chop();
                    continue;
                }
                tokens.push(Token::Div);
            } else if c == '%' {
                if self.current() == '=' {
                    tokens.push(Token::ModuloEq);
                    _ = self.chop();
                    continue;
                }
                tokens.push(Token::Modulo);
            } else if c == '<' {
                if self.current() == '=' {
                    tokens.push(Token::LessEq);
                    _ = self.chop();
                    continue;
                }
                tokens.push(Token::Less);
            } else if c == '>' {
                if self.current() == '=' {
                    tokens.push(Token::GreaterEq);
                    _ = self.chop();
                    continue;
                }
                tokens.push(Token::Greater);
            } else if c == '&' {
                if self.current() == '&' {
                    tokens.push(Token::And);
                    _ = self.chop();
                    continue;
                }
            } else if c == '|' {
                if self.current() == '|' {
                    tokens.push(Token::Or);
                    _ = self.chop();
                    continue;
                }
            } else if c == '!' {
                if self.current() == '=' {
                    tokens.push(Token::NotEq);
                    _ = self.chop();
                    continue;
                }
                tokens.push(Token::Not);
            } else {
                tokens.push(Token::Unknown(String::from(c)));
            }
        }
        tokens
    }

    fn chop(&mut self) -> char {
        if self.cursor >= self.source_len {
            return ' ';
        }
        let c = self.source[self.cursor];
        self.cursor += 1;
        c
    }

    fn current(&mut self) -> char {
        if self.cursor >= self.source_len {
            return ' ';
        }
        self.source[self.cursor]
    }

    fn identify_symbol_or_keyword(&self, value: String) -> Token {
        match value.as_str() {
            "fn" => Token::Func,
            "let" => Token::Let,
            "int" => Token::IntType,
            "float" => Token::FloatType,
            "string" => Token::StringType,
            "bool" => Token::BoolType,
            "false" => Token::False,
            "true" => Token::True,
            "if" => Token::If,
            "elif" => Token::Elif,
            "else" => Token::Else,
            "while" => Token::While,
            _ => Token::Symbol(value)
        }
    }
}
