use smol::lexer::Lexer;
use smol::parser::Parser;
use std::fs;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    let source = read_file(&args[1]);
    let tokens = Lexer::new(source).lex();
    println!("{:#?}", tokens);
    let mut parser = Parser::new(tokens);
    println!("{:#?}", parser.parse());
}

fn read_file(filepath: &str) -> String {
    fs::read_to_string(String::from(filepath)).expect("error opening file")
}
