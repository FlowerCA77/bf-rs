use std::fmt;

use miette::highlighters::{Highlighter, HighlighterState};
use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme, NamedSource, Report, SourceSpan, SpanContents};
use owo_colors::{Style, Styled};
use thiserror::Error;
use tracing::Level;
use tracing_subscriber::EnvFilter;

const ANSI_RESET: &str = "\x1b[0m";
const DIAG_TOP: &str = "==================== DIAGNOSTIC BEGIN ====================";
const DIAG_BOTTOM: &str = "===================== DIAGNOSTIC END =====================";

#[derive(Debug, Clone, Copy)]
struct LogLevelMeta {
    label: &'static str,
    prefix: char,
    tracing_level: Level,
    env_filter: &'static str,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Panic,
    FatalError,
    Error,
    Warning,
    Info,
    Verbose,
    Debug,
}

impl LogLevel {
    const fn meta(self) -> LogLevelMeta {
        match self {
            LogLevel::Panic => LogLevelMeta {
                label: "PANIC",
                prefix: 'P',
                tracing_level: Level::ERROR,
                env_filter: "error",
            },
            LogLevel::FatalError => LogLevelMeta {
                label: "FATAL_ERROR",
                prefix: 'F',
                tracing_level: Level::ERROR,
                env_filter: "error",
            },
            LogLevel::Error => LogLevelMeta {
                label: "ERROR",
                prefix: 'E',
                tracing_level: Level::ERROR,
                env_filter: "error",
            },
            LogLevel::Warning => LogLevelMeta {
                label: "WARNING",
                prefix: 'W',
                tracing_level: Level::WARN,
                env_filter: "warn",
            },
            LogLevel::Info => LogLevelMeta {
                label: "INFO",
                prefix: 'I',
                tracing_level: Level::INFO,
                env_filter: "info",
            },
            LogLevel::Verbose => LogLevelMeta {
                label: "VERBOSE",
                prefix: 'V',
                tracing_level: Level::DEBUG,
                env_filter: "debug",
            },
            LogLevel::Debug => LogLevelMeta {
                label: "DEBUG",
                prefix: 'D',
                tracing_level: Level::DEBUG,
                env_filter: "debug",
            },
        }
    }

    pub fn as_str(self) -> &'static str {
        self.meta().label
    }

    pub fn code_prefix(self) -> char {
        self.meta().prefix
    }

    fn class_index(self) -> u16 {
        self as u16
    }

    pub fn severity_rank(self) -> u8 {
        self as u8
    }

    pub fn enabled_with_threshold(self, threshold: LogLevel) -> bool {
        self.severity_rank() <= threshold.severity_rank()
    }

    pub fn parse_case_insensitive(s: &str) -> Option<LogLevel> {
        let normalized = s.trim().to_ascii_uppercase();
        match normalized.as_str() {
            "PANIC" => Some(LogLevel::Panic),
            "FATAL_ERROR" | "FATAL" => Some(LogLevel::FatalError),
            "ERROR" => Some(LogLevel::Error),
            "WARNING" | "WARN" => Some(LogLevel::Warning),
            "INFO" => Some(LogLevel::Info),
            "VERBOSE" => Some(LogLevel::Verbose),
            "DEBUG" => Some(LogLevel::Debug),
            _ => None,
        }
    }

    fn to_tracing_level(self) -> Level {
        self.meta().tracing_level
    }

    fn to_env_filter(self) -> &'static str {
        self.meta().env_filter
    }

    fn terminal_ansi_color(self) -> &'static str {
        match self.to_tracing_level() {
            Level::ERROR => "\x1b[31m",
            Level::WARN => "\x1b[33m",
            Level::INFO => "\x1b[34m",
            Level::DEBUG | Level::TRACE => "\x1b[90m",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLoc {
    Lexer = 1,
    Parser = 2,
    Ir1 = 3,
    Ir2 = 4,
    Bfvm = 5,
    Driver = 6,
    Runtime = 7,
    Unknown = 255,
}

impl LogLoc {
    pub fn as_str(self) -> &'static str {
        match self {
            LogLoc::Lexer => "LEXER",
            LogLoc::Parser => "PARSER",
            LogLoc::Ir1 => "IR1",
            LogLoc::Ir2 => "IR2",
            LogLoc::Bfvm => "BFVM",
            LogLoc::Driver => "DRIVER",
            LogLoc::Runtime => "RUNTIME",
            LogLoc::Unknown => "UNKNOWN",
        }
    }

    pub fn from_stage(stage: &str) -> LogLoc {
        match stage {
            "LEXER" => LogLoc::Lexer,
            "PARSER" => LogLoc::Parser,
            "IR1" => LogLoc::Ir1,
            "IR2" => LogLoc::Ir2,
            "BFVM" => LogLoc::Bfvm,
            "DRIVER" => LogLoc::Driver,
            "RUNTIME" => LogLoc::Runtime,
            _ => LogLoc::Unknown,
        }
    }
}

