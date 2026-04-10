use std::rc::Rc;

use crate::{List, Nil, ParseError, Token};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AstNode {
    Run(Vec<Token>),
    Loop(Rc<List<AstNode>>),
}

pub type Ast = List<AstNode>;

pub struct Parser {}

impl Parser {
    pub fn parse(tkstream: &[Token]) -> Result<Ast, ParseError> {
        let mut pos = 0usize;
        let ast = Self::parse_block(&mut pos, tkstream)?;

        if pos < tkstream.len() {
            return Err(ParseError::UnexpectedRightBracket { pos });
        }

        Ok(ast)
    }

    fn parse_block(pos: &mut usize, tokens: &[Token]) -> Result<Ast, ParseError> {
        let mut accer = Rc::new(Nil);

        while *pos < tokens.len() {
            match tokens[*pos] {
                Token::JMPIN => {
                    *pos += 1;

                    let inner_list = Self::parse_block(pos, tokens)?;

                    if *pos >= tokens.len() || tokens[*pos] != Token::JMPOUT {
                        return Err(ParseError::UnclosedLeftBracket);
                    }
                    *pos += 1;

                    accer = Rc::new(List::Cons(AstNode::Loop(Rc::new(inner_list)), accer));
                }
                Token::JMPOUT => {
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
