use squalr_engine_api::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
use squalr_engine_api::commands::pointer_scan::pointer_scan_request::PointerScanRequest;
use squalr_engine_api::commands::pointer_scan::pointer_scan_response::PointerScanResponse;
use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use squalr_engine_api::commands::scan::element_scan::element_scan_request::ElementScanRequest;
use squalr_engine_api::commands::scan::element_scan::element_scan_response::ElementScanResponse;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::new::scan_new_response::ScanNewResponse;
use squalr_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use squalr_engine_api::commands::scan::reset::scan_reset_response::ScanResetResponse;
use squalr_engine_api::commands::scan::scan_command::ScanCommand;
use squalr_engine_api::commands::struct_scan::struct_scan_command::StructScanCommand;
use squalr_engine_api::commands::struct_scan::struct_scan_request::StructScanRequest;
use squalr_engine_api::commands::struct_scan::struct_scan_response::StructScanResponse;
use squalr_engine_api::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use squalr_engine_api::structures::scanning::constraints::anonymous_scan_constraint::AnonymousScanConstraint;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use structopt::StructOpt;

use squalr_tests::mocks::mock_engine_bindings::MockEngineBindings;

#[test]
fn scan_new_request_maps_to_scan_new_privileged_command() {
    match (ScanNewRequest {}).to_engine_command() {
        PrivilegedCommand::Scan(ScanCommand::New {
            scan_new_request: ScanNewRequest {},
        }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn typed_response_round_trip_for_scan_new_response() {
    let scan_new_response = ScanNewResponse {};

    let engine_response = scan_new_response.to_engine_response();
    let typed_response_result = ScanNewResponse::from_engine_response(engine_response);

    assert!(typed_response_result.is_ok());
}

#[test]
fn scan_new_request_dispatches_new_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(ScanNewResponse {}.to_engine_response(), ProjectListResponse::default().to_engine_response());
    let dispatched_commands = bindings.get_dispatched_commands();
    let scan_new_request = ScanNewRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_new_request.send_unprivileged(&bindings, move |_scan_new_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Scan(ScanCommand::New { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_new_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanResetResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let scan_new_request = ScanNewRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_new_request.send_unprivileged(&bindings, move |_scan_new_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Scan(ScanCommand::New { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_reset_request_dispatches_reset_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanResetResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let scan_reset_request = ScanResetRequest {};

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_reset_request.send_unprivileged(&bindings, move |scan_reset_response| {
        callback_invoked_clone.store(scan_reset_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Scan(ScanCommand::Reset { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_reset_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanCollectValuesResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let scan_reset_request = ScanResetRequest {};

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_reset_request.send_unprivileged(&bindings, move |_scan_reset_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Scan(ScanCommand::Reset { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_collect_values_request_dispatches_collect_values_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ScanCollectValuesResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let scan_collect_values_request = ScanCollectValuesRequest {};

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_collect_values_request.send_unprivileged(&bindings, move |_scan_collect_values_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Scan(ScanCommand::CollectValues {
            scan_value_collector_request: _,
        }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn scan_collect_values_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanResetResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let scan_collect_values_request = ScanCollectValuesRequest {};

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    scan_collect_values_request.send_unprivileged(&bindings, move |_scan_collect_values_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Scan(ScanCommand::CollectValues {
            scan_value_collector_request: _,
        }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn element_scan_request_dispatches_element_scan_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        ElementScanResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let element_scan_request = ElementScanRequest {
        scan_constraints: vec![
            AnonymousScanConstraint::from_str(">=5;dec;").expect("scan constraint should parse"),
            AnonymousScanConstraint::from_str("==").expect("scan constraint should parse"),
        ],
        data_type_refs: vec![DataTypeRef::new("i32"), DataTypeRef::new("f32")],
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    element_scan_request.send_unprivileged(&bindings, move |_element_scan_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Scan(ScanCommand::ElementScan {
            element_scan_request: captured_element_scan_request,
        }) => {
            assert_eq!(captured_element_scan_request.scan_constraints.len(), 2);
            assert_eq!(captured_element_scan_request.data_type_refs.len(), 2);
            assert_eq!(captured_element_scan_request.data_type_refs[0].get_data_type_id(), "i32");
            assert_eq!(captured_element_scan_request.data_type_refs[1].get_data_type_id(), "f32");
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn element_scan_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanResetResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let element_scan_request = ElementScanRequest {
        scan_constraints: vec![AnonymousScanConstraint::from_str("==").expect("scan constraint should parse")],
        data_type_refs: vec![DataTypeRef::new("i32")],
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    element_scan_request.send_unprivileged(&bindings, move |_element_scan_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::Scan(ScanCommand::ElementScan { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn pointer_scan_request_dispatches_pointer_scan_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        PointerScanResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let pointer_scan_request = PointerScanRequest {
        target_address: squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString::from_str("4096;address;")
            .expect("anonymous value string should parse"),
        pointer_data_type_ref: DataTypeRef::new("u64"),
        max_depth: 5,
        offset_size: 8,
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    pointer_scan_request.send_unprivileged(&bindings, move |_pointer_scan_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::PointerScan(PointerScanCommand {
            pointer_scan_request: captured_pointer_scan_request,
        }) => {
            assert_eq!(
                captured_pointer_scan_request
                    .target_address
                    .get_anonymous_value_string(),
                "4096"
            );
            assert_eq!(
                captured_pointer_scan_request
                    .pointer_data_type_ref
                    .get_data_type_id(),
                "u64"
            );
            assert_eq!(captured_pointer_scan_request.max_depth, 5);
            assert_eq!(captured_pointer_scan_request.offset_size, 8);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn pointer_scan_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        ScanCollectValuesResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let pointer_scan_request = PointerScanRequest {
        target_address: squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString::from_str("4096;address;")
            .expect("anonymous value string should parse"),
        pointer_data_type_ref: DataTypeRef::new("u64"),
        max_depth: 5,
        offset_size: 8,
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    pointer_scan_request.send_unprivileged(&bindings, move |_pointer_scan_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::PointerScan(PointerScanCommand { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn struct_scan_request_dispatches_struct_scan_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        StructScanResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let struct_scan_request = StructScanRequest {
        scan_value: Some(
            squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString::from_str("12;dec;")
                .expect("anonymous value string should parse"),
        ),
        data_type_ids: vec!["u32".to_string(), "f32".to_string()],
        compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    struct_scan_request.send_unprivileged(&bindings, move |_struct_scan_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::StructScan(StructScanCommand {
            struct_scan_request: captured_struct_scan_request,
        }) => {
            assert_eq!(
                captured_struct_scan_request
                    .scan_value
                    .as_ref()
                    .map(|scan_value| scan_value.get_anonymous_value_string()),
                Some("12")
            );
            assert_eq!(captured_struct_scan_request.data_type_ids, vec!["u32".to_string(), "f32".to_string()]);
            assert_eq!(
                captured_struct_scan_request.compare_type,
                ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal)
            );
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn struct_scan_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        PointerScanResponse::default().to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_commands = bindings.get_dispatched_commands();
    let struct_scan_request = StructScanRequest {
        scan_value: Some(
            squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString::from_str("12;dec;")
                .expect("anonymous value string should parse"),
        ),
        data_type_ids: vec!["u32".to_string(), "f32".to_string()],
        compare_type: ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    struct_scan_request.send_unprivileged(&bindings, move |_struct_scan_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_commands_guard = dispatched_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_commands_guard.len(), 1);

    match &dispatched_commands_guard[0] {
        PrivilegedCommand::StructScan(StructScanCommand { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_pointer_scan_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "pointer-scan",
            "--target-address",
            "4096;address;",
            "--pointer-data-type-ref",
            "u64",
            "--max-depth",
            "5",
            "--offset-size",
            "8",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::PointerScan(PointerScanCommand { pointer_scan_request }) => {
            assert_eq!(pointer_scan_request.target_address.get_anonymous_value_string(), "4096");
            assert_eq!(pointer_scan_request.pointer_data_type_ref.get_data_type_id(), "u64");
            assert_eq!(pointer_scan_request.max_depth, 5);
            assert_eq!(pointer_scan_request.offset_size, 8);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_element_scan_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "scan",
            "element-scan",
            "--scan-constraints",
            ">=5;dec;",
            "--scan-constraints",
            "==",
            "--data-type-refs",
            "i32",
            "--data-type-refs",
            "f32",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Scan(ScanCommand::ElementScan { element_scan_request }) => {
            assert_eq!(element_scan_request.scan_constraints.len(), 2);
            assert_eq!(element_scan_request.data_type_refs.len(), 2);

            let first_constraint = &element_scan_request.scan_constraints[0];
            assert_eq!(
                first_constraint.get_scan_compare_type(),
                ScanCompareType::Immediate(ScanCompareTypeImmediate::GreaterThanOrEqual)
            );
            assert_eq!(
                first_constraint
                    .get_anonymous_value_string()
                    .as_ref()
                    .map(|anonymous_value_string| anonymous_value_string.get_anonymous_value_string()),
                Some("5")
            );

            let second_constraint = &element_scan_request.scan_constraints[1];
            assert_eq!(
                second_constraint.get_scan_compare_type(),
                ScanCompareType::Relative(ScanCompareTypeRelative::Unchanged)
            );
            assert_eq!(second_constraint.get_anonymous_value_string(), &None);

            assert_eq!(element_scan_request.data_type_refs[0].get_data_type_id(), "i32");
            assert_eq!(element_scan_request.data_type_refs[1].get_data_type_id(), "f32");
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_new_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "scan", "new"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Scan(ScanCommand::New { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_reset_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "scan", "reset"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Scan(ScanCommand::Reset { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_collect_values_subcommand() {
    let parse_result = std::panic::catch_unwind(|| PrivilegedCommand::from_iter_safe(["squalr-cli", "scan", "collect-values"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::Scan(ScanCommand::CollectValues { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_accepts_scan_struct_scan_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "struct-scan",
            "--scan-value",
            "12;dec;",
            "--data-type-ids",
            "u32",
            "--data-type-ids",
            "f32",
            "--compare-type",
            "==",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        PrivilegedCommand::StructScan(StructScanCommand { struct_scan_request }) => {
            assert_eq!(
                struct_scan_request
                    .scan_value
                    .as_ref()
                    .map(|scan_value| scan_value.get_anonymous_value_string()),
                Some("12")
            );
            assert_eq!(struct_scan_request.data_type_ids, vec!["u32".to_string(), "f32".to_string()]);
            assert_eq!(struct_scan_request.compare_type, ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn privileged_command_parser_rejects_scan_struct_scan_with_invalid_compare_type() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "struct-scan",
            "--scan-value",
            "12;dec;",
            "--data-type-ids",
            "u32",
            "--compare-type",
            "invalid-compare",
        ])
    });

    assert!(parse_result.is_ok());
    assert!(parse_result.expect("parser should not panic").is_err());
}

#[test]
fn privileged_command_parser_accepts_pointer_scan_alias_pscan() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "pscan",
            "--target-address",
            "4096;address;",
            "--pointer-data-type-ref",
            "u64",
            "--max-depth",
            "5",
            "--offset-size",
            "8",
        ])
    });

    assert!(parse_result.is_ok());
    assert!(parse_result.expect("parser should not panic").is_ok());
}

#[test]
fn privileged_command_parser_accepts_struct_scan_alias_sscan() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "sscan",
            "--scan-value",
            "12;dec;",
            "--data-type-ids",
            "u32",
            "--compare-type",
            "==",
        ])
    });

    assert!(parse_result.is_ok());
    assert!(parse_result.expect("parser should not panic").is_ok());
}

#[test]
fn privileged_command_parser_rejects_nested_scan_pointer_scan_subcommand() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "scan",
            "pointer-scan",
            "--target-address",
            "4096;address;",
            "--pointer-data-type-ref",
            "u64",
            "--max-depth",
            "5",
            "--offset-size",
            "8",
        ])
    });

    assert!(parse_result.is_ok());
    assert!(parse_result.expect("parser should not panic").is_err());
}

#[test]
fn privileged_command_parser_rejects_nested_scan_struct_scan_subcommand() {
    let parse_result = std::panic::catch_unwind(|| {
        PrivilegedCommand::from_iter_safe([
            "squalr-cli",
            "scan",
            "struct-scan",
            "--scan-value",
            "12;dec;",
            "--data-type-ids",
            "u32",
            "--compare-type",
            "==",
        ])
    });

    assert!(parse_result.is_ok());
    assert!(parse_result.expect("parser should not panic").is_err());
}