impl fmt::Display for LogLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

pub fn log_no(level: LogLevel, code: u16) -> u16 {
    // FFI stable mapping: top 3 bits store level class, low 13 bits store code number.
    ((level.class_index() & 0x0007) << 13) | (code & 0x1fff)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticDescriptor {
    pub loc: LogLoc,
    pub level: LogLevel,
    pub code: u16,
    pub readable_code: &'static str,
}

impl DiagnosticDescriptor {
    pub const fn new(
        loc: LogLoc,
        level: LogLevel,
        code: u16,
        readable_code: &'static str,
    ) -> Self {
        Self {
            loc,
            level,
            code,
            readable_code,
        }
    }

    pub const fn error(loc: LogLoc, code: u16, readable_code: &'static str) -> Self {
        Self::new(loc, LogLevel::Error, code, readable_code)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    pub loc: LogLoc,
    pub level: LogLevel,
    pub code: Option<u16>,
    pub readable_code: String,
    pub msg: String,
}

impl LogRecord {
    pub fn stage(&self) -> &'static str {
        self.loc.as_str()
    }

    pub fn ffi_no(&self) -> Option<u16> {
        self.code.map(|v| log_no(self.level, v))
    }
}

pub trait DiagnosticError: fmt::Display {
    fn descriptor(&self) -> DiagnosticDescriptor;

    fn detail(&self) -> String {
        self.to_string()
    }

    fn log_loc(&self) -> LogLoc {
        self.descriptor().loc
    }

    fn level(&self) -> LogLevel {
        self.descriptor().level
    }

    fn code(&self) -> u16 {
        self.descriptor().code
    }

    fn readable_code(&self) -> &'static str {
        self.descriptor().readable_code
    }

    fn stage(&self) -> &'static str {
        self.log_loc().as_str()
    }

    fn as_log_record(&self) -> LogRecord {
        let descriptor = self.descriptor();
        LogRecord {
            loc: descriptor.loc,
            level: descriptor.level,
            code: Some(descriptor.code),
            readable_code: descriptor.readable_code.to_string(),
            msg: self.detail(),
        }
    }

    fn as_log_line(&self) -> String {
        format!(
            "[{} @ {}] {}: {}",
            self.level(),
            self.stage(),
            self.readable_code(),
            self.detail()
        )
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
#[diagnostic()]
struct SourceSnippetDiagnostic {
    message: String,
    #[source_code]
    src: NamedSource<String>,
    #[label("here")]
    span: SourceSpan,
}

#[derive(Debug, Clone, Copy)]
struct BrainfuckHighlighter {
    focus_line: usize,
    focus_column: usize,
}

#[derive(Debug, Clone, Copy)]
struct BrainfuckHighlighterState {
    focus_line: usize,
    focus_column: usize,
    snippet_start_line: usize,
    current_line: usize,
}

impl Highlighter for BrainfuckHighlighter {
    fn start_highlighter_state<'h>(
        &'h self,
        source: &dyn SpanContents<'_>,
    ) -> Box<dyn HighlighterState + 'h> {
        Box::new(BrainfuckHighlighterState {
            focus_line: self.focus_line,
            focus_column: self.focus_column,
            snippet_start_line: source.line() + 1,
            current_line: 1,
        })
    }
}

