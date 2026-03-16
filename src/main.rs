use smol::lexer::Lexer;
use smol::parser::Parser;
use smol::interpreter::Interpreter;
use std::fs;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    let filename = args[1].clone();
    let source = read_file(&filename);
    let tokens = Lexer::new(&filename, source).lex();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut interpreter = Interpreter::new(ast);
    interpreter.interpret();
}

fn read_file(filepath: &str) -> String {
    fs::read_to_string(String::from(filepath)).expect("error opening file")
}
