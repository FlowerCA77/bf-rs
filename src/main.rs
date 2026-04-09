use std::dbg;

use bf_rs::Lexer;
use bf_rs::Parser;

fn main() {
    let bf_code = ">>[-]<<[->>+<<]";
    let tkstream = Lexer::run(&bf_code.to_string());

    match Parser::parse(&tkstream) {
        Ok(ast) => {
            println!("Parse successful!");
            dbg!(ast);
        }
        Err(err) => {
            eprintln!("Parse error: {}", err);
        }
    }
}
