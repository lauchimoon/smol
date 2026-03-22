use crate::token::Token;
use crate::token::TokenKind;
use crate::ast::Expr;
use crate::ast::Stmt;
use crate::format;
use std::process;

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
        if matches!(self.current().kind, TokenKind::Func) {
            self.advance();
            return self.fn_stmt();
        }
        if matches!(self.current().kind, TokenKind::Print | TokenKind::Println) {
            let mut newline = false;
            if matches!(self.current().kind, TokenKind::Println) {
                newline = true;
            }
            self.advance();
            return self.print_stmt(newline);
        }
        if matches!(self.current().kind, TokenKind::Return) {
            let token = self.consume().clone();
            return self.return_stmt(token);
        }
        if matches!(self.current().kind, TokenKind::Let) {
            self.advance();
            return self.let_stmt();
        }
        if matches!(self.current().kind, TokenKind::While) {
            self.advance();
            return self.while_stmt();
        }
        if matches!(self.current().kind, TokenKind::If) {
            self.advance();
            return self.if_stmt();
        }
        let expr = Stmt::Expression(self.expression());
        self.consume_expected(TokenKind::Semicolon, "expected ';' after expression");
        expr
    }

    fn fn_stmt(&mut self) -> Stmt {
        let name = self.consume().clone();
        if !matches!(name.kind, TokenKind::Symbol(_)) {
            syntax_error("symbol", &name);
        }
        self.consume_expected(TokenKind::OpenParen, "expected '('");

        let mut params: Vec<(Token, Token)> = Vec::new();
        if !matches!(self.current().kind, TokenKind::CloseParen) {
            params.push(self.param());
            while matches!(self.current().kind, TokenKind::Comma) {
                self.advance();
                params.push(self.param());
            }
        }
        self.consume_expected(TokenKind::CloseParen, "expected ')' after parameter list");

        let ret_type = self.consume().clone();
        if !matches!(ret_type.kind, TokenKind::Symbol(_) | TokenKind::PrimitiveType(_)) {
            syntax_error("symbol or primitive type", &ret_type);
        }
        let body = Box::new(Stmt::Block(self.block()));
        Stmt::Func(name, params, ret_type, body)
    }

    fn param(&mut self) -> (Token, Token) {
        let name = self.consume().clone();
        if !matches!(name.kind, TokenKind::Symbol(_)) {
            syntax_error("symbol", &name);
        }
        self.consume_expected(TokenKind::Colon, "expected ':' after symbol");
        let typ = self.consume().clone();
        if !matches!(typ.kind, TokenKind::Symbol(_) | TokenKind::PrimitiveType(_)) {
            syntax_error("symbol or primitive type", &typ);
        }
        (name, typ)
    }

    fn print_stmt(&mut self, newline: bool) -> Stmt {
        let expr = self.expression();
        self.consume_expected(TokenKind::Semicolon, "expected ';' after print");
        Stmt::Print(expr, newline)
    }

    fn return_stmt(&mut self, token: Token) -> Stmt {
        if matches!(self.current().kind, TokenKind::Semicolon) {
            self.advance();
            return Stmt::Return(token, None);
        }
        let expr = self.expression();
        self.consume_expected(TokenKind::Semicolon, "expected ';' after return");
        Stmt::Return(token, Some(expr))
    }

    fn let_stmt(&mut self) -> Stmt {
        let name = self.consume().clone();
        if !matches!(name.kind, TokenKind::Symbol(_)) {
            syntax_error("symbol", &name);
        }
        self.consume_expected(TokenKind::Colon, "expected ':' after symbol");
        let typ = self.consume().clone();
        if !matches!(typ.kind, TokenKind::Symbol(_) | TokenKind::PrimitiveType(_)) {
            syntax_error("symbol or primitive type", &name);
        }
        self.consume_expected(TokenKind::Equal, "expected '=' after type");
        let value = self.expression();
        self.consume_expected(TokenKind::Semicolon, "expected ';' after expression");
        Stmt::Let(name, typ, value)
    }

    fn while_stmt(&mut self) -> Stmt {
        self.consume_expected(TokenKind::OpenParen, "expected '(' after while");
        let cond = self.expression();
        self.consume_expected(TokenKind::CloseParen, "expected ')' after expression");
        let body = Box::new(Stmt::Block(self.block()));
        Stmt::While(cond, body)
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut stmts: Vec<Stmt> = Vec::new();
        self.consume_expected(TokenKind::OpenCurly, "expected '{' when defining block");
        while !matches!(self.current().kind, TokenKind::CloseCurly) && !self.is_at_end() {
            stmts.push(self.statement());
        }
        self.consume_expected(TokenKind::CloseCurly, "expected '}' after defining block");
        stmts
    }

    fn if_stmt(&mut self) -> Stmt {
        self.consume_expected(TokenKind::OpenParen, "expected '(' after if");
        let cond = self.expression();
        self.consume_expected(TokenKind::CloseParen, "expected ')' after expression");
        let then_block = Box::new(Stmt::Block(self.block()));
        let mut else_block = None;
        if matches!(self.current().kind, TokenKind::Else) {
            self.advance();
            if matches!(self.current().kind, TokenKind::If) {
                self.advance();
                else_block = Some(Box::new(self.if_stmt()));
            } else {
                else_block = Some(Box::new(Stmt::Block(self.block())));
            }
        }
        Stmt::If(cond, then_block, else_block)
    }

    fn expression(&mut self) -> Expr {
        self.assignment()
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or();
        if matches!(self.current().kind, TokenKind::Equal) {
            self.advance();
            let value = self.assignment();
            if matches!(expr, Expr::Variable(_)) {
                return Expr::Assignment(Box::new(expr), Box::new(value));
            }
            die("invalid assignment target", expr.token());
        }
        expr
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();
        while matches!(self.current().kind, TokenKind::Or) {
            let op = self.consume().clone();
            let right = self.and();
            expr = Expr::Logical(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.equality();
        while matches!(self.current().kind, TokenKind::And) {
            let op = self.consume().clone();
            let right = self.equality();
            expr = Expr::Logical(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while matches!(self.current().kind, TokenKind::NotEq | TokenKind::Equals) {
            let op = self.consume().clone();
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        while matches!(self.current().kind, TokenKind::Greater | TokenKind::GreaterEq |
                        TokenKind::Less | TokenKind::LessEq) {
            let op = self.consume().clone();
            let right = self.term();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while matches!(self.current().kind, TokenKind::Plus | TokenKind::Minus) {
            let op = self.consume().clone();
            let right = self.factor();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while matches!(self.current().kind, TokenKind::Modulo | TokenKind::Mul | TokenKind::Div) {
            let op = self.consume().clone();
            let right = self.unary();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        if matches!(self.current().kind, TokenKind::Not | TokenKind::Minus) {
            let op = self.consume().clone();
            let right = self.unary();
            return Expr::Unary(op, Box::new(right));
        }
        self.func_call()
    }

    fn func_call(&mut self) -> Expr {
        let mut expr = self.primary();
        loop {
            if matches!(self.current().kind, TokenKind::OpenParen) {
                if let Expr::Variable(_) = expr {
                    self.advance();
                    expr = self.finish_func_call(expr);
                } else {
                    syntax_error("symbol", self.previous());
                }
            }
            break;
        }
        expr
    }

    fn finish_func_call(&mut self, callee: Expr) -> Expr {
        let mut args: Vec<Expr> = Vec::new();
        if !matches!(self.current().kind, TokenKind::CloseParen) {
            args.push(self.expression());
            while matches!(self.current().kind, TokenKind::Comma) {
                if args.len() >= 255 {
                    die("cannot have more than 255 arguments on function call", self.current());
                }
                self.advance();
                args.push(self.expression());
            }
        }
        self.consume_expected(TokenKind::CloseParen, "expected ')' after function call");
        Expr::FuncCall(Box::new(callee), args)
    }


    fn primary(&mut self) -> Expr {
        if matches!(self.current().kind, TokenKind::False) {
            let token = self.consume().clone();
            return Expr::Literal(token);
        }
        if matches!(self.current().kind, TokenKind::True) {
            let token = self.consume().clone();
            return Expr::Literal(token);
        }
        if matches!(self.current().kind, TokenKind::Str(_) | TokenKind::Number(_) | TokenKind::Char(_)) {
            let token = self.consume().clone();
            return Expr::Literal(token);
        }
        if matches!(self.current().kind, TokenKind::Symbol(_)) {
            let token = self.consume().clone();
            return Expr::Variable(token);
        }
        if matches!(self.current().kind, TokenKind::OpenParen) {
            self.advance();
            let expr = self.expression();
            self.consume_expected(TokenKind::CloseParen, "expected ')' after expression grouping");
            return Expr::Grouping(Box::new(expr));
        }

        syntax_error("expression", self.current());
        unreachable!();
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
        self.current().clone().kind == TokenKind::EOF
    }

    fn consume_expected(&mut self, expected: TokenKind, msg: &str) {
        let check = self.consume().clone();
        if check.kind != expected {
            println!("{}:{}:{} {}", check.origin_file, check.pos.0, check.pos.1, msg);
            process::exit(1);
        }
    }
}

fn syntax_error(expected: &str, token: &Token) {
    eprintln!("{}:{}:{}: {}: expected {expected} but got {token}", token.origin_file, token.pos.0, token.pos.1, format::error("error"));
    process::exit(1);
}

fn die(msg: &str, token: &Token) {
    eprintln!("{}:{}:{} {}: {msg}", token.origin_file, token.pos.0, token.pos.1, format::error("error"));
    process::exit(1);
}
