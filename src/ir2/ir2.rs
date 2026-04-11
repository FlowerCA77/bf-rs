use std::fs;
use std::path::Path;

use crate::{Ir1Block, Ir1Inst, Ir1Program, Ir2Error};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ir2Inst {
    AddPtrImm(i64),
    AddCellImm(i64),
    ReadByteToCell,
    WriteCellLow8,
}

pub type BlockId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ir2Terminator {
    Jump(BlockId),
    BranchCellZero(/* zero */ BlockId, /* non-zero */ BlockId),
    Return,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ir2Block {
    pub id: BlockId,
    pub insts: Vec<Ir2Inst>,
    pub term: Option<Ir2Terminator>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ir2Function {
    pub name: String,
    pub entry: BlockId,
    pub blocks: Vec<Ir2Block>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ir2Program {
    pub functions: Vec<Ir2Function>,
}

struct Builder {
    next_block_id: BlockId,
    blocks: Vec<Ir2Block>,
    current_block: BlockId,
}

impl Builder {
    fn new() -> Builder {
        Builder {
            next_block_id: 0,
            blocks: Vec::new(),
            current_block: 0,
        }
    }

    fn new_block(&mut self) -> BlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;
        self.blocks.push(Ir2Block {
            id,
            insts: Vec::new(),
            term: None,
        });
        id
    }

    fn set_current(&mut self, id: BlockId) -> Result<(), Ir2Error> {
        if id >= self.blocks.len() {
            return Err(Ir2Error::InvalidCurrentBlock { id });
        }
        self.current_block = id;
        Ok(())
    }

    fn push_inst(&mut self, inst: Ir2Inst) -> Result<(), Ir2Error> {
        let cur = self.current_block;
        let block = self.block_mut(cur)?;
        block.insts.push(inst);
        Ok(())
    }

    fn set_term(&mut self, term: Ir2Terminator) -> Result<(), Ir2Error> {
        let cur = self.current_block;
        let block = self.block_mut(cur)?;
        if block.term.is_some() {
            return Err(Ir2Error::TerminatorAlreadySet {
                block_id: self.current_block,
            });
        }
        block.term = Some(term);
        Ok(())
    }

    fn block_mut(&mut self, id: BlockId) -> Result<&mut Ir2Block, Ir2Error> {
        if id >= self.blocks.len() {
            return Err(Ir2Error::InvalidCurrentBlock { id });
        }
        Ok(&mut self.blocks[id])
    }

    fn current_has_term(&self) -> Result<bool, Ir2Error> {
        if self.current_block >= self.blocks.len() {
            return Err(Ir2Error::InvalidCurrentBlock {
                id: self.current_block,
            });
        }
        Ok(self.blocks[self.current_block].term.is_some())
    }
}

fn emit_loop(builder: &mut Builder, inner: &Ir1Block) -> Result<(), Ir2Error> {
    let head = builder.new_block();
    let body = builder.new_block();
    let exit = builder.new_block();

    // 1) current -> head
    if !builder.current_has_term()? {
        builder.set_term(Ir2Terminator::Jump(head))?;
    }

    // 2) head -> branch
    builder.set_current(head)?;
    builder.set_term(Ir2Terminator::BranchCellZero(exit, body))?;

    // 3) body -> inner -> jump head
    builder.set_current(body)?;
    emit_ir1_block(builder, inner)?;
    if !builder.current_has_term()? {
        builder.set_term(Ir2Terminator::Jump(head))?;
    }

    // 4) continue at exit
    builder.set_current(exit)?;
    Ok(())
}

fn emit_ir1_block(builder: &mut Builder, ir1_block: &Ir1Block) -> Result<(), Ir2Error> {
    for inst in ir1_block {
        match inst {
            Ir1Inst::PtrMove(v) => {
                builder.push_inst(Ir2Inst::AddPtrImm(*v))?;
            }
            Ir1Inst::CellAdd(v) => {
                builder.push_inst(Ir2Inst::AddCellImm(*v))?;
            }
            Ir1Inst::Input => {
                builder.push_inst(Ir2Inst::ReadByteToCell)?;
            }
            Ir1Inst::Output => {
                builder.push_inst(Ir2Inst::WriteCellLow8)?;
            }
            Ir1Inst::Loop(inner) => {
                emit_loop(builder, inner)?;
            }
        }
    }
    Ok(())
}

fn lower_ir1_function(name: &str, root_block: &Ir1Block) -> Result<Ir2Function, Ir2Error> {
    let mut builder = Builder::new();
    let entry = builder.new_block();
    builder.set_current(entry)?;

    emit_ir1_block(&mut builder, root_block)?;

    if !builder.current_has_term()? {
        builder.set_term(Ir2Terminator::Return)?;
    }

    Ok(Ir2Function {
        name: name.to_string(),
        entry,
        blocks: builder.blocks,
    })
}

impl Ir2Program {
    pub fn lower(ir1_prog: &Ir1Program) -> Result<Ir2Program, Ir2Error> {
        let func = lower_ir1_function("entry", &ir1_prog.root)?;
        Ok(Ir2Program {
            functions: vec![func],
        })
    }

    pub fn to_bf2_string(&self) -> String {
        let mut out = String::from("BF2\n");
        for func in &self.functions {
            out.push_str(&format!("FUNC {} ENTRY {}\n", func.name, func.entry));
            for block in &func.blocks {
                out.push_str(&format!("BLOCK {}\n", block.id));
                for inst in &block.insts {
                    match inst {
                        Ir2Inst::AddPtrImm(v) => out.push_str(&format!("  PTR {}\n", v)),
                        Ir2Inst::AddCellImm(v) => out.push_str(&format!("  CELL {}\n", v)),
                        Ir2Inst::ReadByteToCell => out.push_str("  IN\n"),
                        Ir2Inst::WriteCellLow8 => out.push_str("  OUT\n"),
                    }
                }
                if let Some(term) = &block.term {
                    match term {
                        Ir2Terminator::Jump(target) => {
                            out.push_str(&format!("  TERM JUMP {}\n", target));
                        }
                        Ir2Terminator::BranchCellZero(zero, nonzero) => {
                            out.push_str(&format!("  TERM BRANCH_ZERO {} {}\n", zero, nonzero));
                        }
                        Ir2Terminator::Return => {
                            out.push_str("  TERM RETURN\n");
                        }
                    }
                }
            }
            out.push_str("END_FUNC\n");
        }
        out
    }

    pub fn from_bf2_str(input: &str) -> Result<Ir2Program, Ir2Error> {
        let lines = Self::meaningful_lines(input);
        if lines.is_empty() {
            return Err(Ir2Error::ParseInvalidHeader {
                found: String::from("<empty>"),
            });
        }

        let (_, header) = lines[0];
        if header != "BF2" {
            return Err(Ir2Error::ParseInvalidHeader {
                found: header.to_string(),
            });
        }

        let mut idx = 1usize;
        let mut functions = Vec::new();

        while idx < lines.len() {
            let (line_no, line) = lines[idx];
            let (func_name, entry) = Self::parse_func_header(line_no, line)?;
            idx += 1;

            let mut blocks = Vec::new();
            let mut saw_end = false;

            while idx < lines.len() {
                let (cur_line_no, cur_line) = lines[idx];

                if cur_line == "END_FUNC" {
                    idx += 1;
                    saw_end = true;
                    break;
                }

                let block_id = Self::parse_block_header(cur_line_no, cur_line)?;
                idx += 1;

                let mut insts = Vec::new();
                let mut term: Option<Ir2Terminator> = None;

                while idx < lines.len() {
                    let (inst_line_no, inst_line) = lines[idx];
                    if inst_line == "END_FUNC" || inst_line.starts_with("BLOCK ") {
                        break;
                    }

                    if let Some(rest) = inst_line.strip_prefix("TERM ") {
                        if term.is_some() {
                            return Err(Ir2Error::TerminatorAlreadySet { block_id });
                        }
                        term = Some(Self::parse_term(inst_line_no, rest)?);
                    } else {
                        insts.push(Self::parse_inst(inst_line_no, inst_line)?);
                    }

                    idx += 1;
                }

                if term.is_none() {
                    return Err(Ir2Error::ParseMissingTerminator {
                        function: func_name.clone(),
                        block_id,
                    });
                }

                blocks.push(Ir2Block {
                    id: block_id,
                    insts,
                    term,
                });
            }

            if !saw_end {
                return Err(Ir2Error::ParseMissingEndFunc {
                    function: func_name,
                });
            }

            let has_entry = blocks.iter().any(|b| b.id == entry);
            if !has_entry {
                return Err(Ir2Error::EntryBlockNotFound {
                    function: func_name,
                    entry,
                });
            }

            functions.push(Ir2Function {
                name: func_name,
                entry,
                blocks,
            });
        }

        Ok(Ir2Program { functions })
    }

    pub fn write_bf2_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Ir2Error> {
        let p = path.as_ref();
        fs::write(p, self.to_bf2_string()).map_err(|e| Ir2Error::Io {
            path: p.to_path_buf(),
            message: e.to_string(),
        })
    }

    pub fn read_bf2_file<P: AsRef<Path>>(path: P) -> Result<Ir2Program, Ir2Error> {
        let p = path.as_ref();
        let content = fs::read_to_string(p).map_err(|e| Ir2Error::Io {
            path: p.to_path_buf(),
            message: e.to_string(),
        })?;
        Ir2Program::from_bf2_str(&content)
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

    fn parse_func_header(line: usize, content: &str) -> Result<(String, BlockId), Ir2Error> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() != 4 || parts[0] != "FUNC" || parts[2] != "ENTRY" {
            return Err(Ir2Error::ParseInvalidFunctionHeader {
                line,
                content: content.to_string(),
            });
        }

        let entry = parts[3]
            .parse::<usize>()
            .map_err(|_| Ir2Error::ParseInvalidOperand {
                line,
                content: content.to_string(),
            })?;

        Ok((parts[1].to_string(), entry))
    }

    fn parse_block_header(line: usize, content: &str) -> Result<BlockId, Ir2Error> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != "BLOCK" {
            return Err(Ir2Error::ParseInvalidBlockHeader {
                line,
                content: content.to_string(),
            });
        }

        parts[1]
            .parse::<usize>()
            .map_err(|_| Ir2Error::ParseInvalidOperand {
                line,
                content: content.to_string(),
            })
    }

    fn parse_inst(line: usize, content: &str) -> Result<Ir2Inst, Ir2Error> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.is_empty() {
            return Err(Ir2Error::ParseInvalidInstruction {
                line,
                content: content.to_string(),
            });
        }

        match parts[0] {
            "PTR" => {
                if parts.len() != 2 {
                    return Err(Ir2Error::ParseInvalidOperand {
                        line,
                        content: content.to_string(),
                    });
                }
                let val = parts[1]
                    .parse::<i64>()
                    .map_err(|_| Ir2Error::ParseInvalidOperand {
                        line,
                        content: content.to_string(),
                    })?;
                Ok(Ir2Inst::AddPtrImm(val))
            }
            "CELL" => {
                if parts.len() != 2 {
                    return Err(Ir2Error::ParseInvalidOperand {
                        line,
                        content: content.to_string(),
                    });
                }
                let val = parts[1]
                    .parse::<i64>()
                    .map_err(|_| Ir2Error::ParseInvalidOperand {
                        line,
                        content: content.to_string(),
                    })?;
                Ok(Ir2Inst::AddCellImm(val))
            }
            "IN" => {
                if parts.len() != 1 {
                    return Err(Ir2Error::ParseInvalidOperand {
                        line,
                        content: content.to_string(),
                    });
                }
                Ok(Ir2Inst::ReadByteToCell)
            }
            "OUT" => {
                if parts.len() != 1 {
                    return Err(Ir2Error::ParseInvalidOperand {
                        line,
                        content: content.to_string(),
                    });
                }
                Ok(Ir2Inst::WriteCellLow8)
            }
            _ => Err(Ir2Error::ParseInvalidInstruction {
                line,
                content: content.to_string(),
            }),
        }
    }

    fn parse_term(line: usize, content: &str) -> Result<Ir2Terminator, Ir2Error> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.is_empty() {
            return Err(Ir2Error::ParseInvalidInstruction {
                line,
                content: content.to_string(),
            });
        }

        match parts[0] {
            "RETURN" => {
                if parts.len() != 1 {
                    return Err(Ir2Error::ParseInvalidOperand {
                        line,
                        content: format!("TERM {}", content),
                    });
                }
                Ok(Ir2Terminator::Return)
            }
            "JUMP" => {
                if parts.len() != 2 {
                    return Err(Ir2Error::ParseInvalidOperand {
                        line,
                        content: format!("TERM {}", content),
                    });
                }
                let target =
                    parts[1]
                        .parse::<usize>()
                        .map_err(|_| Ir2Error::ParseInvalidOperand {
                            line,
                            content: format!("TERM {}", content),
                        })?;
                Ok(Ir2Terminator::Jump(target))
            }
            "BRANCH_ZERO" => {
                if parts.len() != 3 {
                    return Err(Ir2Error::ParseInvalidOperand {
                        line,
                        content: format!("TERM {}", content),
                    });
                }
                let zero =
                    parts[1]
                        .parse::<usize>()
                        .map_err(|_| Ir2Error::ParseInvalidOperand {
                            line,
                            content: format!("TERM {}", content),
                        })?;
                let nonzero =
                    parts[2]
                        .parse::<usize>()
                        .map_err(|_| Ir2Error::ParseInvalidOperand {
                            line,
                            content: format!("TERM {}", content),
                        })?;
                Ok(Ir2Terminator::BranchCellZero(zero, nonzero))
            }
            _ => Err(Ir2Error::ParseInvalidInstruction {
                line,
                content: format!("TERM {}", content),
            }),
        }
    }
}

#[cfg(test)]
#[path = "test_ir2.rs"]
mod tests;
