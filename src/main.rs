use smol::lexer::Lexer;
use smol::parser::Parser;
use smol::interpreter::Interpreter;
use std::fs;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    let source = read_file(&args[1]);
    let tokens = Lexer::new(source).lex();
    let mut parser = Parser::new(tokens);
    let mut interpreter = Interpreter::new(parser.parse());
    interpreter.interpret();
}

fn read_file(filepath: &str) -> String {
    fs::read_to_string(String::from(filepath)).expect("error opening file")
}
