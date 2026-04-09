mod ir1;
mod ir2;
mod lexer;
mod linked_list;
mod parser;

pub use crate::{
    lexer::lexer::{Lexer, Token},
    linked_list::linked_list::{Cons, List, Nil},
    parser::parser::Parser,
    ir1::ir1::{Ir1Program},
};
