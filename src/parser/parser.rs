use std::rc::Rc;

use crate::lexer::lexer::Lexer;
use crate::lexer::lexer::Token;
use crate::linked_list::linked_list::Cons;
use crate::linked_list::linked_list::List;
use crate::linked_list::linked_list::Nil;

/// Segment = (loc, depth, tkstream)
type Segment = (i64, i64, Vec<Token>);

/// for example:
///     ```brainfxxk
///     012345678901234
///     >>[-]<<[->>+<<]
///     ```
/// the IR0 is:
///     ```ir0
///     (0, 0, >>) -> (3, 1, -) -> (5, 0, <<) -> (8, 1, ->>+<<) -> Nil
///     ```
///
/// consider an complexer example:
///     ```brainfxxk
///     0123456789012345678901
///     +[++[+++[++--]---]--]-
///     ```
/// the map is:
///     ```plaintext
///      depth\loc | 0 2  5   9    14  18 21
///     -----------+-------------------------
///              0 | +                    -
///              1 |   ++              --
///              2 |      +++      ---
///              3 |          ++--
///     ```
/// so the IR0 is:
///     ```ir0
///     (0, 0, +) -> (2, 1, ++) -> (5, 2, +++) -> (9, 3, ++--) ->
///         (14, 2, ---) -> (18, 1, --) -> (21, 0, -) -> Nil
///     ```
pub type IR0 = List<Segment>;

pub struct IR1 {}

pub struct IR2 {}

pub struct AST {}

pub struct Parser {}

fn unnest(
    cur_pos: i64,
    cur_depth: i64,
    cur_accer: Rc<List<Segment>>,
    res: &Vec<Token>,
) -> List<Segment> {
    if cur_pos >= res.len() as i64 {
        cur_accer.reverse()
    } else {
        match &res[cur_pos as usize] {
            Token::JMPIN(_) => unnest(cur_pos + 1, cur_depth + 1, cur_accer, res),
            Token::JMPOUT(_) => unnest(cur_pos + 1, cur_depth - 1, cur_accer, res),
            _ => {
                let mut end = cur_pos;
                let mut segment_content: Vec<Token> = Vec::new();

                while end < res.len() as i64 {
                    match &res[end as usize] {
                        Token::JMPIN(_) | Token::JMPOUT(_) => break,
                        token => {
                            segment_content.push(token.clone());
                            end += 1;
                        }
                    }
                }

                let new_accer = if !segment_content.is_empty() {
                    Rc::new(List::Cons((cur_pos, cur_depth, segment_content), cur_accer))
                } else {
                    cur_accer
                };

                unnest(end, cur_depth, new_accer, res)
            }
        }
    }
}

impl Parser {
    pub fn parse_phase1(&self, tkstream: &Vec<Token>) -> IR0 {
        unnest(0i64, 0i64, Rc::new(Nil), tkstream)
    }

    pub fn parse_phase2(&self, _ir: &IR0) -> IR1 {
        todo!()
    }

    pub fn parse_phase3(&self, _ir: &IR1) -> IR2 {
        todo!()
    }

    pub fn parse_phase4(&self, _ir: &IR2) -> AST {
        todo!()
    }

    pub fn parse(&self, tkstream: &Vec<Token>) -> AST {
        let ir0 = self.parse_phase1(tkstream);
        let ir1 = self.parse_phase2(&ir0);
        let ir2 = self.parse_phase3(&ir1);
        let ast = self.parse_phase4(&ir2);
        ast
    }
}
