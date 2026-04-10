use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedRightBracket { pos: usize },
    UnclosedLeftBracket,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedRightBracket { pos } => {
                write!(f, "Unexpected ']' at position {}", pos)
            }
            ParseError::UnclosedLeftBracket => {
                write!(f, "Unclosed '['. Reached EOF before finding matching ']'")
            }
        }
    }
}

impl Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    UnexpectedBracketInRun,
    Overflow,
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LowerError::UnexpectedBracketInRun => {
                write!(f, "Unexpected bracket token appeared in a Run node")
            }
            LowerError::Overflow => write!(f, "Integer overflow while lowering to IR1"),
        }
    }
}

impl Error for LowerError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BfError {
    Parse(ParseError),
    Lower(LowerError),
}

impl fmt::Display for BfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BfError::Parse(err) => write!(f, "Parse error: {}", err),
            BfError::Lower(err) => write!(f, "Lowering error: {}", err),
        }
    }
}

impl Error for BfError {}

impl From<ParseError> for BfError {
    fn from(value: ParseError) -> Self {
        BfError::Parse(value)
    }
}

impl From<LowerError> for BfError {
    fn from(value: LowerError) -> Self {
        BfError::Lower(value)
    }
}
