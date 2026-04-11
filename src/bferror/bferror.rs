use crate::logging::logging::{DiagnosticDescriptor, DiagnosticError, LogLoc};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("Unexpected ']' at token index {pos}")]
    UnexpectedRightBracket { pos: usize },
    #[error("Unclosed '[' started at token index {pos}. Reached EOF before finding matching ']'")]
    UnclosedLeftBracket { pos: usize },
}

impl DiagnosticError for ParseError {
    fn descriptor(&self) -> DiagnosticDescriptor {
        match self {
            ParseError::UnexpectedRightBracket { .. } => {
                DiagnosticDescriptor::error(LogLoc::Parser, 1001, "E_PARSE_UNEXPECTED_RIGHT_BRACKET")
            }
            ParseError::UnclosedLeftBracket { .. } => {
                DiagnosticDescriptor::error(LogLoc::Parser, 1002, "E_PARSE_UNCLOSED_LEFT_BRACKET")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Ir1Error {
    #[error("Unexpected bracket token appeared in a Run node")]
    UnexpectedBracketInRun,
    #[error("Invalid BF1 header. Expected 'BF1', found '{found}'")]
    ParseInvalidHeader { found: String },
    #[error("Unexpected LOOP_END at line {line}")]
    ParseUnexpectedLoopEnd { line: usize },
    #[error("Missing LOOP_END for LOOP_BEGIN")]
    ParseUnclosedLoop,
    #[error("Invalid BF1 instruction at line {line}: {content}")]
    ParseInvalidInstruction { line: usize, content: String },
    #[error("Invalid BF1 operand at line {line}: {content}")]
    ParseInvalidOperand { line: usize, content: String },
    #[error("I/O error at '{path}': {message}", path = .path.display())]
    Io { path: PathBuf, message: String },
}

impl DiagnosticError for Ir1Error {
    fn descriptor(&self) -> DiagnosticDescriptor {
        match self {
            Ir1Error::UnexpectedBracketInRun => {
                DiagnosticDescriptor::error(LogLoc::Ir1, 2001, "E_IR1_UNEXPECTED_BRACKET_IN_RUN")
            }
            Ir1Error::ParseInvalidHeader { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir1, 2101, "E_IR1_PARSE_INVALID_HEADER")
            }
            Ir1Error::ParseUnexpectedLoopEnd { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir1, 2102, "E_IR1_PARSE_UNEXPECTED_LOOP_END")
            }
            Ir1Error::ParseUnclosedLoop => {
                DiagnosticDescriptor::error(LogLoc::Ir1, 2103, "E_IR1_PARSE_UNCLOSED_LOOP")
            }
            Ir1Error::ParseInvalidInstruction { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir1, 2104, "E_IR1_PARSE_INVALID_INSTRUCTION")
            }
            Ir1Error::ParseInvalidOperand { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir1, 2105, "E_IR1_PARSE_INVALID_OPERAND")
            }
            Ir1Error::Io { .. } => DiagnosticDescriptor::error(LogLoc::Ir1, 2201, "E_IR1_IO"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Ir2Error {
    #[error("Invalid current block id: {id}")]
    InvalidCurrentBlock { id: usize },
    #[error("Block {block_id} already has a terminator")]
    TerminatorAlreadySet { block_id: usize },
    #[error("Invalid BF2 header. Expected 'BF2', found '{found}'")]
    ParseInvalidHeader { found: String },
    #[error("Invalid BF2 function header at line {line}: {content}")]
    ParseInvalidFunctionHeader { line: usize, content: String },
    #[error("Invalid BF2 block header at line {line}: {content}")]
    ParseInvalidBlockHeader { line: usize, content: String },
    #[error("Invalid BF2 instruction at line {line}: {content}")]
    ParseInvalidInstruction { line: usize, content: String },
    #[error("Invalid BF2 operand at line {line}: {content}")]
    ParseInvalidOperand { line: usize, content: String },
    #[error("Missing terminator in function '{function}' block {block_id}")]
    ParseMissingTerminator { function: String, block_id: usize },
    #[error("Missing END_FUNC for function '{function}'")]
    ParseMissingEndFunc { function: String },
    #[error("Function '{function}' has entry block {entry} that is not defined")]
    EntryBlockNotFound { function: String, entry: usize },
    #[error("I/O error at '{path}': {message}", path = .path.display())]
    Io { path: PathBuf, message: String },
}

impl DiagnosticError for Ir2Error {
    fn descriptor(&self) -> DiagnosticDescriptor {
        match self {
            Ir2Error::InvalidCurrentBlock { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir2, 3001, "E_IR2_INVALID_CURRENT_BLOCK")
            }
            Ir2Error::TerminatorAlreadySet { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir2, 3002, "E_IR2_TERMINATOR_ALREADY_SET")
            }
            Ir2Error::ParseInvalidHeader { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir2, 3101, "E_IR2_PARSE_INVALID_HEADER")
            }
            Ir2Error::ParseInvalidFunctionHeader { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir2, 3102, "E_IR2_PARSE_INVALID_FUNCTION_HEADER")
            }
            Ir2Error::ParseInvalidBlockHeader { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir2, 3103, "E_IR2_PARSE_INVALID_BLOCK_HEADER")
            }
            Ir2Error::ParseInvalidInstruction { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir2, 3104, "E_IR2_PARSE_INVALID_INSTRUCTION")
            }
            Ir2Error::ParseInvalidOperand { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir2, 3105, "E_IR2_PARSE_INVALID_OPERAND")
            }
            Ir2Error::ParseMissingTerminator { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir2, 3106, "E_IR2_PARSE_MISSING_TERMINATOR")
            }
            Ir2Error::ParseMissingEndFunc { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir2, 3107, "E_IR2_PARSE_MISSING_END_FUNC")
            }
            Ir2Error::EntryBlockNotFound { .. } => {
                DiagnosticDescriptor::error(LogLoc::Ir2, 3108, "E_IR2_ENTRY_BLOCK_NOT_FOUND")
            }
            Ir2Error::Io { .. } => DiagnosticDescriptor::error(LogLoc::Ir2, 3201, "E_IR2_IO"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuntimeError {
    #[error("requested artifact kind '{kind}' is not supported")]
    ArtifactKindUnsupported {
        kind: String,
    },
    #[error("ptr={current} delta={delta} attempted={attempted} valid=[{min}, {max}]")]
    PtrOutOfBounds {
        current: i64,
        delta: i64,
        attempted: i128,
        min: i128,
        max: i128,
    },
    #[error("call to '{function}' is not supported in V1")]
    CallUnsupported {
        function: String,
    },
    #[error("{operation} failed ({message})")]
    Io {
        operation: String,
        message: String,
    },
    #[error("function '{function}' has duplicate block id {block_id}")]
    DuplicateBlockId {
        function: String,
        block_id: usize,
    },
    #[error("multiple entry functions named '{name}'")]
    DuplicateEntryFunction {
        name: String,
    },
    #[error("entry function '{name}' not found")]
    EntryFunctionNotFound {
        name: String,
    },
    #[error("function '{function}' has missing entry block {entry}")]
    EntryBlockNotFound {
        function: String,
        entry: usize,
    },
    #[error("function '{function}' references unknown block {block_id}")]
    UnknownBlockId {
        function: String,
        block_id: usize,
    },
    #[error("function '{function}' block {block_id} is missing terminator")]
    MissingTerminator {
        function: String,
        block_id: usize,
    },
    #[error("{source}", source = .0.as_log_line())]
    InvalidProgramText(#[from] Ir2Error),
}

impl DiagnosticError for RuntimeError {
    fn descriptor(&self) -> DiagnosticDescriptor {
        match self {
            RuntimeError::ArtifactKindUnsupported { .. } => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4001, "E_ARTIFACT_KIND_UNSUPPORTED")
            }
            RuntimeError::PtrOutOfBounds { .. } => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4002, "E_PTR_OUT_OF_BOUNDS")
            }
            RuntimeError::CallUnsupported { .. } => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4003, "E_CALL_UNSUPPORTED")
            }
            RuntimeError::Io { .. } => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4201, "E_BROKEN_IOSTREAM")
            }
            RuntimeError::DuplicateBlockId { .. } => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4301, "E_CONFLICT_BLOCKS")
            }
            RuntimeError::DuplicateEntryFunction { .. } => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4302, "E_CONFLICT_FUNCTIONS")
            }
            RuntimeError::EntryFunctionNotFound { .. } => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4303, "E_FUNCTION_NOT_EXISTS")
            }
            RuntimeError::EntryBlockNotFound { .. } => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4304, "E_ENTRY_BLOCK_NOT_FOUND")
            }
            RuntimeError::UnknownBlockId { .. } => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4305, "E_UNKNOWN_BLOCK_ID")
            }
            RuntimeError::MissingTerminator { .. } => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4306, "E_MISSING_TERMINATOR")
            }
            RuntimeError::InvalidProgramText(_) => {
                DiagnosticDescriptor::error(LogLoc::Bfvm, 4307, "E_BAD_CODE")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BfError {
    #[error("{0}")]
    Parse(#[from] ParseError),
    #[error("{0}")]
    Ir1(#[from] Ir1Error),
    #[error("{0}")]
    Ir2(#[from] Ir2Error),
    #[error("{0}")]
    Runtime(#[from] RuntimeError),
}

impl DiagnosticError for BfError {
    fn descriptor(&self) -> DiagnosticDescriptor {
        match self {
            BfError::Parse(err) => err.descriptor(),
            BfError::Ir1(err) => err.descriptor(),
            BfError::Ir2(err) => err.descriptor(),
            BfError::Runtime(err) => err.descriptor(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LogLevel;

    #[test]
    fn log_level_filtering_works() {
        assert!(LogLevel::Error.enabled_with_threshold(LogLevel::Info));
        assert!(!LogLevel::Debug.enabled_with_threshold(LogLevel::Info));
        assert!(LogLevel::Panic.enabled_with_threshold(LogLevel::Warning));
    }

    #[test]
    fn parse_log_level_case_insensitive() {
        assert_eq!(
            LogLevel::parse_case_insensitive("fatal_error"),
            Some(LogLevel::FatalError)
        );
        assert_eq!(
            LogLevel::parse_case_insensitive("Warn"),
            Some(LogLevel::Warning)
        );
        assert_eq!(LogLevel::parse_case_insensitive("nope"), None);
    }

    #[test]
    fn runtime_error_renders_with_stage_and_code() {
        let err = RuntimeError::ArtifactKindUnsupported {
            kind: String::from("dynamic-lib"),
        };
        assert_eq!(
            err.as_log_line(),
            "[ERROR @ BFVM] E_ARTIFACT_KIND_UNSUPPORTED: requested artifact kind 'dynamic-lib' is not supported"
        );
    }

    #[test]
    fn bferror_wrap_preserves_runtime_diagnostic_format() {
        let err: BfError = RuntimeError::EntryFunctionNotFound {
            name: String::from("entry"),
        }
        .into();
        assert_eq!(
            err.as_log_line(),
            "[ERROR @ BFVM] E_FUNCTION_NOT_EXISTS: entry function 'entry' not found"
        );
    }

    #[test]
    fn parse_error_renders_with_stage_and_code() {
        let err = ParseError::UnclosedLeftBracket { pos: 12 };
        assert_eq!(
            err.as_log_line(),
            "[ERROR @ PARSER] E_PARSE_UNCLOSED_LEFT_BRACKET: Unclosed '[' started at token index 12. Reached EOF before finding matching ']'"
        );
    }

    #[test]
    fn runtime_from_ir2_keeps_source_context() {
        let ir2 = Ir2Error::ParseInvalidHeader {
            found: String::from("NOPE"),
        };
        let runtime = RuntimeError::from(ir2);
        assert!(runtime
            .as_log_line()
            .contains("[ERROR @ BFVM] E_BAD_CODE: [ERROR @ IR2] E_IR2_PARSE_INVALID_HEADER"));
    }

    #[test]
    fn runtime_function_structure_errors_have_distinct_codes() {
        let entry_missing = RuntimeError::EntryBlockNotFound {
            function: String::from("entry"),
            entry: 0,
        };
        let unknown_block = RuntimeError::UnknownBlockId {
            function: String::from("entry"),
            block_id: 1,
        };
        let missing_term = RuntimeError::MissingTerminator {
            function: String::from("entry"),
            block_id: 2,
        };

        assert_eq!(entry_missing.code(), 4304);
        assert_eq!(unknown_block.code(), 4305);
        assert_eq!(missing_term.code(), 4306);
        assert_eq!(entry_missing.readable_code(), "E_ENTRY_BLOCK_NOT_FOUND");
        assert_eq!(unknown_block.readable_code(), "E_UNKNOWN_BLOCK_ID");
        assert_eq!(missing_term.readable_code(), "E_MISSING_TERMINATOR");
    }
}