impl HighlighterState for BrainfuckHighlighterState {
    fn highlight_line<'s>(&mut self, line: &'s str) -> Vec<Styled<&'s str>> {
        let mut out = Vec::new();
        let mut col = 1usize;
        let mut chars = line.char_indices().peekable();

        while let Some((start, ch)) = chars.next() {
            let end = chars.peek().map(|(idx, _)| *idx).unwrap_or(line.len());
            let segment = &line[start..end];
            let absolute_line = self.snippet_start_line + self.current_line - 1;

            let style = if absolute_line == self.focus_line && col == self.focus_column {
                Style::new().red().bold()
            } else {
                match ch {
                    '>' | '<' => Style::new().cyan(),
                    '+' | '-' => Style::new().green(),
                    '[' | ']' => Style::new().yellow(),
                    '.' | ',' => Style::new().magenta(),
                    _ => Style::new(),
                }
            };

            out.push(style.style(segment));
            col += 1;
        }

        self.current_line += 1;
        out
    }
}

pub fn render_brainfuck_parse_diagnostic(
    message: &str,
    source: &str,
    line: usize,
    column: usize,
) -> Option<String> {
    let offset = line_col_to_byte_offset(source, line, column)?;
    let span_len = source[offset..]
        .chars()
        .next()
        .map(|ch| ch.len_utf8())
        .unwrap_or(0);

    let diagnostic = SourceSnippetDiagnostic {
        message: message.to_string(),
        src: NamedSource::new("<raw-bf.bf>", source.to_string()).with_language("brainfuck"),
        span: (offset, span_len).into(),
    };

    let mut out = String::new();
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode())
        .with_syntax_highlighting(BrainfuckHighlighter {
            focus_line: line,
            focus_column: column,
        });
    if handler
        .render_report(&mut out, Report::new(diagnostic).as_ref())
        .is_err()
    {
        return None;
    }

    Some(format!("{}\n{}\n{}", DIAG_TOP, out.trim_end(), DIAG_BOTTOM))
}

fn line_col_to_byte_offset(source: &str, line: usize, column: usize) -> Option<usize> {
    let mut cur_line = 1usize;
    let mut cur_col = 1usize;

    for (idx, ch) in source.char_indices() {
        if cur_line == line && cur_col == column {
            return Some(idx);
        }

        if ch == '\n' {
            cur_line += 1;
            cur_col = 1;
        } else {
            cur_col += 1;
        }
    }

    if cur_line == line && cur_col == column {
        return Some(source.len());
    }

    None
}

#[derive(Debug, Clone)]
pub struct Logger {
    threshold: LogLevel,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            threshold: LogLevel::Info,
        }
    }
}

