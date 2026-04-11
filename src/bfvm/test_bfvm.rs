use super::*;

fn block(id: BlockId, insts: Vec<Ir2Inst>, term: Option<Ir2Terminator>) -> Ir2Block {
    Ir2Block { id, insts, term }
}

fn function(name: &str, entry: BlockId, blocks: Vec<Ir2Block>) -> Ir2Function {
    Ir2Function {
        name: name.to_string(),
        entry,
        blocks,
    }
}

#[test]
fn execute_str_on_maps_parse_error_to_runtime_error() {
    let mut vm = Status::new();
    let err = execute_str_on("NOT_BF2\n", &mut vm).unwrap_err();
    assert!(matches!(err, RuntimeError::InvalidProgramText(_)));
}

#[test]
fn run_fails_when_entry_function_missing() {
    let mut vm = Status::new();
    let prog = Ir2Program {
        functions: vec![function(
            "helper",
            0,
            vec![block(0, vec![], Some(Ir2Terminator::Return))],
        )],
    };

    let err = vm.run(prog).unwrap_err();
    assert!(matches!(
        err,
        RuntimeError::EntryFunctionNotFound { ref name } if name == "entry"
    ));
}

#[test]
fn run_fails_on_duplicate_entry_functions() {
    let mut vm = Status::new();
    let prog = Ir2Program {
        functions: vec![
            function(
                "entry",
                0,
                vec![block(0, vec![], Some(Ir2Terminator::Return))],
            ),
            function(
                "entry",
                0,
                vec![block(0, vec![], Some(Ir2Terminator::Return))],
            ),
        ],
    };

    let err = vm.run(prog).unwrap_err();
    assert!(matches!(
        err,
        RuntimeError::DuplicateEntryFunction { ref name } if name == "entry"
    ));
}

#[test]
fn run_reports_missing_terminator_with_diagnostics() {
    let mut vm = Status::new();
    let prog = Ir2Program {
        functions: vec![function("entry", 0, vec![block(0, vec![], None)])],
    };

    let err = vm.run(prog).unwrap_err();
    assert!(matches!(
        err,
        RuntimeError::MissingTerminator {
            ref function,
            block_id: 0
        } if function == "entry"
    ));
}

#[test]
fn run_reports_pointer_oob_with_diagnostics() {
    let mut vm = Status::new();
    let prog = Ir2Program {
        functions: vec![function(
            "entry",
            0,
            vec![block(
                0,
                vec![Ir2Inst::AddPtrImm(-1)],
                Some(Ir2Terminator::Return),
            )],
        )],
    };

    let err = vm.run(prog).unwrap_err();
    assert!(matches!(
        err,
        RuntimeError::PtrOutOfBounds {
            current: 0,
            delta: -1,
            attempted: -1,
            min: 0,
            ..
        }
    ));
}

#[test]
fn add_uses_wrapping_semantics() {
    let mut vm = Status::new();
    vm.tape[0] = i64::MAX;

    let prog = Ir2Program {
        functions: vec![function(
            "entry",
            0,
            vec![block(
                0,
                vec![Ir2Inst::AddCellImm(1)],
                Some(Ir2Terminator::Return),
            )],
        )],
    };

    vm.run(prog).unwrap();
    assert_eq!(vm.tape[0], i64::MIN);
}

#[test]
fn vm_accepts_logger_attachment_and_detachment() {
    let logger = Logger::new(LogLevel::Debug);
    let mut vm = Status::with_logger(logger.clone());

    let prog = Ir2Program {
        functions: vec![function(
            "entry",
            0,
            vec![block(0, vec![], Some(Ir2Terminator::Return))],
        )],
    };

    vm.run(prog).unwrap();

    vm.detach_logger();
    vm.attach_logger(logger);

    let prog2 = Ir2Program {
        functions: vec![function(
            "entry",
            0,
            vec![block(0, vec![], Some(Ir2Terminator::Return))],
        )],
    };
    vm.run(prog2).unwrap();
}
