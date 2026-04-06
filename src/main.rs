use std::dbg;

use bf_rs::Lexer;
use bf_rs::Parser;

fn main() {
    let mut lexer = Lexer{depth: 0};
    let parser = Parser{};

    let bf_code = ">>[-]<<[->>+<<]";
    let tkstream = lexer.run(&bf_code.to_string());
    let ir0 = parser.parse_phase1(&tkstream);
    dbg!(ir0);
}