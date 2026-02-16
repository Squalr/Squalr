use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use squalr_engine_api::commands::project_items::add::project_items_add_request::ProjectItemsAddRequest;
use squalr_engine_api::commands::project_items::add::project_items_add_response::ProjectItemsAddResponse;
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::commands::project_items::create::project_items_create_response::ProjectItemsCreateResponse;
use squalr_engine_api::commands::project_items::delete::project_items_delete_response::ProjectItemsDeleteResponse;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_response::ProjectItemsListResponse;
use squalr_engine_api::commands::project_items::move_item::project_items_move_response::ProjectItemsMoveResponse;
use squalr_engine_api::commands::project_items::project_items_command::ProjectItemsCommand;
use squalr_engine_api::commands::project_items::rename::project_items_rename_response::ProjectItemsRenameResponse;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_response::ProjectItemsReorderResponse;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_tests::shared_execution_context;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use structopt::StructOpt;

use squalr_tests::mocks::mock_engine_bindings::MockEngineBindings;

#[test]
fn project_items_add_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectItemsAddResponse {
            success: true,
            added_project_item_count: 2,
        }
        .to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_items_add_request = ProjectItemsAddRequest {
        scan_result_refs: vec![ScanResultRef::new(21), ScanResultRef::new(34)],
        target_directory_path: None,
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_add_request.send_unprivileged(&bindings, &execution_context, move |project_items_add_response| {
        callback_invoked_clone.store(
            project_items_add_response.success && project_items_add_response.added_project_item_count == 2,
            Ordering::SeqCst,
        );
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Add {
            project_items_add_request: captured_project_items_add_request,
        }) => {
            assert_eq!(captured_project_items_add_request.scan_result_refs.len(), 2);
            assert_eq!(captured_project_items_add_request.scan_result_refs[0].get_scan_result_global_index(), 21);
            assert_eq!(captured_project_items_add_request.scan_result_refs[1].get_scan_result_global_index(), 34);
            assert!(
                captured_project_items_add_request
                    .target_directory_path
                    .is_none()
            );
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_items_add_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectItemsListResponse::default().to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_items_add_request = ProjectItemsAddRequest {
        scan_result_refs: vec![ScanResultRef::new(8)],
        target_directory_path: None,
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_add_request.send_unprivileged(&bindings, &execution_context, move |_project_items_add_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Add { project_items_add_request }) => {
            assert_eq!(project_items_add_request.scan_result_refs.len(), 1);
            assert_eq!(project_items_add_request.scan_result_refs[0].get_scan_result_global_index(), 8);
            assert!(project_items_add_request.target_directory_path.is_none());
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

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
fn unprivileged_command_parser_accepts_project_items_add_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project-items",
            "add",
            "--scan-result-refs",
            "12",
            "--scan-result-refs",
            "29",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Add { project_items_add_request }) => {
            assert_eq!(project_items_add_request.scan_result_refs.len(), 2);
            assert_eq!(project_items_add_request.scan_result_refs[0].get_scan_result_global_index(), 12);
            assert_eq!(project_items_add_request.scan_result_refs[1].get_scan_result_global_index(), 29);
            assert!(project_items_add_request.target_directory_path.is_none());
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_items_add_target_directory_path() {
    let target_directory_path = format!("{}/Addresses", Project::PROJECT_DIR);
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project-items",
            "add",
            "--scan-result-refs",
            "12",
            "--target-directory-path",
            &target_directory_path,
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Add { project_items_add_request }) => {
            assert_eq!(project_items_add_request.scan_result_refs.len(), 1);
            assert_eq!(project_items_add_request.scan_result_refs[0].get_scan_result_global_index(), 12);
            assert_eq!(project_items_add_request.target_directory_path, Some(PathBuf::from(target_directory_path)));
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

#[test]
fn project_items_create_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectItemsCreateResponse {
            success: true,
            created_project_item_path: PathBuf::from("Addresses/New Folder"),
        }
        .to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_items_create_request = ProjectItemsCreateRequest {
        parent_directory_path: PathBuf::from("Addresses"),
        project_item_name: "New Folder".to_string(),
        project_item_type: "directory".to_string(),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_create_request.send_unprivileged(&bindings, &execution_context, move |project_items_create_response| {
        callback_invoked_clone.store(project_items_create_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Create {
            project_items_create_request: captured_project_items_create_request,
        }) => {
            assert_eq!(captured_project_items_create_request.parent_directory_path, PathBuf::from("Addresses"));
            assert_eq!(captured_project_items_create_request.project_item_name, "New Folder".to_string());
            assert_eq!(captured_project_items_create_request.project_item_type, "directory".to_string());
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_items_create_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project-items",
            "create",
            "--parent-directory-path",
            "Addresses",
            "--project-item-name",
            "New Folder",
            "--project-item-type",
            "directory",
        ])
    });

    assert!(parse_result.is_ok());
    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Create { project_items_create_request }) => {
            assert_eq!(project_items_create_request.parent_directory_path, PathBuf::from("Addresses"));
            assert_eq!(project_items_create_request.project_item_name, "New Folder".to_string());
            assert_eq!(project_items_create_request.project_item_type, "directory".to_string());
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_items_delete_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project-items",
            "delete",
            "--project-item-paths",
            "Addresses/scan_result_1.json",
            "--project-item-paths",
            "Addresses/scan_result_2.json",
        ])
    });

    assert!(parse_result.is_ok());
    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Delete { project_items_delete_request }) => {
            assert_eq!(project_items_delete_request.project_item_paths.len(), 2);
            assert_eq!(
                project_items_delete_request.project_item_paths[0],
                PathBuf::from("Addresses/scan_result_1.json")
            );
            assert_eq!(
                project_items_delete_request.project_item_paths[1],
                PathBuf::from("Addresses/scan_result_2.json")
            );
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_items_rename_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project-items",
            "rename",
            "--project-item-path",
            "Addresses/scan_result_1.json",
            "--project-item-name",
            "health.json",
        ])
    });

    assert!(parse_result.is_ok());
    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Rename { project_items_rename_request }) => {
            assert_eq!(project_items_rename_request.project_item_path, PathBuf::from("Addresses/scan_result_1.json"));
            assert_eq!(project_items_rename_request.project_item_name, "health.json".to_string());
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_items_move_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project-items",
            "move",
            "--project-item-paths",
            "Addresses/scan_result_1.json",
            "--project-item-paths",
            "Addresses/scan_result_2.json",
            "--target-directory-path",
            "Favorites",
        ])
    });

    assert!(parse_result.is_ok());
    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Move { project_items_move_request }) => {
            assert_eq!(project_items_move_request.project_item_paths.len(), 2);
            assert_eq!(project_items_move_request.project_item_paths[0], PathBuf::from("Addresses/scan_result_1.json"));
            assert_eq!(project_items_move_request.target_directory_path, PathBuf::from("Favorites"));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_items_reorder_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project-items",
            "reorder",
            "--project-item-paths",
            "Addresses/scan_result_2.json",
            "--project-item-paths",
            "Addresses/scan_result_1.json",
        ])
    });

    assert!(parse_result.is_ok());
    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Reorder { project_items_reorder_request }) => {
            assert_eq!(project_items_reorder_request.project_item_paths.len(), 2);
            assert_eq!(
                project_items_reorder_request.project_item_paths[0],
                PathBuf::from("Addresses/scan_result_2.json")
            );
            assert_eq!(
                project_items_reorder_request.project_item_paths[1],
                PathBuf::from("Addresses/scan_result_1.json")
            );
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn project_items_reorder_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectItemsReorderResponse {
            success: true,
            reordered_project_item_count: 2,
        }
        .to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();
    let project_items_reorder_request = ProjectItemsReorderRequest {
        project_item_paths: vec![
            PathBuf::from("Addresses/scan_result_2.json"),
            PathBuf::from("Addresses/scan_result_1.json"),
        ],
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_reorder_request.send_unprivileged(&bindings, &execution_context, move |project_items_reorder_response| {
        callback_invoked_clone.store(
            project_items_reorder_response.success && project_items_reorder_response.reordered_project_item_count == 2,
            Ordering::SeqCst,
        );
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Reorder {
            project_items_reorder_request: captured_project_items_reorder_request,
        }) => {
            assert_eq!(
                captured_project_items_reorder_request.project_item_paths,
                vec![
                    PathBuf::from("Addresses/scan_result_2.json"),
                    PathBuf::from("Addresses/scan_result_1.json"),
                ]
            );
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_items_reorder_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectItemsListResponse::default().to_engine_response(),
    );
    let execution_context = shared_execution_context();
    let project_items_reorder_request = ProjectItemsReorderRequest {
        project_item_paths: vec![PathBuf::from("Addresses/scan_result_2.json")],
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_items_reorder_request.send_unprivileged(&bindings, &execution_context, move |_project_items_reorder_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));
}

#[test]
fn project_items_new_response_types_round_trip_through_engine_response() {
    let delete_response = ProjectItemsDeleteResponse {
        success: true,
        deleted_project_item_count: 2,
    };
    let rename_response = ProjectItemsRenameResponse {
        success: true,
        renamed_project_item_path: PathBuf::from("Addresses/health.json"),
    };
    let move_response = ProjectItemsMoveResponse {
        success: true,
        moved_project_item_count: 2,
    };
    let reorder_response = ProjectItemsReorderResponse {
        success: true,
        reordered_project_item_count: 3,
    };

    let delete_round_trip = ProjectItemsDeleteResponse::from_engine_response(delete_response.to_engine_response())
        .expect("delete response should deserialize from command response");
    let rename_round_trip = ProjectItemsRenameResponse::from_engine_response(rename_response.to_engine_response())
        .expect("rename response should deserialize from command response");
    let move_round_trip =
        ProjectItemsMoveResponse::from_engine_response(move_response.to_engine_response()).expect("move response should deserialize from command response");
    let reorder_round_trip = ProjectItemsReorderResponse::from_engine_response(reorder_response.to_engine_response())
        .expect("reorder response should deserialize from command response");

    assert!(delete_round_trip.success);
    assert_eq!(delete_round_trip.deleted_project_item_count, 2);
    assert!(rename_round_trip.success);
    assert_eq!(rename_round_trip.renamed_project_item_path, PathBuf::from("Addresses/health.json"));
    assert!(move_round_trip.success);
    assert_eq!(move_round_trip.moved_project_item_count, 2);
    assert!(reorder_round_trip.success);
    assert_eq!(reorder_round_trip.reordered_project_item_count, 3);
}
