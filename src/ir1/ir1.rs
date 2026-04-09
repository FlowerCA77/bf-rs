use crate::{
    List::{self, Cons, Nil},
    Token,
    parser::parser::{Ast, AstNode},
};

pub enum LowerError {
    UnexpectedBracketInRun,
    Overflow,
}

#[derive(Debug)]
pub struct Ir1Program {
    pub root: Ir1Block,
}

pub type Ir1Block = Vec<Ir1Inst>;

#[derive(Debug)]
pub enum Ir1Inst {
    PtrMove(i32),
    CellAdd(i32),
    Input,
    Output,
    Loop(Ir1Block),
}

impl Ir1Program {
    fn flush_ptr(ptr_delta: &mut i32, out_block: &mut Ir1Block) {
        if *ptr_delta != 0 {
            out_block.push(Ir1Inst::PtrMove(*ptr_delta));
            *ptr_delta = 0;
        }
    }

    fn flush_cell(cell_delta: &mut i32, out_block: &mut Ir1Block) {
        if *cell_delta != 0 {
            out_block.push(Ir1Inst::CellAdd(*cell_delta));
            *cell_delta = 0;
        }
    }

    fn lower_run(tkstream: &[Token], out_block: &mut Ir1Block) -> Result<(), LowerError> {
        let mut ptr_delta = 0;
        let mut cell_delta = 0;

        for tk in tkstream {
            match tk {
                Token::MOVR => {
                    Ir1Program::flush_cell(&mut cell_delta, out_block);
                    ptr_delta += 1;
                }
                Token::MOVL => {
                    Ir1Program::flush_cell(&mut cell_delta, out_block);
                    ptr_delta -= 1;
                }
                Token::INC => {
                    Ir1Program::flush_ptr(&mut ptr_delta, out_block);
                    cell_delta += 1;
                }
                Token::DEC => {
                    Ir1Program::flush_ptr(&mut ptr_delta, out_block);
                    cell_delta -= 1;
                }
                Token::INPUT => {
                    Ir1Program::flush_ptr(&mut ptr_delta, out_block);
                    Ir1Program::flush_cell(&mut cell_delta, out_block);
                    out_block.push(Ir1Inst::Input);
                }
                Token::OUTPUT => {
                    Ir1Program::flush_ptr(&mut ptr_delta, out_block);
                    Ir1Program::flush_cell(&mut cell_delta, out_block);
                    out_block.push(Ir1Inst::Output);
                }
                Token::JMPIN | Token::JMPOUT => {
                    return Err(LowerError::UnexpectedBracketInRun);
                }
                _ => (),
            };
        }

        Ir1Program::flush_ptr(&mut ptr_delta, out_block);
        Ir1Program::flush_cell(&mut cell_delta, out_block);

        Ok(())
    }

    fn lower_block(astNodes: &List<AstNode>) -> Result<Ir1Block, LowerError> {
        let mut out_block = Ir1Block::new();
        let mut cursor = astNodes;
        loop {
            match cursor {
                Cons(node, next) => {
                    match node {
                        AstNode::Run(tkstream) => {
                            Ir1Program::lower_run(tkstream, &mut out_block)?;
                        }
                        AstNode::Loop(inner_ast) => {
                            let inner_block = Ir1Program::lower_block(&inner_ast);
                            out_block.push(Ir1Inst::Loop(inner_block?));
                        }
                    };
                    cursor = next;
                }
                Nil => {
                    break;
                }
            }
        }
        Ok(out_block)
    }

    pub fn lower(ast: &Ast) -> Result<Ir1Program, LowerError> {
        Ok(Ir1Program {
            root: Ir1Program::lower_block(ast)?,
        })
    }
}