impl Logger {
    pub fn init_global_subscriber(default_level: LogLevel) {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(default_level.to_env_filter()));

        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(true)
            .without_time()
            .with_target(false)
            .with_level(false)
            .with_writer(std::io::stdout)
            .try_init();
    }

    pub fn new(threshold: LogLevel) -> Self {
        Self { threshold }
    }

    pub fn threshold(&self) -> LogLevel {
        self.threshold
    }

    pub fn set_threshold(&mut self, threshold: LogLevel) {
        self.threshold = threshold;
    }

    pub fn render_record(&self, record: &LogRecord) -> String {
        let id_col = match record.code {
            Some(id) => format!("{}{:04}", record.level.code_prefix(), id),
            None => String::from("------"),
        };

        format!(
            "{:<11} | {:<8} | {:<6} | {:<36} | {}",
            record.level,
            record.stage(),
            id_col,
            &record.readable_code,
            &record.msg
        )
    }

    fn render_record_for_terminal(&self, record: &LogRecord) -> String {
        let plain = self.render_record(record);
        let color = record.level.terminal_ansi_color();
        if let Some((first_line, rest)) = plain.split_once('\n') {
            format!("{}{}{}\n{}", color, first_line, ANSI_RESET, rest)
        } else {
            format!("{}{}{}", color, plain, ANSI_RESET)
        }
    }

    fn derive_raw_code(readable_code: &str) -> u16 {
        let mut hash: u32 = 2_166_136_261;
        for b in readable_code.as_bytes() {
            hash ^= *b as u32;
            hash = hash.wrapping_mul(16_777_619);
        }
        ((hash % 9000) as u16) + 1000
    }

    pub fn record_raw_with_id(
        &self,
        level: LogLevel,
        stage: &str,
        code: Option<u16>,
        readable_code: &str,
        msg: &str,
    ) -> Option<LogRecord> {
        if !level.enabled_with_threshold(self.threshold) {
            return None;
        }

        let code = code.or(Some(Self::derive_raw_code(readable_code)));

        Some(LogRecord {
            loc: LogLoc::from_stage(stage),
            level,
            code,
            readable_code: readable_code.to_string(),
            msg: msg.to_string(),
        })
    }

    pub fn record_raw(
        &self,
        level: LogLevel,
        stage: &str,
        readable_code: &str,
        msg: &str,
    ) -> Option<LogRecord> {
        self.record_raw_with_id(level, stage, None, readable_code, msg)
    }

    pub fn record_error<E: DiagnosticError + ?Sized>(&self, err: &E) -> Option<LogRecord> {
        self.record_raw_with_id(
            err.level(),
            err.stage(),
            Some(err.code()),
            err.readable_code(),
            &err.detail(),
        )
    }

    pub fn record_unhandled<E: DiagnosticError + ?Sized>(&self, err: &E) -> Option<LogRecord> {
        self.record_raw_with_id(
            LogLevel::FatalError,
            err.stage(),
            Some(err.code()),
            err.readable_code(),
            &err.detail(),
        )
    }

    pub fn record_panic(&self, detail: &str) -> Option<LogRecord> {
        self.record_raw_with_id(LogLevel::Panic, "RUNTIME", None, "E_PANIC", detail)
    }

    pub fn emit_record(&self, record: &LogRecord) {
        let line = self.render_record_for_terminal(record);
        self.emit_line(record.level, &line);
    }

    pub fn emit_raw(&self, level: LogLevel, stage: &str, readable_code: &str, detail: &str) {
        if let Some(record) = self.record_raw(level, stage, readable_code, detail) {
            self.emit_record(&record);
        }
    }

    pub fn emit_error<E: DiagnosticError + ?Sized>(&self, err: &E) {
        if let Some(record) = self.record_error(err) {
            self.emit_record(&record);
        }
    }

    pub fn emit_unhandled<E: DiagnosticError + ?Sized>(&self, err: &E) {
        if let Some(record) = self.record_unhandled(err) {
            self.emit_record(&record);
        }
    }

    pub fn emit_panic(&self, detail: &str) {
        if let Some(record) = self.record_panic(detail) {
            self.emit_record(&record);
        }
    }

    fn emit_line(&self, level: LogLevel, line: &str) {
        let _ = level;
        println!("{}", line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RuntimeError;

    #[test]
    fn logger_filters_by_threshold() {
        let logger = Logger::new(LogLevel::Warning);
        assert!(logger
            .record_raw(LogLevel::Error, "BFVM", "E_X", "err")
            .is_some());
        assert!(logger
            .record_raw(LogLevel::Debug, "BFVM", "D_X", "debug")
            .is_none());
    }

    #[test]
    fn logger_renders_runtime_error_record() {
        let logger = Logger::new(LogLevel::Debug);
        let err = RuntimeError::EntryFunctionNotFound {
            name: String::from("entry"),
        };
        let record = logger.record_error(&err).unwrap();
        let line = logger.render_record(&record);
        assert!(line.starts_with("ERROR"));
        assert!(line.contains("| BFVM"));
        assert!(line.contains("| E4303"));
        assert!(line.contains("E_FUNCTION_NOT_EXISTS"));
        assert!(line.ends_with("entry function 'entry' not found"));
    }

    #[test]
    fn logger_promotes_unhandled_to_fatal() {
        let logger = Logger::new(LogLevel::Debug);
        let err = RuntimeError::EntryFunctionNotFound {
            name: String::from("entry"),
        };
        let record = logger.record_unhandled(&err).unwrap();
        let line = logger.render_record(&record);
        assert!(line.starts_with("FATAL_ERROR"));
        assert!(line.contains("| BFVM"));
        assert!(line.contains("| F4303"));
        assert!(line.contains("E_FUNCTION_NOT_EXISTS"));
        assert!(line.ends_with("entry function 'entry' not found"));
    }

    #[test]
    fn logger_formats_panic_level() {
        let logger = Logger::new(LogLevel::Debug);
        let record = logger.record_panic("panic payload").unwrap();
        let line = logger.render_record(&record);
        assert!(line.starts_with("PANIC"));
        assert!(line.contains("| RUNTIME"));
        assert!(line.contains("| P"));
        assert!(line.contains("E_PANIC"));
        assert!(line.ends_with("panic payload"));
    }

    #[test]
    fn logger_assigns_numeric_code_to_raw_record() {
        let logger = Logger::new(LogLevel::Debug);
        let record = logger
            .record_raw(LogLevel::Info, "DRIVER", "I_PIPELINE_START", "start")
            .unwrap();
        assert!(record.code.is_some());
        let line = logger.render_record(&record);
        assert!(line.contains("| I"));
    }

    #[test]
    fn logger_builds_structured_record_for_error() {
        let logger = Logger::new(LogLevel::Debug);
        let err = RuntimeError::EntryFunctionNotFound {
            name: String::from("entry"),
        };

        let record = logger.record_error(&err).unwrap();
        assert_eq!(record.loc, LogLoc::Bfvm);
        assert_eq!(record.level, LogLevel::Error);
        assert_eq!(record.code, Some(4303));
        assert_eq!(record.readable_code, "E_FUNCTION_NOT_EXISTS");
        assert_eq!(record.ffi_no(), Some(log_no(LogLevel::Error, 4303)));
    }

    #[test]
    fn logger_respects_verbose_threshold() {
        let logger = Logger::new(LogLevel::Verbose);
        assert!(logger
            .record_raw(LogLevel::Verbose, "BFVM", "V_EVT", "trace")
            .is_some());
        assert!(logger
            .record_raw(LogLevel::Debug, "BFVM", "D_EVT", "deep")
            .is_none());
    }

    #[test]
    fn log_level_alias_parsing_is_available() {
        assert_eq!(
            LogLevel::parse_case_insensitive("warn"),
            Some(LogLevel::Warning)
        );
        assert_eq!(
            LogLevel::parse_case_insensitive("fatal"),
            Some(LogLevel::FatalError)
        );
    }

    #[test]
    fn log_no_is_stable_for_ffi() {
        assert_eq!(log_no(LogLevel::Error, 1002), ((2u16) << 13) | 1002u16);
        assert_eq!(log_no(LogLevel::FatalError, 4303), ((1u16) << 13) | 4303u16);
    }

    #[test]
    fn diagnostic_renderer_contains_wrapped_highlight() {
        let rendered = render_brainfuck_parse_diagnostic("oops", "\n[++\n", 2, 1).unwrap();
        assert!(rendered.contains("DIAGNOSTIC BEGIN"));
        assert!(rendered.contains("DIAGNOSTIC END"));
        assert!(!rendered.contains("bf-highlight"));
    }
}
