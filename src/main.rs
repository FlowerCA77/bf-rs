use std::dbg;

use bf_rs::{Lexer, Parser, Ir1Program};

fn main() {
    let bfcode = ">>+<<-.";

    let tkstream = Lexer::run(&bfcode.to_string());

    let ast = Parser::parse(&tkstream).unwrap();

    let ir1 = Ir1Program::lower(&ast);
    match ir1 {
        Ok(ir1ok) => {dbg!(ir1ok);}
        Err(_) => {println!("Ir1 failed");}
    }
}
