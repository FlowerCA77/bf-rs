#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    MOVR,
    MOVL,
    INC,
    DEC,
    OUTPUT,
    INPUT,
    JMPIN,
    JMPOUT,
    COMMENT,
}

pub struct Lexer {}

impl Lexer {
    pub fn run(code: &String) -> Vec<Token> {
        let stream = code.chars();
        stream
            .map(|ch| match ch {
                '>' => Token::MOVR,
                '<' => Token::MOVL,
                '+' => Token::INC,
                '-' => Token::DEC,
                '.' => Token::OUTPUT,
                ',' => Token::INPUT,
                '[' => Token::JMPIN,
                ']' => Token::JMPOUT,
                _ => Token::COMMENT,
            })
            .filter(|tk| !matches!(tk, &Token::COMMENT))
            .collect()
    }
}
