mod ir1;
mod ir2;
mod lexer;
mod linked_list;
mod parser;

pub use crate::{
    ir1::ir1::Ir1Program,
    lexer::lexer::{Lexer, Token},
    linked_list::linked_list::{Cons, List, Nil},
    parser::parser::Parser,
};
