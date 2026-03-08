use crate::token::Token;
use crate::ast::Expr;
use crate::ast::Stmt;

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {tokens: tokens, cursor: 0}
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.statement());
        }
        stmts
    }

    fn statement(&mut self) -> Stmt {
        if matches!(self.current(), Token::Return) {
            self.advance();
            return self.return_stmt();
        }
        if matches!(self.current(), Token::Let) {
            self.advance();
            return self.let_stmt();
        }
        if matches!(self.current(), Token::While) {
            self.advance();
            return self.while_stmt();
        }
        Stmt::Expression(self.expression())
    }

    fn return_stmt(&mut self) -> Stmt {
        if matches!(self.current(), Token::Semicolon) {
            self.advance();
            return Stmt::Return(None);
        }
        let expr = self.expression();
        self.consume_expected(Token::Semicolon, "return: expected ';'");
        return Stmt::Return(Some(expr));
    }

    fn let_stmt(&mut self) -> Stmt {
        let name = self.consume().clone();
        if !matches!(name, Token::Symbol(_)) {
            panic!("let: expected Symbol, got {:#?}", name);
        }
        self.consume_expected(Token::Colon, "let: expected ':'");
        let typ = self.consume().clone();
        if !matches!(typ, Token::Symbol(_)) {
            panic!("let: expected Symbol, got {:#?}", typ);
        }
        self.consume_expected(Token::Equal, "let: expected '='");
        let value = self.expression();
        self.consume_expected(Token::Semicolon, "let: expected ';'");
        Stmt::Let(name, typ, value)
    }

    fn while_stmt(&mut self) -> Stmt {
        self.consume_expected(Token::OpenParen, "while: expected '('");
        let cond = self.expression();
        self.consume_expected(Token::CloseParen, "while: expected ')'");
        let body = self.block();
        Stmt::While(cond, body)
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut stmts: Vec<Stmt> = Vec::new();
        self.consume_expected(Token::OpenCurly, "block: expected '{{'");
        while !matches!(self.current(), Token::CloseCurly) && !self.is_at_end() {
            stmts.push(self.statement());
        }

        self.consume_expected(Token::CloseCurly, "block: expected '}}'");
        stmts
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while matches!(self.current(), Token::NotEq | Token::Equals) {
            let op = self.consume().clone();
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        while matches!(self.current(),Token::Greater | Token::GreaterEq |
                        Token::Less | Token::LessEq) {
            let op = self.consume().clone();
            let right = self.term();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while matches!(self.current(),Token::Plus | Token::Minus) {
            let op = self.consume().clone();
            let right = self.factor();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while matches!(self.current(), Token::Asterisk | Token::Div) {
            let op = self.consume().clone();
            let right = self.unary();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        if matches!(self.current(), Token::Not | Token::Minus) {
            let op = self.consume().clone();
            let right = self.unary();
            return Expr::Unary(op, Box::new(right));
        }
        self.primary()
    }

    fn primary(&mut self) -> Expr {
        if matches!(self.current(), Token::False) {
            return Expr::Literal(Token::False);
        }
        if matches!(self.current(), Token::True) {
            return Expr::Literal(Token::True);
        }
        if matches!(self.current(), Token::Str(_) | Token::Number(_)) {
            let token = self.consume().clone();
            return Expr::Literal(token);
        }
        if matches!(self.current(), Token::OpenParen) {
            self.advance();
            let expr = self.expression();
            self.consume_expected(Token::CloseParen, "expression: expected ')'");
            return Expr::Grouping(Box::new(expr));
        }
        panic!("expression: unknown case {:#?}", self.current());
    }

    fn previous(&self) -> &Token {
        let mut idx = 0;
        if self.cursor > 0 {
            idx = self.cursor - 1;
        }
        &self.tokens[idx]
    }

    fn current(&self) -> &Token {
        if self.cursor > self.tokens.len() {
            return self.previous();
        }
        &self.tokens[self.cursor]
    }

    fn advance(&mut self) {
        self.cursor += 1;
    }

    fn consume(&mut self) -> &Token {
        let token = &self.tokens[self.cursor];
        self.cursor += 1;
        token
    }

    fn is_at_end(&self) -> bool {
        self.current().clone() == Token::EOF
    }

    fn consume_expected(&mut self, expected: Token, message: &str) {
        let check = self.consume().clone();
        if check != expected {
            panic!("{}", message);
        }
    }
}
