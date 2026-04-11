use crate::{
    BlockId, Ir2Block, Ir2Function, Ir2Inst, Ir2Program, Ir2Terminator, LogLevel, Logger,
    RuntimeError,
};
use std::{
    collections::HashMap,
    io::{Read, Stdin, Stdout, Write, stdin, stdout},
};

const BFCAP: usize = 30_000;
const ENTRY_FUNCTION_NAME: &str = "entry";

pub struct Status {
    tape: Vec<i64>,
    pos: i64,
    cur_block_id: BlockId,
    instream: Stdin,
    outstream: Stdout,
    logger: Option<Logger>,
}

impl Status {
    fn emit_trace(&self, level: LogLevel, code: &str, detail: String) {
        if let Some(logger) = &self.logger {
            logger.emit_raw(level, "BFVM", code, &detail);
        }
    }

    fn current_cell(&self) -> i64 {
        self.tape[self.pos as usize]
    }

    fn mov(&mut self, v: i64) -> Result<(), RuntimeError> {
        let attempted = self.pos as i128 + v as i128;
        if attempted >= BFCAP as i128 || attempted < 0 {
            return Err(RuntimeError::PtrOutOfBounds {
                current: self.pos,
                delta: v,
                attempted,
                min: 0,
                max: (BFCAP - 1) as i128,
            });
        }
        self.pos = attempted as i64;
        self.emit_trace(
            LogLevel::Debug,
            "D_BFVM_PTR_MOVE",
            format!("ptr moved by {} -> {}", v, self.pos),
        );

        Ok(())
    }

    fn add(&mut self, v: i64) -> Result<(), RuntimeError> {
        let idx = self.pos as usize;
        let before = self.tape[idx];
        self.tape[idx] = self.tape[idx].wrapping_add(v);
        self.emit_trace(
            LogLevel::Debug,
            "D_BFVM_CELL_ADD",
            format!(
                "cell[{}] {} + {} -> {}",
                self.pos, before, v, self.tape[idx]
            ),
        );

        Ok(())
    }

    fn input(&mut self) -> Result<(), RuntimeError> {
        let mut buf = [0u8];
        match self.instream.read(&mut buf) {
            Ok(0) => {
                self.tape[self.pos as usize] = 0i64;
                self.emit_trace(
                    LogLevel::Verbose,
                    "V_BFVM_STDIN_EOF",
                    format!("stdin EOF -> cell[{}] set to 0", self.pos),
                );
                Ok(())
            }
            Ok(_) => {
                self.tape[self.pos as usize] = buf[0] as i64;
                self.emit_trace(
                    LogLevel::Verbose,
                    "V_BFVM_STDIN_READ",
                    format!("stdin byte {} -> cell[{}]", buf[0], self.pos),
                );
                Ok(())
            }
            Err(e) => Err(RuntimeError::Io {
                operation: String::from("stdin read"),
                message: e.to_string(),
            }),
        }
    }

    fn output(&mut self) -> Result<(), RuntimeError> {
        let buf = [self.tape[self.pos as usize] as u8];
        let write_result = self
            .outstream
            .write_all(&buf)
            .map_err(|e| RuntimeError::Io {
                operation: String::from("stdout write"),
                message: e.to_string(),
            });

        if write_result.is_ok() {
            self.emit_trace(
                LogLevel::Verbose,
                "V_BFVM_STDOUT_WRITE",
                format!("stdout byte {} from cell[{}]", buf[0], self.pos),
            );
        }

        write_result
    }

    fn execute_block(
        &mut self,
        function_name: &str,
        lookup: &HashMap<BlockId, &Ir2Block>,
        block_id: BlockId,
    ) -> Result<Ir2Terminator, RuntimeError> {
        let block = lookup
            .get(&block_id)
            .ok_or_else(|| RuntimeError::UnknownBlockId {
                function: function_name.to_string(),
                block_id,
            })?;

        self.cur_block_id = block.id;
        self.emit_trace(
            LogLevel::Verbose,
            "V_BFVM_ENTER_BLOCK",
            format!(
                "function={} block={} ptr={} cell={}",
                function_name,
                block.id,
                self.pos,
                self.current_cell()
            ),
        );

        for inst in &block.insts {
            self.emit_trace(
                LogLevel::Debug,
                "D_BFVM_EXEC_INST",
                format!(
                    "function={} block={} inst={:?}",
                    function_name, block.id, inst
                ),
            );
            match inst {
                Ir2Inst::AddPtrImm(v) => self.mov(*v)?,
                Ir2Inst::AddCellImm(v) => self.add(*v)?,
                Ir2Inst::ReadByteToCell => self.input()?,
                Ir2Inst::WriteCellLow8 => self.output()?,
            }
        }

        block
            .term
            .clone()
            .ok_or_else(|| RuntimeError::MissingTerminator {
                function: function_name.to_string(),
                block_id: block.id,
            })
    }

