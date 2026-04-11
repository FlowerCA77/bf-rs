use bf_rs::{
    bfvm::bfvm::Status, BfError, DiagnosticError, Ir1Program, Ir2Program, Lexer, LogLevel,
    LogRecord, Logger, ParseError, Parser, Token, TokenWithPos, render_brainfuck_parse_diagnostic,
};
use std::{env, fs};

const OUT_DIR: &str = "artifacts";
const IR1_PATH: &str = "artifacts/pipeline_valid.bf1";
const IR2_PATH: &str = "artifacts/pipeline_valid.bf2";
const LOG_PATH: &str = "artifacts/pipeline.log";

/*
 * ====== THIS IS NOT THE ENTRY POINT FOR NOW! ======
 * ====== THE BINARY ENTRY IS NOT YET STARTED NOW! ======
 * ====== THIS CODE IS JUST FOR QUICK DEBUG! ======
 */
fn main() {
    let level = env::var("BFC_LOG_LEVEL")
        .ok()
        .and_then(|v| LogLevel::parse_case_insensitive(&v))
        .unwrap_or(LogLevel::Info);
    Logger::init_global_subscriber(level);
    let logger = Logger::new(level);

    let panic_logger = logger.clone();
    std::panic::set_hook(Box::new(move |info| {
        panic_logger.emit_panic(&info.to_string());
    }));

    let mut logs: Vec<LogRecord> = Vec::new();

    if let Err(e) = fs::create_dir_all(OUT_DIR) {
        logger.emit_raw(
            LogLevel::FatalError,
            "DRIVER",
            "E_LOG_INIT",
            &format!("failed to create '{}': {}", OUT_DIR, e),
        );
        return;
    }

    append_raw(
        &logger,
        &mut logs,
        LogLevel::Info,
        "DRIVER",
        "I_PIPELINE_START",
        "start staged pipeline with file IO and diagnostics",
    );

    match run_valid_pipeline(&logger, &mut logs) {
        Ok(()) => append_raw(
            &logger,
            &mut logs,
            LogLevel::Info,
            "DRIVER",
            "I_VALID_PIPELINE_OK",
            "valid BF pipeline completed",
        ),
        Err(err) => append_unhandled(&logger, &mut logs, &err),
    }

    run_invalid_pipeline(&logger, &mut logs);

    append_raw(
        &logger,
        &mut logs,
        LogLevel::Info,
        "DRIVER",
        "I_LOG_WRITE",
        &format!("writing log file to {}", LOG_PATH),
    );

    let log_content = logs
        .iter()
        .map(|record| logger.render_record(record))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    if let Err(e) = fs::write(LOG_PATH, log_content) {
        logger.emit_raw(
            LogLevel::FatalError,
            "DRIVER",
            "E_LOG_WRITE",
            &format!("failed to write '{}': {}", LOG_PATH, e),
        );
        return;
    }

    logger.emit_raw(
        LogLevel::Info,
        "DRIVER",
        "I_DONE",
        &format!("pipeline done, see {} {} {}", IR1_PATH, IR2_PATH, LOG_PATH),
    );
}

