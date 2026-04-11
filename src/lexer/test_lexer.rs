use super::*;
use crate::{LogLevel, Logger};

#[test]
fn run_maps_brainfxxk_tokens_and_filters_comments() {
    let code = String::from("a+>[-]., <\n");
    let tokens = Lexer::run(&code);

    assert_eq!(
        tokens,
        vec![
            Token::INC,
            Token::MOVR,
            Token::JMPIN,
            Token::DEC,
            Token::JMPOUT,
            Token::OUTPUT,
            Token::INPUT,
            Token::MOVL,
        ]
    );
}

#[test]
fn run_returns_empty_for_empty_input() {
    let code = String::from("");
    let tokens = Lexer::run(&code);
    assert!(tokens.is_empty());
}

#[test]
fn run_keeps_brackets_in_order() {
    let code = String::from("[][[]]");
    let tokens = Lexer::run(&code);

    assert_eq!(
        tokens,
        vec![
            Token::JMPIN,
            Token::JMPOUT,
            Token::JMPIN,
            Token::JMPIN,
            Token::JMPOUT,
            Token::JMPOUT,
        ]
    );
}

#[test]
fn run_with_logger_keeps_tokenization_result() {
    let logger = Logger::new(LogLevel::Debug);
    let tokens = Lexer::run_with_logger("a+[]", Some(&logger));
    assert_eq!(tokens, vec![Token::INC, Token::JMPIN, Token::JMPOUT]);
}
