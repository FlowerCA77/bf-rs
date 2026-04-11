use super::*;
use crate::{AstNode, Ir1Error, Lexer, List, Parser, Token};
use std::rc::Rc;

fn lower_from_code(code: &str) -> Ir1Program {
    let tokens = Lexer::run(&code.to_string());
    let ast = Parser::parse(&tokens).unwrap();
    let lowered = Ir1Program::lower(&ast);
    assert!(lowered.is_ok());
    lowered.ok().unwrap()
}

#[test]
fn lower_linear_program_merges_runs_correctly() {
    let ir = lower_from_code(">>+<<-.");
    assert_eq!(ir.root.len(), 5);

    match &ir.root[0] {
        Ir1Inst::PtrMove(v) => assert_eq!(*v, 2),
        _ => panic!("expected PtrMove(2)"),
    }
    match &ir.root[1] {
        Ir1Inst::CellAdd(v) => assert_eq!(*v, 1),
        _ => panic!("expected CellAdd(1)"),
    }
    match &ir.root[2] {
        Ir1Inst::PtrMove(v) => assert_eq!(*v, -2),
        _ => panic!("expected PtrMove(-2)"),
    }
    match &ir.root[3] {
        Ir1Inst::CellAdd(v) => assert_eq!(*v, -1),
        _ => panic!("expected CellAdd(-1)"),
    }
    match &ir.root[4] {
        Ir1Inst::Output => {}
        _ => panic!("expected Output"),
    }
}

#[test]
fn lower_nested_loop_program() {
    let ir = lower_from_code("[-]");
    assert_eq!(ir.root.len(), 1);

    match &ir.root[0] {
        Ir1Inst::Loop(inner) => {
            assert_eq!(inner.len(), 1);
            match &inner[0] {
                Ir1Inst::CellAdd(v) => assert_eq!(*v, -1),
                _ => panic!("expected CellAdd(-1) in inner loop"),
            }
        }
        _ => panic!("expected Loop node"),
    }
}

#[test]
fn lower_errors_if_run_contains_bracket_token() {
    let malformed_ast = List::Cons(AstNode::Run(vec![Token::JMPIN]), Rc::new(List::Nil));

    let result = Ir1Program::lower(&malformed_ast);
    assert!(matches!(result, Err(Ir1Error::UnexpectedBracketInRun)));
}

#[test]
fn bf1_parse_from_raw_string() {
    let raw = r#"
BF1
PTR 2
CELL -1
LOOP_BEGIN
  CELL -1
LOOP_END
OUT
"#;

    let ir1 = Ir1Program::from_bf1_str(raw).unwrap();
    assert_eq!(
        ir1.root,
        vec![
            Ir1Inst::PtrMove(2),
            Ir1Inst::CellAdd(-1),
            Ir1Inst::Loop(vec![Ir1Inst::CellAdd(-1)]),
            Ir1Inst::Output,
        ]
    );
}

#[test]
fn bf1_roundtrip_raw_string() {
    let ir = lower_from_code(">>+<<-.");
    let text = ir.to_bf1_string();
    let reparsed = Ir1Program::from_bf1_str(&text).unwrap();
    assert_eq!(reparsed, ir);
}