fn run_valid_pipeline(logger: &Logger, logs: &mut Vec<LogRecord>) -> Result<(), BfError> {
    let bf_valid = r#"
        >>>>>+++++<<<<<-----
        >>>>++++<<<<----
        >+>+>+>+<<<<-
        >>>+++<<<---
        >>++<<--
        >+++++<-----
        >>>>+<<<<-
        >>+++<<---
        >++++<----
        >>+++++<<-----
    "#;

    append_raw(
        logger,
        logs,
        LogLevel::Info,
        "DRIVER",
        "I_VALID_INPUT",
        &format!("valid raw BF chars={}", bf_valid.chars().count()),
    );

    let tkstream = Lexer::run_with_logger(bf_valid, Some(logger));
    append_raw(
        logger,
        logs,
        LogLevel::Debug,
        "LEXER",
        "D_TOKENS",
        &format!("token_count={}", tkstream.len()),
    );

    let ast = Parser::parse_with_logger(&tkstream, Some(logger)).map_err(BfError::from)?;

    let ir1 = Ir1Program::lower_with_logger(&ast, Some(logger)).map_err(BfError::from)?;
    ir1.write_bf1_file(IR1_PATH).map_err(BfError::from)?;
    append_raw(
        logger,
        logs,
        LogLevel::Info,
        "IR1",
        "I_IR1_FILE_WRITTEN",
        IR1_PATH,
    );

    let ir1_from_file = Ir1Program::read_bf1_file(IR1_PATH).map_err(BfError::from)?;

    let ir2 = Ir2Program::lower_with_logger(&ir1_from_file, Some(logger)).map_err(BfError::from)?;
    ir2.write_bf2_file(IR2_PATH).map_err(BfError::from)?;
    append_raw(
        logger,
        logs,
        LogLevel::Info,
        "IR2",
        "I_IR2_FILE_WRITTEN",
        IR2_PATH,
    );

    let ir2_from_file = Ir2Program::read_bf2_file(IR2_PATH).map_err(BfError::from)?;

    let mut vm = Status::with_logger(logger.clone());
    vm.run(ir2_from_file).map_err(BfError::from)?;

    append_raw(
        logger,
        logs,
        LogLevel::Info,
        "BFVM",
        "I_VM_RUN_OK",
        "valid IR2 file executed successfully",
    );

    Ok(())
}

fn run_invalid_pipeline(logger: &Logger, logs: &mut Vec<LogRecord>) {
    let bf_invalid = r#"
        +++++[>+++++<-]
        >>+<<
        [->+<]
        [>+<-]
        >>>++<<
    "#;

    append_raw(
        logger,
        logs,
        LogLevel::Info,
        "DRIVER",
        "I_INVALID_INPUT",
        &format!("invalid raw BF chars={}", bf_invalid.chars().count()),
    );

    let positioned = Lexer::run_with_positions(bf_invalid, Some(logger));
    let tkstream: Vec<Token> = positioned.iter().map(|tp| tp.token.clone()).collect();

    match Parser::parse(&tkstream) {
        Ok(_) => append_raw(
            logger,
            logs,
            LogLevel::Warning,
            "PARSER",
            "W_EXPECTED_PARSE_ERROR_MISSING",
            "invalid BF input unexpectedly parsed successfully",
        ),
        Err(err) => {
            let context = render_parse_context(&err, bf_invalid, &positioned);
            let wrapped: BfError = err.into();
            append_error_with_context(logger, logs, &wrapped, context.as_deref());
        }
    }
}

fn append_raw(
    logger: &Logger,
    logs: &mut Vec<LogRecord>,
    level: LogLevel,
    stage: &str,
    code: &str,
    detail: &str,
) {
    if let Some(record) = logger.record_raw(level, stage, code, detail) {
        logs.push(record);
    }
}

fn append_error_with_context<E: DiagnosticError + ?Sized>(
    logger: &Logger,
    logs: &mut Vec<LogRecord>,
    err: &E,
    context: Option<&str>,
) {
    if let Some(mut record) = logger.record_error(err) {
        if let Some(ctx) = context {
            if !ctx.is_empty() {
                record.msg = format!("{}\n{}", record.msg, ctx);
            }
        }
        logger.emit_record(&record);
        logs.push(record);
    }
}

fn append_unhandled<E: DiagnosticError + ?Sized>(
    logger: &Logger,
    logs: &mut Vec<LogRecord>,
    err: &E,
) {
    if let Some(record) = logger.record_unhandled(err) {
        logger.emit_record(&record);
        logs.push(record);
    }
}

fn render_parse_context(
    err: &ParseError,
    source: &str,
    positioned: &[TokenWithPos],
) -> Option<String> {
    let token_idx = match err {
        ParseError::UnexpectedRightBracket { pos } => *pos,
        ParseError::UnclosedLeftBracket { pos } => *pos,
    };

    let pos = positioned.get(token_idx)?;

    render_brainfuck_parse_diagnostic(&err.detail(), source, pos.line, pos.column)
}
