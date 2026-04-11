use super::*;
use crate::{Ir1Program, Lexer, LogLevel, Logger, Parser};

fn lower_ir2_from_code(code: &str) -> Ir2Program {
    let tokens = Lexer::run(&code.to_string());
    let ast = Parser::parse(&tokens).unwrap();
    let ir1 = Ir1Program::lower(&ast).unwrap();
    Ir2Program::lower(&ir1).unwrap()
}

#[test]
fn lower_linear_program_into_single_block() {
    let ir2 = lower_ir2_from_code(">>+<<-.");

    assert_eq!(ir2.functions.len(), 1);
    let func = &ir2.functions[0];
    assert_eq!(func.entry, 0);
    assert_eq!(func.blocks.len(), 1);

    let block = &func.blocks[0];
    assert_eq!(
        block.insts,
        vec![
            Ir2Inst::AddPtrImm(2),
            Ir2Inst::AddCellImm(1),
            Ir2Inst::AddPtrImm(-2),
            Ir2Inst::AddCellImm(-1),
            Ir2Inst::WriteCellLow8,
        ]
    );
    assert_eq!(block.term, Some(Ir2Terminator::Return));
}

#[test]
fn lower_loop_program_into_cfg() {
    let ir2 = lower_ir2_from_code("[-]");
    let func = &ir2.functions[0];

    assert_eq!(func.blocks.len(), 4);

    // entry -> head
    assert_eq!(func.blocks[0].term, Some(Ir2Terminator::Jump(1)));

    // head branches to exit(3) or body(2)
    assert_eq!(
        func.blocks[1].term,
        Some(Ir2Terminator::BranchCellZero(3, 2))
    );

    // body executes and jumps back to head
    assert_eq!(func.blocks[2].insts, vec![Ir2Inst::AddCellImm(-1)]);
    assert_eq!(func.blocks[2].term, Some(Ir2Terminator::Jump(1)));

    // exit returns
    assert_eq!(func.blocks[3].term, Some(Ir2Terminator::Return));
}

#[test]
fn bf2_parse_from_raw_string() {
    let raw = r#"
BF2
FUNC entry ENTRY 0
BLOCK 0
  PTR 2
  CELL -1
  TERM RETURN
END_FUNC
"#;

    let ir2 = Ir2Program::from_bf2_str(raw).unwrap();
    assert_eq!(ir2.functions.len(), 1);
    assert_eq!(ir2.functions[0].name, "entry");
    assert_eq!(ir2.functions[0].blocks.len(), 1);
    assert_eq!(ir2.functions[0].blocks[0].insts.len(), 2);
    assert_eq!(ir2.functions[0].blocks[0].term, Some(Ir2Terminator::Return));
}

#[test]
fn bf2_roundtrip_raw_string() {
    let ir2 = lower_ir2_from_code("[-]");
    let text = ir2.to_bf2_string();
    let reparsed = Ir2Program::from_bf2_str(&text).unwrap();
    assert_eq!(reparsed, ir2);
}

#[test]
fn lower_with_logger_keeps_cfg_shape() {
    let logger = Logger::new(LogLevel::Debug);
    let tokens = Lexer::run_with_logger("[-]", Some(&logger));
    let ast = Parser::parse_with_logger(&tokens, Some(&logger)).unwrap();
    let ir1 = Ir1Program::lower_with_logger(&ast, Some(&logger)).unwrap();
    let ir2 = Ir2Program::lower_with_logger(&ir1, Some(&logger)).unwrap();

    assert_eq!(ir2.functions.len(), 1);
    assert_eq!(ir2.functions[0].blocks.len(), 4);
}

#[test]
fn bf2_parse_rejects_invalid_function_header_shape() {
    let raw = r#"
BF2
FUNC entry 0
"#;

    let err = Ir2Program::from_bf2_str(raw).unwrap_err();
    assert!(matches!(err, Ir2Error::ParseInvalidFunctionHeader { .. }));
}

#[test]
fn bf2_parse_rejects_term_with_missing_operand() {
    let raw = r#"
BF2
FUNC entry ENTRY 0
BLOCK 0
  TERM JUMP
END_FUNC
"#;

    let err = Ir2Program::from_bf2_str(raw).unwrap_err();
    assert!(matches!(err, Ir2Error::ParseInvalidOperand { .. }));
}
