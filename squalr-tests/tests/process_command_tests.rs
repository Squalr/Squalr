use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::process::close::process_close_request::ProcessCloseRequest;
use squalr_engine_api::commands::process::close::process_close_response::ProcessCloseResponse;
use squalr_engine_api::commands::process::list::process_list_request::ProcessListRequest;
use squalr_engine_api::commands::process::list::process_list_response::ProcessListResponse;
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::process::open::process_open_response::ProcessOpenResponse;
use squalr_engine_api::commands::process::process_command::ProcessCommand;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use structopt::StructOpt;

use squalr_tests::mocks::mock_engine_bindings::MockEngineBindings;

#[test]
fn process_list_request_dispatches_list_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ProcessListResponse {
            processes: vec![
                ProcessInfo::new(1776, "first.exe".to_string(), true, None),
                ProcessInfo::new(9001, "second.exe".to_string(), false, None),
            ],
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let process_list_request = ProcessListRequest {
        require_windowed: true,
        search_name: Some("first".to_string()),
        match_case: true,
        limit: Some(25),
        fetch_icons: true,
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    process_list_request.send_unprivileged(&bindings, move |process_list_response| {
        let callback_should_mark_success = process_list_response.processes.len() == 2
            && process_list_response.processes[0].get_process_id_raw() == 1776
            && process_list_response.processes[0].get_name() == "first.exe"
            && process_list_response.processes[0].get_is_windowed()
            && process_list_response.processes[1].get_process_id_raw() == 9001
            && process_list_response.processes[1].get_name() == "second.exe"
            && !process_list_response.processes[1].get_is_windowed();
        callback_invoked_clone.store(callback_should_mark_success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Process(ProcessCommand::List {
            process_list_request: captured_process_list_request,
        }) => {
            assert!(captured_process_list_request.require_windowed);
            assert_eq!(captured_process_list_request.search_name, Some("first".to_string()));
            assert!(captured_process_list_request.match_case);
            assert_eq!(captured_process_list_request.limit, Some(25));
            assert!(captured_process_list_request.fetch_icons);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn process_list_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ProcessOpenResponse { opened_process_info: None }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let process_list_request = ProcessListRequest {
        require_windowed: false,
        search_name: Some("calc".to_string()),
        match_case: false,
        limit: Some(2),
        fetch_icons: false,
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    process_list_request.send_unprivileged(&bindings, move |_process_list_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Process(ProcessCommand::List {
            process_list_request: captured_process_list_request,
        }) => {
            assert!(!captured_process_list_request.require_windowed);
            assert_eq!(captured_process_list_request.search_name, Some("calc".to_string()));
            assert!(!captured_process_list_request.match_case);
            assert_eq!(captured_process_list_request.limit, Some(2));
            assert!(!captured_process_list_request.fetch_icons);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn process_open_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let process_open_request = ProcessOpenRequest {
        process_id: Some(1234),
        search_name: None,
        match_case: false,
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    process_open_request.send_unprivileged(&bindings, move |_process_open_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Process(ProcessCommand::Open {
            process_open_request: captured_process_open_request,
        }) => {
            assert_eq!(captured_process_open_request.process_id, Some(1234));
            assert_eq!(captured_process_open_request.search_name, None);
            assert!(!captured_process_open_request.match_case);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn process_open_request_dispatches_open_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ProcessOpenResponse {
            opened_process_info: Some(squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo::new(
                31337,
                "calc.exe".to_string(),
                0xCAFE,
                Bitness::Bit64,
                None,
            )),
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let process_open_request = ProcessOpenRequest {
        process_id: Some(31337),
        search_name: Some("calc".to_string()),
        match_case: true,
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    process_open_request.send_unprivileged(&bindings, move |process_open_response| {
        if let Some(opened_process_info) = process_open_response.opened_process_info {
            let was_expected_process = opened_process_info.get_process_id_raw() == 31337
                && opened_process_info.get_name() == "calc.exe"
                && opened_process_info.get_handle() == 0xCAFE
                && opened_process_info.get_bitness() == Bitness::Bit64;
            callback_invoked_clone.store(was_expected_process, Ordering::SeqCst);
        }
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Process(ProcessCommand::Open {
            process_open_request: captured_process_open_request,
        }) => {
            assert_eq!(captured_process_open_request.process_id, Some(31337));
            assert_eq!(captured_process_open_request.search_name, Some("calc".to_string()));
            assert!(captured_process_open_request.match_case);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn process_close_request_dispatches_close_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ProcessCloseResponse {
            process_info: Some(squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo::new(
                1337,
                "game.exe".to_string(),
                0xBEEF,
                Bitness::Bit64,
                None,
            )),
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let process_close_request = ProcessCloseRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    process_close_request.send_unprivileged(&bindings, move |process_close_response| {
        if let Some(closed_process_info) = process_close_response.process_info {
            let was_expected_process = closed_process_info.get_process_id_raw() == 1337
                && closed_process_info.get_name() == "game.exe"
                && closed_process_info.get_handle() == 0xBEEF
                && closed_process_info.get_bitness() == Bitness::Bit64;
            callback_invoked_clone.store(was_expected_process, Ordering::SeqCst);
        }
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Process(ProcessCommand::Close { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn process_close_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ProcessOpenResponse { opened_process_info: None }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();

    let process_close_request = ProcessCloseRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    process_close_request.send_unprivileged(&bindings, move |_process_close_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Process(ProcessCommand::Close { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_process_list_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "process",
            "list",
            "--require-windowed",
            "--search-name",
            "calc",
            "--match-case",
            "--limit",
            "10",
            "--fetch-icons",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Process(ProcessCommand::List { process_list_request }) => {
            assert!(process_list_request.require_windowed);
            assert_eq!(process_list_request.search_name, Some("calc".to_string()));
            assert!(process_list_request.match_case);
            assert_eq!(process_list_request.limit, Some(10));
            assert!(process_list_request.fetch_icons);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_process_open_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "process",
            "open",
            "--process-id",
            "1337",
            "--search-name",
            "calc",
            "--match-case",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Process(ProcessCommand::Open { process_open_request }) => {
            assert_eq!(process_open_request.process_id, Some(1337));
            assert_eq!(process_open_request.search_name, Some("calc".to_string()));
            assert!(process_open_request.match_case);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_process_close_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "process", "close"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Process(ProcessCommand::Close { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_rejects_process_open_with_invalid_process_id() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "process", "open", "--process-id", "not-a-number"]));

    assert!(parse_result.is_ok());
    assert!(parse_result.expect("parser should not panic").is_err());
}
