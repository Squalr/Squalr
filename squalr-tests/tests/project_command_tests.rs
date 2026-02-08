use squalr_engine_api::commands::memory::write::memory_write_response::MemoryWriteResponse;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::close::project_close_response::ProjectCloseResponse;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::create::project_create_response::ProjectCreateResponse;
use squalr_engine_api::commands::project::delete::project_delete_request::ProjectDeleteRequest;
use squalr_engine_api::commands::project::delete::project_delete_response::ProjectDeleteResponse;
use squalr_engine_api::commands::project::export::project_export_request::ProjectExportRequest;
use squalr_engine_api::commands::project::export::project_export_response::ProjectExportResponse;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest as UnprivilegedProjectOpenRequest;
use squalr_engine_api::commands::project::open::project_open_response::ProjectOpenResponse;
use squalr_engine_api::commands::project::project_command::ProjectCommand;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project::rename::project_rename_response::ProjectRenameResponse;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::project::save::project_save_response::ProjectSaveResponse;
use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::commands::unprivileged_command_response::TypedUnprivilegedCommandResponse;
use squalr_tests::shared_execution_context;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use structopt::StructOpt;

use squalr_tests::mocks::mock_engine_bindings::MockEngineBindings;

#[test]
fn project_list_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_list_request = ProjectListRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_list_request.send_unprivileged(&bindings, &execution_context, move |project_list_response| {
        callback_invoked_clone.store(project_list_response.projects_info.is_empty(), Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::List { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_open_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectOpenResponse { success: true }.to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_open_request = UnprivilegedProjectOpenRequest {
        open_file_browser: true,
        project_directory_path: Some(PathBuf::from("C:\\Projects\\ContractProject")),
        project_name: Some("ContractProject".to_string()),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_open_request.send_unprivileged(&bindings, &execution_context, move |project_open_response| {
        callback_invoked_clone.store(project_open_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Open {
            project_open_request: captured_project_open_request,
        }) => {
            assert!(captured_project_open_request.open_file_browser);
            assert_eq!(
                captured_project_open_request
                    .project_directory_path
                    .as_ref()
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\ContractProject".to_string())
            );
            assert_eq!(captured_project_open_request.project_name, Some("ContractProject".to_string()));
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_create_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectCreateResponse {
            success: true,
            new_project_path: PathBuf::from("C:\\Projects\\ContractCreateProject"),
        }
        .to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_create_request = ProjectCreateRequest {
        project_directory_path: Some(PathBuf::from("C:\\Projects")),
        project_name: Some("ContractCreateProject".to_string()),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_create_request.send_unprivileged(&bindings, &execution_context, move |project_create_response| {
        let callback_should_mark_success =
            project_create_response.success && project_create_response.new_project_path == PathBuf::from("C:\\Projects\\ContractCreateProject");
        callback_invoked_clone.store(callback_should_mark_success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Create {
            project_create_request: captured_project_create_request,
        }) => {
            assert_eq!(
                captured_project_create_request
                    .project_directory_path
                    .as_ref()
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects".to_string())
            );
            assert_eq!(captured_project_create_request.project_name, Some("ContractCreateProject".to_string()));
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_delete_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectDeleteResponse { success: true }.to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_delete_request = ProjectDeleteRequest {
        project_directory_path: Some(PathBuf::from("C:\\Projects\\ContractDeleteProject")),
        project_name: Some("ContractDeleteProject".to_string()),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_delete_request.send_unprivileged(&bindings, &execution_context, move |project_delete_response| {
        callback_invoked_clone.store(project_delete_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Delete {
            project_delete_request: captured_project_delete_request,
        }) => {
            assert_eq!(
                captured_project_delete_request
                    .project_directory_path
                    .as_ref()
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\ContractDeleteProject".to_string())
            );
            assert_eq!(captured_project_delete_request.project_name, Some("ContractDeleteProject".to_string()));
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_rename_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectRenameResponse {
            success: true,
            new_project_path: PathBuf::from("C:\\Projects\\RenamedProject"),
        }
        .to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_rename_request = ProjectRenameRequest {
        project_directory_path: PathBuf::from("C:\\Projects\\OriginalProject"),
        new_project_name: "RenamedProject".to_string(),
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_rename_request.send_unprivileged(&bindings, &execution_context, move |project_rename_response| {
        let callback_should_mark_success =
            project_rename_response.success && project_rename_response.new_project_path == PathBuf::from("C:\\Projects\\RenamedProject");
        callback_invoked_clone.store(callback_should_mark_success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Rename {
            project_rename_request: captured_project_rename_request,
        }) => {
            assert_eq!(
                captured_project_rename_request
                    .project_directory_path
                    .display()
                    .to_string(),
                "C:\\Projects\\OriginalProject".to_string()
            );
            assert_eq!(captured_project_rename_request.new_project_name, "RenamedProject".to_string());
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_export_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectExportResponse { success: true }.to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_export_request = ProjectExportRequest {
        project_directory_path: Some(PathBuf::from("C:\\Projects\\ContractExportProject")),
        project_name: Some("ContractExportProject".to_string()),
        open_export_folder: true,
    };
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_export_request.send_unprivileged(&bindings, &execution_context, move |project_export_response| {
        callback_invoked_clone.store(project_export_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Export {
            project_export_request: captured_project_export_request,
        }) => {
            assert_eq!(
                captured_project_export_request
                    .project_directory_path
                    .as_ref()
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\ContractExportProject".to_string())
            );
            assert_eq!(captured_project_export_request.project_name, Some("ContractExportProject".to_string()));
            assert!(captured_project_export_request.open_export_folder);
        }
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_close_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectCloseResponse { success: true }.to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_close_request = ProjectCloseRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_close_request.send_unprivileged(&bindings, &execution_context, move |project_close_response| {
        callback_invoked_clone.store(project_close_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Close { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_save_request_dispatches_unprivileged_command_and_invokes_typed_callback() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectSaveResponse { success: true }.to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_save_request = ProjectSaveRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_save_request.send_unprivileged(&bindings, &execution_context, move |project_save_response| {
        callback_invoked_clone.store(project_save_response.success, Ordering::SeqCst);
    });

    assert!(callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Save { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn project_save_request_does_not_invoke_callback_when_response_variant_is_wrong() {
    let bindings = MockEngineBindings::new(
        MemoryWriteResponse { success: true }.to_engine_response(),
        ProjectListResponse::default().to_engine_response(),
    );
    let dispatched_unprivileged_commands = bindings.get_dispatched_unprivileged_commands();

    let execution_context = shared_execution_context();

    let project_save_request = ProjectSaveRequest {};
    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    project_save_request.send_unprivileged(&bindings, &execution_context, move |_project_save_response| {
        callback_invoked_clone.store(true, Ordering::SeqCst);
    });

    assert!(!callback_invoked.load(Ordering::SeqCst));

    let dispatched_unprivileged_commands_guard = dispatched_unprivileged_commands
        .lock()
        .expect("command capture lock should be available");
    assert_eq!(dispatched_unprivileged_commands_guard.len(), 1);

    match &dispatched_unprivileged_commands_guard[0] {
        UnprivilegedCommand::Project(ProjectCommand::Save { .. }) => {}
        dispatched_command => panic!("unexpected dispatched command: {dispatched_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_create_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "create",
            "--project-directory-path",
            "C:\\Projects",
            "--project-name",
            "UnitTestProject",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Create { project_create_request }) => {
            assert_eq!(
                project_create_request
                    .project_directory_path
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects".to_string())
            );
            assert_eq!(project_create_request.project_name, Some("UnitTestProject".to_string()));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_rename_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "rename",
            "--project-directory-path",
            "C:\\Projects\\OldProject",
            "--new-project-name",
            "RenamedProject",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Rename { project_rename_request }) => {
            assert_eq!(
                project_rename_request
                    .project_directory_path
                    .display()
                    .to_string(),
                "C:\\Projects\\OldProject".to_string()
            );
            assert_eq!(project_rename_request.new_project_name, "RenamedProject".to_string());
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_open_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "open",
            "--open-file-browser",
            "--project-directory-path",
            "C:\\Projects\\OpenProject",
            "--project-name",
            "OpenProject",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Open { project_open_request }) => {
            assert!(project_open_request.open_file_browser);
            assert_eq!(
                project_open_request
                    .project_directory_path
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\OpenProject".to_string())
            );
            assert_eq!(project_open_request.project_name, Some("OpenProject".to_string()));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_delete_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "delete",
            "--project-directory-path",
            "C:\\Projects\\DeleteProject",
            "--project-name",
            "DeleteProject",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Delete { project_delete_request }) => {
            assert_eq!(
                project_delete_request
                    .project_directory_path
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\DeleteProject".to_string())
            );
            assert_eq!(project_delete_request.project_name, Some("DeleteProject".to_string()));
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_export_with_long_flags() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "export",
            "--project-directory-path",
            "C:\\Projects\\ExportProject",
            "--project-name",
            "ExportProject",
            "--open-export-folder",
        ])
    });

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Export { project_export_request }) => {
            assert_eq!(
                project_export_request
                    .project_directory_path
                    .map(|project_directory_path| project_directory_path.display().to_string()),
                Some("C:\\Projects\\ExportProject".to_string())
            );
            assert_eq!(project_export_request.project_name, Some("ExportProject".to_string()));
            assert!(project_export_request.open_export_folder);
        }
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_list_subcommand() {
    let parse_result = std::panic::catch_unwind(|| UnprivilegedCommand::from_iter_safe(["squalr-cli", "project", "list"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::List { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_close_subcommand() {
    let parse_result = std::panic::catch_unwind(|| UnprivilegedCommand::from_iter_safe(["squalr-cli", "project", "close"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Close { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_accepts_project_save_subcommand() {
    let parse_result = std::panic::catch_unwind(|| UnprivilegedCommand::from_iter_safe(["squalr-cli", "project", "save"]));

    assert!(parse_result.is_ok());

    let parsed_command_result = parse_result.expect("parser should not panic");
    assert!(parsed_command_result.is_ok());

    match parsed_command_result.expect("command should parse successfully") {
        UnprivilegedCommand::Project(ProjectCommand::Save { .. }) => {}
        parsed_command => panic!("unexpected parsed command: {parsed_command:?}"),
    }
}

#[test]
fn unprivileged_command_parser_rejects_project_rename_when_new_name_is_missing() {
    let parse_result = std::panic::catch_unwind(|| {
        UnprivilegedCommand::from_iter_safe([
            "squalr-cli",
            "project",
            "rename",
            "--project-directory-path",
            "C:\\Projects\\OldProject",
        ])
    });

    assert!(parse_result.is_ok());
    assert!(parse_result.expect("parser should not panic").is_err());
}
