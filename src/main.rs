use bf_rs::{BfError, Ir1Program, Lexer, Parser};

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

fn compile_to_ir1(bfcode: &str) -> Result<Ir1Program, BfError> {
    let tkstream = Lexer::run(&bfcode.to_string());
    let ast = Parser::parse(&tkstream)?;
    let ir1 = Ir1Program::lower(&ast)?;
    Ok(ir1)
}
