use bf_rs::{
    bferror::bferror::LowerError, ir1::ir1::Ir1Program, lexer::lexer::Lexer, parser::parser::Parser,
};

/*
 * ====== THIS IS NOT THE ENTRY POINT FOR NOW! ======
 * ====== THE BINARY ENTRY IS NOT YET STARTED NOW! ======
 * ====== THIS CODE IS JUST FOR QUICK DEBUG! ======
 */
fn main() {
    let bfcode = ">>+<<-.";

    match compile_to_ir1(bfcode) {
        Ok(ir1ok) => {
            dbg!(ir1ok);
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}

fn compile_to_ir1(bfcode: &str) -> Result<Ir1Program, LowerError> {
    let tkstream = Lexer::run(&bfcode.to_string());
    let ast_res = Parser::parse(&tkstream);
    let ir1 = match ast_res {
        Ok(ast) => Ir1Program::lower(&ast),
        Err(_) => Err(LowerError::UnexpectedBracketInRun),
    };
    ir1
}
