use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_response::ProjectItemsListResponse;
use squalr_engine_api::commands::project_items::project_items_command::ProjectItemsCommand;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use squalr_tests::shared_execution_context;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use structopt::StructOpt;

use squalr_tests::mocks::mock_engine_bindings::MockEngineBindings;

#[test]
fn project_items_activate_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectItemsActivateResponse {}.to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_items_activate_request = ProjectItemsActivateRequest {
        project_item_paths: vec![
            "Addresses.Player.Health".to_string(),
            "Addresses.Player.Ammo".to_string(),
        ],
        is_activated: true,
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_activate_request.send_unprivileged(&bindings, &execution_context, move |_project_items_activate_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Activate {
            project_items_activate_request: captured_project_items_activate_request,
        }) => {
            assert_eq!(
                captured_project_items_activate_request.project_item_paths,
                vec![
                    "Addresses.Player.Health".to_string(),
                    "Addresses.Player.Ammo".to_string(),
                ]
            );
            assert!(captured_project_items_activate_request.is_activated);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_items_list_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectItemsListResponse::default().to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_items_list_request = ProjectItemsListRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_list_request.send_unprivileged(&bindings, &execution_context, move |project_items_list_response| {
        let response_has_empty_project = project_items_list_response.opened_project_info.is_none() && project_items_list_response.opened_project_root.is_none();
        callback_invoked_clone.store(response_has_empty_project, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::List { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_items_list_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_items_list_request = ProjectItemsListRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_list_request.send_unprivileged(&bindings, &execution_context, move |_project_items_list_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::List { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_items_activate_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project-items",
            "activate",
            "--project-item-paths",
            "Addresses.Player.Health",
            "--project-item-paths",
            "Addresses.Player.Ammo",
            "--is-activated",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Activate {
            project_items_activate_request,
        }) => {
            assert_eq!(project_items_activate_request.project_item_paths.len(), 2);
            assert_eq!(project_items_activate_request.project_item_paths[0], "Addresses.Player.Health".to_string());
            assert_eq!(project_items_activate_request.project_item_paths[1], "Addresses.Player.Ammo".to_string());
            assert!(project_items_activate_request.is_activated);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_items_list_subcommand() {
    let parse_result = std::panic::catch_unwind(|| UnprivilegedCommand::from_iter_safe(["squalr-cli", "project-items", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::List { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_rejects_project_items_activate_when_path_value_is_missing() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project-items",
            "activate",
            "--project-item-paths",
            "--is-activated",
        ])
    });

    assert!(parse_result.is_ok());
    assert!(parse_result.expect("parser should not panic").is_err());
}
