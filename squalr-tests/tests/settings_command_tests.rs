use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::settings::general::general_settings_command::GeneralSettingsCommand;
use squalr_engine_api::commands::settings::general::list::general_settings_list_request::GeneralSettingsListRequest;
use squalr_engine_api::commands::settings::general::list::general_settings_list_response::GeneralSettingsListResponse;
use squalr_engine_api::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest;
use squalr_engine_api::commands::settings::general::set::general_settings_set_response::GeneralSettingsSetResponse;
use squalr_engine_api::commands::settings::memory::list::memory_settings_list_request::MemorySettingsListRequest;
use squalr_engine_api::commands::settings::memory::list::memory_settings_list_response::MemorySettingsListResponse;
use squalr_engine_api::commands::settings::memory::memory_settings_command::MemorySettingsCommand;
use squalr_engine_api::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest;
use squalr_engine_api::commands::settings::memory::set::memory_settings_set_response::MemorySettingsSetResponse;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_response::ScanSettingsListResponse;
use squalr_engine_api::commands::settings::scan::scan_settings_command::ScanSettingsCommand;
use squalr_engine_api::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest;
use squalr_engine_api::commands::settings::scan::set::scan_settings_set_response::ScanSettingsSetResponse;
use squalr_engine_api::commands::settings::settings_command::SettingsCommand;
use squalr_engine_api::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use structopt::StructOpt;

use squalr_tests::mocks::mock_engine_bindings::MockEngineBindings;

