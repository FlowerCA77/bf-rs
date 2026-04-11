use std::fs;
use std::path::Path;

use crate::{
    Ast, AstNode, Ir1Error,
    List::{self, Cons, Nil},
    Token,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ir1Program {
    pub root: Ir1Block,
}

pub type Ir1Block = Vec<Ir1Inst>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ir1Inst {
    PtrMove(i64),
    CellAdd(i64),
    Input,
    Output,
    Loop(Ir1Block),
}

impl Ir1Program {
    fn flush_ptr(ptr_delta: &mut i64, out_block: &mut Ir1Block) {
        if *ptr_delta != 0 {
            out_block.push(Ir1Inst::PtrMove(*ptr_delta));
            *ptr_delta = 0;
        }
    }

    fn flush_cell(cell_delta: &mut i64, out_block: &mut Ir1Block) {
        if *cell_delta != 0 {
            out_block.push(Ir1Inst::CellAdd(*cell_delta));
            *cell_delta = 0;
        }
    }

    fn lower_run(tkstream: &[Token], out_block: &mut Ir1Block) -> Result<(), Ir1Error> {
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
                    return Err(Ir1Error::UnexpectedBracketInRun);
                }
                Token::COMMENT => (),
            };
        }

        Ir1Program::flush_ptr(&mut ptr_delta, out_block);
        Ir1Program::flush_cell(&mut cell_delta, out_block);

        Ok(())
    }

    fn lower_block(ast_nodes: &List<AstNode>) -> Result<Ir1Block, Ir1Error> {
        let mut out_block = Ir1Block::new();
        let mut cursor = ast_nodes;
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

    pub fn lower(ast: &Ast) -> Result<Ir1Program, Ir1Error> {
        Ok(Ir1Program {
            root: Ir1Program::lower_block(ast)?,
        })
    }

    pub fn to_bf1_string(&self) -> String {
        let mut out = String::from("BF1\n");
        Self::emit_bf1_block(&self.root, 0, &mut out);
        out
    }

    pub fn from_bf1_str(input: &str) -> Result<Ir1Program, Ir1Error> {
        let lines = Self::meaningful_lines(input);
        if lines.is_empty() {
            return Err(Ir1Error::ParseInvalidHeader {
                found: String::from("<empty>"),
            });
        }

        let (_, header) = lines[0];
        if header != "BF1" {
            return Err(Ir1Error::ParseInvalidHeader {
                found: header.to_string(),
            });
        }

        let mut idx = 1usize;
        let root = Self::parse_bf1_block(&lines, &mut idx, false)?;
        if idx != lines.len() {
            let (line, content) = lines[idx];
            return Err(Ir1Error::ParseInvalidInstruction {
                line,
                content: content.to_string(),
            });
        }

        Ok(Ir1Program { root })
    }

    pub fn write_bf1_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Ir1Error> {
        let p = path.as_ref();
        fs::write(p, self.to_bf1_string()).map_err(|e| Ir1Error::Io {
            path: p.to_path_buf(),
            message: e.to_string(),
        })
    }

    pub fn read_bf1_file<P: AsRef<Path>>(path: P) -> Result<Ir1Program, Ir1Error> {
        let p = path.as_ref();
        let content = fs::read_to_string(p).map_err(|e| Ir1Error::Io {
            path: p.to_path_buf(),
            message: e.to_string(),
        })?;
        Ir1Program::from_bf1_str(&content)
    }

    fn emit_bf1_block(block: &Ir1Block, indent: usize, out: &mut String) {
        let pad = " ".repeat(indent);
        for inst in block {
            match inst {
                Ir1Inst::PtrMove(v) => out.push_str(&format!("{}PTR {}\n", pad, v)),
                Ir1Inst::CellAdd(v) => out.push_str(&format!("{}CELL {}\n", pad, v)),
                Ir1Inst::Input => out.push_str(&format!("{}IN\n", pad)),
                Ir1Inst::Output => out.push_str(&format!("{}OUT\n", pad)),
                Ir1Inst::Loop(inner) => {
                    out.push_str(&format!("{}LOOP_BEGIN\n", pad));
                    Self::emit_bf1_block(inner, indent + 2, out);
                    out.push_str(&format!("{}LOOP_END\n", pad));
                }
            }
        }
    }

    fn meaningful_lines(input: &str) -> Vec<(usize, &str)> {
        input
            .lines()
            .enumerate()
            .filter_map(|(i, raw)| {
                let line = raw.trim();
                if line.is_empty() || line.starts_with('#') {
                    None
                } else {
                    Some((i + 1, line))
                }
            })
            .collect()
    }

    fn parse_bf1_block(
        lines: &[(usize, &str)],
        idx: &mut usize,
        stop_on_loop_end: bool,
    ) -> Result<Ir1Block, Ir1Error> {
        let mut block = Vec::new();

        while *idx < lines.len() {
            let (line_no, line) = lines[*idx];

            if line == "LOOP_END" {
                if stop_on_loop_end {
                    *idx += 1;
                    return Ok(block);
                }
                return Err(Ir1Error::ParseUnexpectedLoopEnd { line: line_no });
            }

            if line == "LOOP_BEGIN" {
                *idx += 1;
                let inner = Self::parse_bf1_block(lines, idx, true)?;
                block.push(Ir1Inst::Loop(inner));
                continue;
            }

            let inst = Self::parse_bf1_inst(line_no, line)?;
            block.push(inst);
            *idx += 1;
        }

        if stop_on_loop_end {
            Err(Ir1Error::ParseUnclosedLoop)
        } else {
            Ok(block)
        }
    }

    fn parse_bf1_inst(line_no: usize, line: &str) -> Result<Ir1Inst, Ir1Error> {
        let mut parts = line.split_whitespace();
        let op = parts
            .next()
            .ok_or_else(|| Ir1Error::ParseInvalidInstruction {
                line: line_no,
                content: line.to_string(),
            })?;

        match op {
            "PTR" => {
                let val = parts.next().ok_or_else(|| Ir1Error::ParseInvalidOperand {
                    line: line_no,
                    content: line.to_string(),
                })?;
                if parts.next().is_some() {
                    return Err(Ir1Error::ParseInvalidOperand {
                        line: line_no,
                        content: line.to_string(),
                    });
                }
                let parsed = val
                    .parse::<i64>()
                    .map_err(|_| Ir1Error::ParseInvalidOperand {
                        line: line_no,
                        content: line.to_string(),
                    })?;
                Ok(Ir1Inst::PtrMove(parsed))
            }
            "CELL" => {
                let val = parts.next().ok_or_else(|| Ir1Error::ParseInvalidOperand {
                    line: line_no,
                    content: line.to_string(),
                })?;
                if parts.next().is_some() {
                    return Err(Ir1Error::ParseInvalidOperand {
                        line: line_no,
                        content: line.to_string(),
                    });
                }
                let parsed = val
                    .parse::<i64>()
                    .map_err(|_| Ir1Error::ParseInvalidOperand {
                        line: line_no,
                        content: line.to_string(),
                    })?;
                Ok(Ir1Inst::CellAdd(parsed))
            }
            "IN" => {
                if parts.next().is_some() {
                    return Err(Ir1Error::ParseInvalidOperand {
                        line: line_no,
                        content: line.to_string(),
                    });
                }
                Ok(Ir1Inst::Input)
            }
            "OUT" => {
                if parts.next().is_some() {
                    return Err(Ir1Error::ParseInvalidOperand {
                        line: line_no,
                        content: line.to_string(),
                    });
                }
                Ok(Ir1Inst::Output)
            }
            _ => Err(Ir1Error::ParseInvalidInstruction {
                line: line_no,
                content: line.to_string(),
            }),
        }
    }
}

#[cfg(test)]
#[path = "test_ir1.rs"]
mod tests;