    fn execute_function(&mut self, func: &Ir2Function) -> Result<(), RuntimeError> {
        if func.blocks.is_empty() {
            return Err(RuntimeError::EntryBlockNotFound {
                function: func.name.clone(),
                entry: func.entry,
            });
        }

        self.emit_trace(
            LogLevel::Info,
            "I_BFVM_ENTER_FUNCTION",
            format!("function={} entry={}", func.name, func.entry),
        );

        let mut lookup = HashMap::<BlockId, &Ir2Block>::new();

        for block in &func.blocks {
            if lookup.get(&block.id).is_some() {
                return Err(RuntimeError::DuplicateBlockId {
                    function: func.name.clone(),
                    block_id: block.id,
                });
            }
            lookup.insert(block.id, block);
        }

        if !lookup.contains_key(&func.entry) {
            return Err(RuntimeError::EntryBlockNotFound {
                function: func.name.clone(),
                entry: func.entry,
            });
        }

        self.cur_block_id = func.entry;

        loop {
            let term = self.execute_block(&func.name, &lookup, self.cur_block_id)?;
            self.emit_trace(
                LogLevel::Debug,
                "D_BFVM_TERMINATOR",
                format!(
                    "function={} block={} term={:?}",
                    func.name, self.cur_block_id, term
                ),
            );

            match term {
                Ir2Terminator::Jump(block_id) => {
                    self.cur_block_id = block_id;
                }
                Ir2Terminator::BranchCellZero(zero, nonzero) => {
                    self.cur_block_id = if self.tape[self.pos as usize] == 0 {
                        zero
                    } else {
                        nonzero
                    };
                }
                Ir2Terminator::Return => {
                    self.emit_trace(
                        LogLevel::Info,
                        "I_BFVM_RETURN",
                        format!("function={} returned", func.name),
                    );
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn run(&mut self, ir2_prog: Ir2Program) -> Result<(), RuntimeError> {
        let mut entry_func: Option<&Ir2Function> = None;
        for func in &ir2_prog.functions {
            if func.name == ENTRY_FUNCTION_NAME {
                if entry_func.is_some() {
                    return Err(RuntimeError::DuplicateEntryFunction {
                        name: ENTRY_FUNCTION_NAME.to_string(),
                    });
                }
                entry_func = Some(func);
            }
        }

        let entry_func = entry_func.ok_or_else(|| RuntimeError::EntryFunctionNotFound {
            name: ENTRY_FUNCTION_NAME.to_string(),
        })?;

        self.emit_trace(
            LogLevel::Info,
            "I_BFVM_SELECT_ENTRY",
            format!("selected entry function={}", entry_func.name),
        );

        self.execute_function(entry_func)
    }

    pub fn new() -> Status {
        Status {
            tape: vec![0_i64; BFCAP],
            pos: 0,
            cur_block_id: 0,
            instream: stdin(),
            outstream: stdout(),
            logger: None,
        }
    }

    pub fn with_logger(logger: Logger) -> Status {
        let mut vm = Status::new();
        vm.logger = Some(logger);
        vm
    }

    pub fn attach_logger(&mut self, logger: Logger) {
        self.logger = Some(logger);
    }

    pub fn detach_logger(&mut self) {
        self.logger = None;
    }
}

pub fn execute_str_on(code: &str, vm: &mut Status) -> Result<(), RuntimeError> {
    let ir2_prog = Ir2Program::from_bf2_str(code).map_err(RuntimeError::from)?;
    vm.run(ir2_prog)
}

#[cfg(test)]
#[path = "test_bfvm.rs"]
mod tests;