#[test]
fn general_settings_set_request_dispatches_set_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        GeneralSettingsSetResponse {}.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let general_settings_set_request = GeneralSettingsSetRequest {
        engine_request_delay: Some(300),
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    general_settings_set_request.send_unprivileged(&bindings, move |_general_settings_set_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Settings(SettingsCommand::General {
            general_settings_command:
                GeneralSettingsCommand::Set {
                    general_settings_set_request: captured_general_settings_set_request,
                },
        }) => {
            assert_eq!(captured_general_settings_set_request.engine_request_delay, Some(300));
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn general_settings_set_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        GeneralSettingsListResponse {
            general_settings: Err("settings unavailable".to_string()),
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let general_settings_set_request = GeneralSettingsSetRequest {
        engine_request_delay: Some(900),
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    general_settings_set_request.send_unprivileged(&bindings, move |_general_settings_set_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Settings(SettingsCommand::General {
            general_settings_command:
                GeneralSettingsCommand::Set {
                    general_settings_set_request: captured_general_settings_set_request,
                },
        }) => {
            assert_eq!(captured_general_settings_set_request.engine_request_delay, Some(900));
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn general_settings_list_request_dispatches_list_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        GeneralSettingsListResponse {
            general_settings: Err("general settings unavailable".to_string()),
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let general_settings_list_request = GeneralSettingsListRequest {};

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    general_settings_list_request.send_unprivileged(&bindings, move |general_settings_list_response| {
        callback_invoked_clone.store(general_settings_list_response.general_settings.is_err(), Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Settings(SettingsCommand::General {
            general_settings_command: GeneralSettingsCommand::List {
                general_settings_list_request: _,
            },
        }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn memory_settings_list_request_dispatches_list_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemorySettingsListResponse {
            memory_settings: Err("memory settings unavailable".to_string()),
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let memory_settings_list_request = MemorySettingsListRequest {};

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    memory_settings_list_request.send_unprivileged(&bindings, move |memory_settings_list_response| {
        callback_invoked_clone.store(memory_settings_list_response.memory_settings.is_err(), Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Settings(SettingsCommand::Memory {
            memory_settings_command: MemorySettingsCommand::List {
                memory_settings_list_request: _,
            },
        }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn memory_settings_set_request_dispatches_set_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemorySettingsSetResponse {}.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let memory_settings_set_request = MemorySettingsSetRequest {
        start_address: Some(12288),
        end_address: Some(65536),
        only_query_usermode: Some(true),
        required_write: Some(false),
        ..Default::default()
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    memory_settings_set_request.send_unprivileged(&bindings, move |_memory_settings_set_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Settings(SettingsCommand::Memory {
            memory_settings_command:
                MemorySettingsCommand::Set {
                    memory_settings_set_request: captured_memory_settings_set_request,
                },
        }) => {
            assert_eq!(captured_memory_settings_set_request.start_address, Some(12288));
            assert_eq!(captured_memory_settings_set_request.end_address, Some(65536));
            assert_eq!(captured_memory_settings_set_request.only_query_usermode, Some(true));
            assert_eq!(captured_memory_settings_set_request.required_write, Some(false));
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_settings_set_request_dispatches_set_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanSettingsSetResponse {}.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let scan_settings_set_request = ScanSettingsSetRequest {
        results_page_size: Some(256),
        results_read_interval_ms: None,
        project_read_interval_ms: None,
        freeze_interval_ms: None,
        memory_alignment: Some(MemoryAlignment::Alignment4),
        memory_read_mode: Some(MemoryReadMode::ReadInterleavedWithScan),
        floating_point_tolerance: Some(FloatingPointTolerance::Tolerance10E3),
        is_single_threaded_scan: Some(false),
        debug_perform_validation_scan: Some(true),
    };

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_settings_set_request.send_unprivileged(&bindings, move |_scan_settings_set_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Settings(SettingsCommand::Scan {
            scan_settings_command: ScanSettingsCommand::Set {
                scan_settings_set_request: captured_scan_settings_set_request,
            },
        }) => {
            assert_eq!(captured_scan_settings_set_request.results_page_size, Some(256));
            assert_eq!(captured_scan_settings_set_request.memory_alignment, Some(MemoryAlignment::Alignment4));
            assert_eq!(
                captured_scan_settings_set_request.memory_read_mode,
                Some(MemoryReadMode::ReadInterleavedWithScan)
            );
            assert_eq!(
                captured_scan_settings_set_request.floating_point_tolerance,
                Some(FloatingPointTolerance::Tolerance10E3)
            );
            assert_eq!(captured_scan_settings_set_request.debug_perform_validation_scan, Some(true));
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_settings_list_request_dispatches_list_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanSettingsListResponse {
            scan_settings: Err("scan settings unavailable".to_string()),
        }
        .to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let scan_settings_list_request = ScanSettingsListRequest {};

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_settings_list_request.send_unprivileged(&bindings, move |scan_settings_list_response| {
        callback_invoked_clone.store(scan_settings_list_response.scan_settings.is_err(), Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Settings(SettingsCommand::Scan {
            scan_settings_command: ScanSettingsCommand::List { scan_settings_list_request: _ },
        }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_memory_settings_set_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "settings",
            "memory",
            "set",
            "--start-address",
            "4096",
            "--end-address",
            "8192",
            "--only-query-usermode",
            "true",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::Memory {
            memory_settings_command: MemorySettingsCommand::Set { memory_settings_set_request },
        }) => {
            assert_eq!(memory_settings_set_request.start_address, Some(4096));
            assert_eq!(memory_settings_set_request.end_address, Some(8192));
            assert_eq!(memory_settings_set_request.only_query_usermode, Some(true));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_settings_set_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "settings",
            "scan",
            "set",
            "--results-page-size",
            "512",
            "--memory-alignment",
            "8",
            "--memory-read-mode",
            "i",
            "--floating-point-tolerance",
            "epsilon",
            "--is-single-threaded-scan",
            "true",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::Scan {
            scan_settings_command: ScanSettingsCommand::Set { scan_settings_set_request },
        }) => {
            assert_eq!(scan_settings_set_request.results_page_size, Some(512));
            assert_eq!(scan_settings_set_request.memory_alignment, Some(MemoryAlignment::Alignment8));
            assert_eq!(scan_settings_set_request.memory_read_mode, Some(MemoryReadMode::ReadInterleavedWithScan));
            assert_eq!(
                scan_settings_set_request.floating_point_tolerance,
                Some(FloatingPointTolerance::ToleranceEpsilon)
            );
            assert_eq!(scan_settings_set_request.is_single_threaded_scan, Some(true));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_general_settings_set_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "settings",
            "general",
            "set",
            "--engine-request-delay",
            "250",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::General {
            general_settings_command: GeneralSettingsCommand::Set { general_settings_set_request },
        }) => {
            assert_eq!(general_settings_set_request.engine_request_delay, Some(250));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_general_settings_list_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "settings", "general", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::General {
            general_settings_command: GeneralSettingsCommand::List { .. },
        }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_memory_settings_list_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "settings", "memory", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::Memory {
            memory_settings_command: MemorySettingsCommand::List { .. },
        }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_settings_list_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "settings", "scan", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Settings(SettingsCommand::Scan {
            scan_settings_command: ScanSettingsCommand::List { .. },
        }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_rejects_scan_settings_set_with_invalid_memory_alignment() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "settings",
            "scan",
            "set",
            "--memory-alignment",
            "invalid",
        ])
    });

    assert!(parse_result.is_ok());
    assert!(parse_result.expect("parser should not panic").is_err());
}
