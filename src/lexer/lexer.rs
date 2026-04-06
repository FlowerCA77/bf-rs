#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    MOVR,
    MOVL,
    INC,
    DEC,
    OUTPUT,
    INPUT,
    JMPIN(i64),
    JMPOUT(i64),
    COMMENT,
}

pub struct Lexer {
    pub depth: i64,
}

impl Lexer {
    pub fn run(&mut self, code: &String) -> Vec<Token> {
        let stream = code.chars();
        stream
            .map(|ch| match ch {
                '>' => Token::MOVR,
                '<' => Token::MOVL,
                '+' => Token::INC,
                '-' => Token::DEC,
                '.' => Token::OUTPUT,
                ',' => Token::INPUT,
                '[' => {
                    self.depth += 1;
                    Token::JMPIN(self.depth - 1)
                }
                ']' => {
                    self.depth -= 1;
                    Token::JMPOUT(self.depth + 1)
                }
                _ => Token::COMMENT,
            })
            .filter(|tk| !matches!(tk, &Token::COMMENT))
            .collect()
    }

    pub fn reset(&mut self) {
        self.depth = 0i64;
    }
}
