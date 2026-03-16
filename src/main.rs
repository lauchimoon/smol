use smol::lexer::Lexer;
use smol::parser::Parser;
use smol::interpreter::Interpreter;
use smol::format;
use std::fs;
use std::env;
use std::process;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        error("no input file");
    }

    let filename = args[1].clone();
    let source = read_file(&filename);
    let tokens = Lexer::new(&filename, source).lex();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut interpreter = Interpreter::new(ast);
    interpreter.interpret();
}

fn read_file(filepath: &str) -> String {
    match fs::read_to_string(String::from(filepath)) {
        Ok(src) => src,
        Err(_) => {
            error(format!("file '{filepath}' not found").as_str());
            unreachable!()
        }
    }
}

fn error(msg: &str) {
    eprintln!("{}: {}: {msg}", format::bold("smol"), format::error("error"));
    process::exit(1);
}
