use std::dbg;

use bf_rs::{Lexer, Parser, Ir1Program};

/*
 * ====== THIS IS NOT THE ENTRY POINT FOR NOW! ======
 * ====== THE BINARY ENTRY IS NOT YET STARTED NOW! ======
 * ====== THIS CODE IS JUST FOR QUICK DEBUG! ======
 */
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
