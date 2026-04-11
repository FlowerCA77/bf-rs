use std::error::Error;
use std::fmt;
use std::path::PathBuf;

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
pub enum Ir1Error {
    UnexpectedBracketInRun,
    ParseInvalidHeader { found: String },
    ParseUnexpectedLoopEnd { line: usize },
    ParseUnclosedLoop,
    ParseInvalidInstruction { line: usize, content: String },
    ParseInvalidOperand { line: usize, content: String },
    Io { path: PathBuf, message: String },
}

impl fmt::Display for Ir1Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ir1Error::UnexpectedBracketInRun => {
                write!(f, "Unexpected bracket token appeared in a Run node")
            }
            Ir1Error::ParseInvalidHeader { found } => {
                write!(f, "Invalid BF1 header. Expected 'BF1', found '{}'", found)
            }
            Ir1Error::ParseUnexpectedLoopEnd { line } => {
                write!(f, "Unexpected LOOP_END at line {}", line)
            }
            Ir1Error::ParseUnclosedLoop => write!(f, "Missing LOOP_END for LOOP_BEGIN"),
            Ir1Error::ParseInvalidInstruction { line, content } => {
                write!(f, "Invalid BF1 instruction at line {}: {}", line, content)
            }
            Ir1Error::ParseInvalidOperand { line, content } => {
                write!(f, "Invalid BF1 operand at line {}: {}", line, content)
            }
            Ir1Error::Io { path, message } => {
                write!(f, "I/O error at '{}': {}", path.display(), message)
            }
        }
    }
}

impl Error for Ir1Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ir2Error {
    InvalidCurrentBlock { id: usize },
    TerminatorAlreadySet { block_id: usize },
    ParseInvalidHeader { found: String },
    ParseInvalidFunctionHeader { line: usize, content: String },
    ParseInvalidBlockHeader { line: usize, content: String },
    ParseInvalidInstruction { line: usize, content: String },
    ParseInvalidOperand { line: usize, content: String },
    ParseMissingTerminator { function: String, block_id: usize },
    ParseMissingEndFunc { function: String },
    EntryBlockNotFound { function: String, entry: usize },
    Io { path: PathBuf, message: String },
}

impl fmt::Display for Ir2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ir2Error::InvalidCurrentBlock { id } => {
                write!(f, "Invalid current block id: {}", id)
            }
            Ir2Error::TerminatorAlreadySet { block_id } => {
                write!(f, "Block {} already has a terminator", block_id)
            }
            Ir2Error::ParseInvalidHeader { found } => {
                write!(f, "Invalid BF2 header. Expected 'BF2', found '{}'", found)
            }
            Ir2Error::ParseInvalidFunctionHeader { line, content } => {
                write!(
                    f,
                    "Invalid BF2 function header at line {}: {}",
                    line, content
                )
            }
            Ir2Error::ParseInvalidBlockHeader { line, content } => {
                write!(f, "Invalid BF2 block header at line {}: {}", line, content)
            }
            Ir2Error::ParseInvalidInstruction { line, content } => {
                write!(f, "Invalid BF2 instruction at line {}: {}", line, content)
            }
            Ir2Error::ParseInvalidOperand { line, content } => {
                write!(f, "Invalid BF2 operand at line {}: {}", line, content)
            }
            Ir2Error::ParseMissingTerminator { function, block_id } => {
                write!(
                    f,
                    "Missing terminator in function '{}' block {}",
                    function, block_id
                )
            }
            Ir2Error::ParseMissingEndFunc { function } => {
                write!(f, "Missing END_FUNC for function '{}'", function)
            }
            Ir2Error::EntryBlockNotFound { function, entry } => {
                write!(
                    f,
                    "Function '{}' has entry block {} that is not defined",
                    function, entry
                )
            }
            Ir2Error::Io { path, message } => {
                write!(f, "I/O error at '{}': {}", path.display(), message)
            }
        }
    }
}

impl Error for Ir2Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BfError {
    Parse(ParseError),
    Ir1(Ir1Error),
    Ir2(Ir2Error),
}

impl fmt::Display for BfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BfError::Parse(err) => write!(f, "Parse error: {}", err),
            BfError::Ir1(err) => write!(f, "IR1 error: {}", err),
            BfError::Ir2(err) => write!(f, "IR2 error: {}", err),
        }
    }
}

impl Error for BfError {}

impl From<ParseError> for BfError {
    fn from(value: ParseError) -> Self {
        BfError::Parse(value)
    }
}

impl From<Ir1Error> for BfError {
    fn from(value: Ir1Error) -> Self {
        BfError::Ir1(value)
    }
}

impl From<Ir2Error> for BfError {
    fn from(value: Ir2Error) -> Self {
        BfError::Ir2(value)
    }
}
