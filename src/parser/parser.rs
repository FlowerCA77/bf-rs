use std::rc::Rc;

use crate::{List, LogLevel, Logger, Nil, ParseError, Token};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AstNode {
    Run(Vec<Token>),
    Loop(Rc<List<AstNode>>),
}

pub type Ast = List<AstNode>;

pub struct Parser {}

impl Parser {
    pub fn parse(tkstream: &[Token]) -> Result<Ast, ParseError> {
        Self::parse_with_logger(tkstream, None)
    }

    pub fn parse_with_logger(tkstream: &[Token], logger: Option<&Logger>) -> Result<Ast, ParseError> {
        if let Some(logger) = logger {
            logger.emit_raw(
                LogLevel::Info,
                "PARSER",
                "I_PARSE_START",
                &format!("tokens={}", tkstream.len()),
            );
        }

        let mut pos = 0usize;
        let ast = match Self::parse_block(&mut pos, tkstream, logger) {
            Ok(ast) => ast,
            Err(err) => {
                if let Some(logger) = logger {
                    logger.emit_error(&err);
                }
                return Err(err);
            }
        };

        if pos < tkstream.len() {
            let err = ParseError::UnexpectedRightBracket { pos };
            if let Some(logger) = logger {
                logger.emit_error(&err);
            }
            return Err(err);
        }

        if let Some(logger) = logger {
            logger.emit_raw(
                LogLevel::Debug,
                "PARSER",
                "D_PARSE_DONE",
                &format!("consumed_tokens={}", pos),
            );
        }

        Ok(ast)
    }

    fn parse_block(
        pos: &mut usize,
        tokens: &[Token],
        logger: Option<&Logger>,
    ) -> Result<Ast, ParseError> {
        let mut accer = Rc::new(Nil);

        while *pos < tokens.len() {
            match tokens[*pos] {
                Token::JMPIN => {
                    let left_bracket_pos = *pos;
                    if let Some(logger) = logger {
                        logger.emit_raw(
                            LogLevel::Verbose,
                            "PARSER",
                            "V_PARSE_LOOP_BEGIN",
                            &format!("token_pos={}", *pos),
                        );
                    }
                    *pos += 1;

                    let inner_list = Self::parse_block(pos, tokens, logger)?;

                    if *pos >= tokens.len() || tokens[*pos] != Token::JMPOUT {
                        return Err(ParseError::UnclosedLeftBracket {
                            pos: left_bracket_pos,
                        });
                    }
                    *pos += 1;

                    accer = Rc::new(List::Cons(AstNode::Loop(Rc::new(inner_list)), accer));
                }
                Token::JMPOUT => {
                    if let Some(logger) = logger {
                        logger.emit_raw(
                            LogLevel::Verbose,
                            "PARSER",
                            "V_PARSE_LOOP_END",
                            &format!("token_pos={}", *pos),
                        );
                    }
                    break;
                }
                _ => {
                    let mut run_tokens = Vec::new();
                    while *pos < tokens.len() {
                        match tokens[*pos] {
                            Token::JMPIN | Token::JMPOUT => break,
                            ref token => {
                                run_tokens.push(token.clone());
                                *pos += 1;
                            }
                        }
                    }

                    if !run_tokens.is_empty() {
                        if let Some(logger) = logger {
                            logger.emit_raw(
                                LogLevel::Debug,
                                "PARSER",
                                "D_PARSE_RUN_SEGMENT",
                                &format!("segment_len={} ends_at={}", run_tokens.len(), *pos),
                            );
                        }
                        accer = Rc::new(List::Cons(AstNode::Run(run_tokens), accer));
                    }
                }
            }
        }

        Ok(accer.reverse())
    }
}

#[cfg(test)]
#[path = "test_parser.rs"]
mod tests;
