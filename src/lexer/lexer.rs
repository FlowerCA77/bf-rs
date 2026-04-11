use crate::{LogLevel, Logger};

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenWithPos {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {}

impl Lexer {
    pub fn run(code: &String) -> Vec<Token> {
        Self::run_with_logger(code.as_str(), None)
    }

    pub fn run_with_logger(code: &str, logger: Option<&Logger>) -> Vec<Token> {
        if let Some(logger) = logger {
            logger.emit_raw(
                LogLevel::Info,
                "LEXER",
                "I_LEXER_START",
                &format!("input_chars={}", code.chars().count()),
            );
        }

        let mut tokens = Vec::new();
        let mut filtered_comments = 0usize;

        for ch in code.chars() {
            let tk = match ch {
                '>' => Token::MOVR,
                '<' => Token::MOVL,
                '+' => Token::INC,
                '-' => Token::DEC,
                '.' => Token::OUTPUT,
                ',' => Token::INPUT,
                '[' => Token::JMPIN,
                ']' => Token::JMPOUT,
                _ => Token::COMMENT,
            };

            if matches!(tk, Token::COMMENT) {
                filtered_comments += 1;
                continue;
            }

            tokens.push(tk);
        }

        if let Some(logger) = logger {
            logger.emit_raw(
                LogLevel::Debug,
                "LEXER",
                "D_LEXER_DONE",
                &format!(
                    "tokens={} filtered_comments={}",
                    tokens.len(), filtered_comments
                ),
            );
        }

        tokens
    }

    pub fn run_with_positions(code: &str, logger: Option<&Logger>) -> Vec<TokenWithPos> {
        if let Some(logger) = logger {
            logger.emit_raw(
                LogLevel::Info,
                "LEXER",
                "I_LEXER_START",
                &format!("input_chars={}", code.chars().count()),
            );
        }

        let mut tokens = Vec::new();
        let mut filtered_comments = 0usize;
        let mut line = 1usize;
        let mut column = 1usize;

        for ch in code.chars() {
            let tk = match ch {
                '>' => Token::MOVR,
                '<' => Token::MOVL,
                '+' => Token::INC,
                '-' => Token::DEC,
                '.' => Token::OUTPUT,
                ',' => Token::INPUT,
                '[' => Token::JMPIN,
                ']' => Token::JMPOUT,
                _ => Token::COMMENT,
            };

            if matches!(tk, Token::COMMENT) {
                filtered_comments += 1;
            } else {
                tokens.push(TokenWithPos {
                    token: tk,
                    line,
                    column,
                });
            }

            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        if let Some(logger) = logger {
            logger.emit_raw(
                LogLevel::Debug,
                "LEXER",
                "D_LEXER_DONE",
                &format!(
                    "tokens={} filtered_comments={}",
                    tokens.len(), filtered_comments
                ),
            );
        }

        tokens
    }
}

#[cfg(test)]
#[path = "test_lexer.rs"]
mod tests;
