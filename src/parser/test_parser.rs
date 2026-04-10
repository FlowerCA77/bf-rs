use super::*;
use crate::{Lexer, List, ParseError, Token};
use std::rc::Rc;

fn list_of(nodes: Vec<AstNode>) -> Ast {
    let mut acc = List::Nil;
    for node in nodes.into_iter().rev() {
        acc = List::Cons(node, Rc::new(acc));
    }
    acc
}

#[test]
fn parse_run_only_program() {
    let code = String::from("++--");
    let tokens = Lexer::run(&code);
    let ast = Parser::parse(&tokens).unwrap();

    let expected = list_of(vec![AstNode::Run(vec![
        Token::INC,
        Token::INC,
        Token::DEC,
        Token::DEC,
    ])]);
    assert_eq!(ast, expected);
}

#[test]
fn parse_preserves_empty_loop() {
    let code = String::from("[]");
    let tokens = Lexer::run(&code);
    let ast = Parser::parse(&tokens).unwrap();

    let expected = list_of(vec![AstNode::Loop(Rc::new(List::Nil))]);
    assert_eq!(ast, expected);
}

#[test]
fn parse_nested_loop_structure() {
    let code = String::from("[[+]-]");
    let tokens = Lexer::run(&code);
    let ast = Parser::parse(&tokens).unwrap();

    let inner_loop_body = list_of(vec![AstNode::Run(vec![Token::INC])]);
    let outer_body = list_of(vec![
        AstNode::Loop(Rc::new(inner_loop_body)),
        AstNode::Run(vec![Token::DEC]),
    ]);
    let expected = list_of(vec![AstNode::Loop(Rc::new(outer_body))]);

    assert_eq!(ast, expected);
}

#[test]
fn parse_rejects_unclosed_left_bracket() {
    let code = String::from("[+");
    let tokens = Lexer::run(&code);
    let result = Parser::parse(&tokens);
    assert!(matches!(result, Err(ParseError::UnclosedLeftBracket)));
}

#[test]
fn parse_rejects_unexpected_right_bracket() {
    let code = String::from("+]");
    let tokens = Lexer::run(&code);
    let result = Parser::parse(&tokens);
    assert!(matches!(
        result,
        Err(ParseError::UnexpectedRightBracket { .. })
    ));
}
