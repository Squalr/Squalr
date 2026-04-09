use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project_items::add::project_items_add_request::ProjectItemsAddRequest;
use squalr_engine_api::commands::project_items::add::project_items_add_response::ProjectItemsAddResponse;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_response::ScanResultsRefreshResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_api::structures::structs::symbolic_field_definition::SymbolicFieldDefinition;
use squalr_engine_api::utils::file_system::file_system_utils::FileSystemUtils;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

impl UnprivilegedCommandRequestExecutor for ProjectItemsAddRequest {
    type ResponseType = ProjectItemsAddResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.scan_result_refs.is_empty() {
            return ProjectItemsAddResponse {
                success: true,
                added_project_item_count: 0,
            };
        }

        let scan_results = match request_scan_results(engine_unprivileged_state, self) {
            Some(scan_results) => scan_results,
            None => {
                return ProjectItemsAddResponse {
                    success: false,
                    added_project_item_count: 0,
                };
            }
        };

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project = match opened_project.write() {
            Ok(opened_project) => opened_project,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for add command: {}", error);

                return ProjectItemsAddResponse {
                    success: false,
                    added_project_item_count: 0,
                };
            }
        };
        let opened_project = match opened_project.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                log::error!("Cannot add scan results to project items without an opened project.");

                return ProjectItemsAddResponse {
                    success: false,
                    added_project_item_count: 0,
                };
            }
        };
        let project_directory_path = match opened_project.get_project_info().get_project_directory() {
            Some(project_directory_path) => project_directory_path,
            None => {
                log::error!("Failed to resolve opened project directory for project item add operation.");

                return ProjectItemsAddResponse {
                    success: false,
                    added_project_item_count: 0,
                };
            }
        };

        let added_file_paths = add_scan_results_to_project(
            engine_unprivileged_state,
            opened_project,
            &project_directory_path,
            &scan_results,
            &self.target_directory_path,
        );

        if added_file_paths.is_empty() {
            return ProjectItemsAddResponse {
                success: true,
                added_project_item_count: 0,
            };
        }

        if let Err(error) = create_placeholder_files(&added_file_paths) {
            log::error!("Failed creating project item placeholder files before save: {}", error);

            return ProjectItemsAddResponse {
                success: false,
                added_project_item_count: 0,
            };
        }

        if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
            log::error!("Failed to save project after add operation: {}", error);

            return ProjectItemsAddResponse {
                success: false,
                added_project_item_count: 0,
            };
        }

        project_manager.notify_project_items_changed();

        ProjectItemsAddResponse {
            success: true,
            added_project_item_count: added_file_paths.len() as u64,
        }
    }
}

fn request_scan_results(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    project_items_add_request: &ProjectItemsAddRequest,
) -> Option<Vec<ScanResult>> {
    let scan_results_refresh_request = ScanResultsRefreshRequest {
        scan_result_refs: project_items_add_request.scan_result_refs.clone(),
    };
    let scan_results_refresh_command = scan_results_refresh_request.to_engine_command();
    let (scan_results_sender, scan_results_receiver): (
        Sender<Result<ScanResultsRefreshResponse, String>>,
        Receiver<Result<ScanResultsRefreshResponse, String>>,
    ) = channel();

    let dispatch_result = match engine_unprivileged_state.get_bindings().read() {
        Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
            scan_results_refresh_command,
            Box::new(move |engine_response| {
                let conversion_result = match ScanResultsRefreshResponse::from_engine_response(engine_response) {
                    Ok(scan_results_refresh_response) => Ok(scan_results_refresh_response),
                    Err(unexpected_response) => Err(format!("Unexpected response variant for project-items add: {:?}", unexpected_response)),
                };

                if let Err(error) = scan_results_sender.send(conversion_result) {
                    log::error!("Failed to deliver refreshed scan results to project-items add command: {}", error);
                }
            }),
        ),
        Err(error) => {
            log::error!("Failed to acquire engine bindings lock for project-items add command: {}", error);
            return None;
        }
    };

    if let Err(error) = dispatch_result {
        log::error!("Failed to dispatch refresh request for project-items add command: {}", error);
        return None;
    }

    match scan_results_receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(scan_results_refresh_response)) => Some(scan_results_refresh_response.scan_results),
        Ok(Err(error)) => {
            log::error!("Failed to convert refresh response for project-items add command: {}", error);
            None
        }
        Err(error) => {
            log::error!("Timed out waiting for refreshed scan results during project-items add command: {}", error);
            None
        }
    }
}

fn add_scan_results_to_project(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    opened_project: &mut Project,
    project_directory_path: &PathBuf,
    scan_results: &[ScanResult],
    target_directory_path: &Option<PathBuf>,
) -> Vec<PathBuf> {
    let project_items = opened_project.get_project_items_mut();
    let mut added_file_paths = Vec::new();
    let project_root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
    let root_directory_project_item_ref = ProjectItemRef::new(project_root_directory_path.clone());

    if !project_items.contains_key(&root_directory_project_item_ref) {
        let root_directory_project_item = ProjectItemTypeDirectory::new_project_item(&root_directory_project_item_ref);
        project_items.insert(root_directory_project_item_ref, root_directory_project_item);
    }
    let selected_directory_path = resolve_selected_directory_path(project_directory_path, &project_root_directory_path, project_items, target_directory_path);
    let directory_relative_path = selected_directory_path
        .strip_prefix(project_directory_path)
        .unwrap_or(&selected_directory_path)
        .to_path_buf();

    for scan_result in scan_results {
        let data_type_ref = scan_result.get_data_type_ref();
        let default_data_value = match engine_unprivileged_state.get_default_value(data_type_ref) {
            Some(default_data_value) => default_data_value,
            None => {
                log::warn!("Skipping scan result add for unsupported data type: {}", data_type_ref.get_data_type_id());
                continue;
            }
        };
        let project_item_file_stem = build_project_item_file_stem(scan_result);
        let project_item_absolute_path =
            generate_unique_project_item_file_path(project_directory_path, &directory_relative_path, project_items, &project_item_file_stem);
        let project_item_ref = ProjectItemRef::new(project_item_absolute_path.clone());

        let project_item_name = build_project_item_name(scan_result);
        let mut project_item = ProjectItemTypeAddress::new_project_item(
            &project_item_name,
            scan_result.get_module_offset(),
            scan_result.get_module(),
            "",
            default_data_value,
        );
        let symbolic_field_definition = build_symbolic_field_definition_string(engine_unprivileged_state, scan_result);
        ProjectItemTypeAddress::set_field_symbolic_struct_definition_reference(&mut project_item, &symbolic_field_definition);

        project_items.insert(project_item_ref, project_item);
        added_file_paths.push(project_item_absolute_path);
    }

    added_file_paths
}

fn build_symbolic_field_definition_string(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    scan_result: &ScanResult,
) -> String {
    let data_type_ref = scan_result.get_data_type_ref().clone();
    let unit_size_in_bytes = engine_unprivileged_state
        .get_default_value(&data_type_ref)
        .map(|default_data_value| default_data_value.get_size_in_bytes())
        .unwrap_or(0);
    let value_size_in_bytes = scan_result
        .get_recently_read_value()
        .as_ref()
        .or_else(|| scan_result.get_valued_result().get_current_value().as_ref())
        .map(|data_value| data_value.get_size_in_bytes())
        .unwrap_or(unit_size_in_bytes);
    let explicit_array_length = resolve_scan_result_array_length(scan_result, value_size_in_bytes, unit_size_in_bytes);

    if let Some(array_length) = explicit_array_length {
        return SymbolicFieldDefinition::new(data_type_ref, ContainerType::ArrayFixed(array_length)).to_string();
    }

    if unit_size_in_bytes == 0 || value_size_in_bytes <= unit_size_in_bytes || value_size_in_bytes % unit_size_in_bytes != 0 {
        return data_type_ref.to_string();
    }

    let element_count = value_size_in_bytes / unit_size_in_bytes;
    SymbolicFieldDefinition::new(data_type_ref, ContainerType::ArrayFixed(element_count)).to_string()
}

fn resolve_scan_result_array_length(
    scan_result: &ScanResult,
    value_size_in_bytes: u64,
    unit_size_in_bytes: u64,
) -> Option<u64> {
    let display_container_type = scan_result
        .get_recently_read_display_values()
        .iter()
        .chain(scan_result.get_current_display_values().iter())
        .map(|display_value| display_value.get_container_type())
        .find(|container_type| matches!(container_type, ContainerType::Array | ContainerType::ArrayFixed(_)))?;

    match display_container_type {
        ContainerType::ArrayFixed(array_length) => Some(array_length.max(1)),
        ContainerType::Array => {
            if unit_size_in_bytes == 0 {
                Some(1)
            } else {
                Some((value_size_in_bytes / unit_size_in_bytes).max(1))
            }
        }
        _ => None,
    }
}

fn generate_unique_project_item_file_path(
    project_directory_path: &Path,
    directory_relative_path: &Path,
    project_items: &std::collections::HashMap<ProjectItemRef, squalr_engine_api::structures::projects::project_items::project_item::ProjectItem>,
    project_item_file_stem: &str,
) -> PathBuf {
    let mut duplicate_sequence_number: u64 = 0;

    loop {
        let project_item_file_name = if duplicate_sequence_number == 0 {
            format!("{}.json", project_item_file_stem)
        } else {
            format!("{}_{}.json", project_item_file_stem, duplicate_sequence_number)
        };
        let project_item_relative_path = directory_relative_path.join(project_item_file_name);
        let project_item_absolute_path = project_directory_path.join(project_item_relative_path);
        let project_item_ref = ProjectItemRef::new(project_item_absolute_path.clone());

        if !project_items.contains_key(&project_item_ref) {
            return project_item_absolute_path;
        }

        duplicate_sequence_number = duplicate_sequence_number.saturating_add(1);
    }
}

fn resolve_selected_directory_path(
    project_directory_path: &Path,
    project_root_directory_path: &Path,
    project_items: &std::collections::HashMap<ProjectItemRef, squalr_engine_api::structures::projects::project_items::project_item::ProjectItem>,
    target_directory_path: &Option<PathBuf>,
) -> PathBuf {
    let Some(target_directory_path) = target_directory_path else {
        return project_root_directory_path.to_path_buf();
    };
    let resolved_target_path = resolve_project_item_path(project_directory_path, target_directory_path);
    let resolved_directory_path = if is_directory_path(&resolved_target_path, project_items) {
        resolved_target_path
    } else {
        match resolved_target_path.parent() {
            Some(parent_path) => parent_path.to_path_buf(),
            None => project_root_directory_path.to_path_buf(),
        }
    };

    if resolved_directory_path.starts_with(project_root_directory_path) {
        resolved_directory_path
    } else {
        project_root_directory_path.to_path_buf()
    }
}

fn resolve_project_item_path(
    project_directory_path: &Path,
    project_item_path: &Path,
) -> PathBuf {
    if FileSystemUtils::is_cross_platform_absolute_path(project_item_path) {
        project_item_path.to_path_buf()
    } else {
        project_directory_path.join(project_item_path)
    }
}

fn is_directory_path(
    project_item_path: &Path,
    project_items: &std::collections::HashMap<ProjectItemRef, squalr_engine_api::structures::projects::project_items::project_item::ProjectItem>,
) -> bool {
    let project_item_ref = ProjectItemRef::new(project_item_path.to_path_buf());
    project_items
        .get(&project_item_ref)
        .map(|project_item| project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID)
        .unwrap_or(project_item_path.extension().is_none())
}

fn create_placeholder_files(file_paths: &[PathBuf]) -> Result<(), String> {
    for file_path in file_paths {
        if let Some(parent_path) = file_path.parent() {
            if let Err(error) = fs::create_dir_all(parent_path) {
                return Err(format!("Failed creating project item parent directory {:?}: {}", parent_path, error));
            }
        }

        if !file_path.exists() {
            if let Err(error) = File::create(file_path) {
                return Err(format!("Failed creating project item file {:?}: {}", file_path, error));
            }
        }
    }

    Ok(())
}

fn build_project_item_name(scan_result: &ScanResult) -> String {
    if scan_result.is_module() {
        scan_result.get_address_display_text()
    } else {
        format!("0x{:X}", scan_result.get_address())
    }
}

fn build_project_item_file_stem(scan_result: &ScanResult) -> String {
    if scan_result.is_module() {
        let sanitized_module_name = sanitize_file_name_component(scan_result.get_module());

        format!("{}_0x{:X}", sanitized_module_name, scan_result.get_module_offset())
    } else {
        format!("address_0x{:X}", scan_result.get_address())
    }
}

fn sanitize_file_name_component(file_name_component: &str) -> String {
    let mut sanitized_component = String::with_capacity(file_name_component.len());
    let mut previous_character_was_underscore = false;

    for name_character in file_name_component.chars() {
        let mapped_character = if name_character.is_ascii_alphanumeric() { name_character } else { '_' };

        if mapped_character == '_' {
            if previous_character_was_underscore {
                continue;
            }

            previous_character_was_underscore = true;
        } else {
            previous_character_was_underscore = false;
        }

        sanitized_component.push(mapped_character);
    }

    let trimmed_component = sanitized_component.trim_matches('_');

    if trimmed_component.is_empty() {
        String::from("module")
    } else {
        trimmed_component.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{build_project_item_file_stem, generate_unique_project_item_file_path, resolve_selected_directory_path};
    use crate::command_executors::project_items::add::project_items_add_request_executor::build_symbolic_field_definition_string;
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::commands::{privileged_command::PrivilegedCommand, privileged_command_response::PrivilegedCommandResponse};
    use squalr_engine_api::engine::{
        engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings, engine_binding_error::EngineBindingError, engine_event_envelope::EngineEventEnvelope,
        engine_execution_context::EngineExecutionContext,
    };
    use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
    use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
    use squalr_engine_api::structures::data_values::container_type::ContainerType;
    use squalr_engine_api::structures::data_values::data_value::DataValue;
    use squalr_engine_api::structures::memory::normalized_module::ModuleAddressDisplay;
    use squalr_engine_api::structures::projects::project::Project;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
    use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
    use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
    use squalr_engine_api::structures::scan_results::scan_result_valued::ScanResultValued;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, RwLock};

    fn create_directory_item_map(
        paths: &[&str],
        project_directory_path: &Path,
    ) -> HashMap<ProjectItemRef, ProjectItem> {
        let mut project_items = HashMap::new();

        for relative_path in paths {
            let absolute_path = project_directory_path.join(relative_path);
            let project_item_ref = ProjectItemRef::new(absolute_path.clone());
            let project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);

            project_items.insert(project_item_ref, project_item);
        }

        project_items
    }

    #[test]
    fn resolve_selected_directory_path_defaults_to_hidden_project_root() {
        let project_directory_path = Path::new("C:/Projects/TestProject");
        let project_root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
        let project_items = create_directory_item_map(&[Project::PROJECT_DIR], project_directory_path);

        let resolved_directory_path = resolve_selected_directory_path(project_directory_path, &project_root_directory_path, &project_items, &None);

        assert_eq!(resolved_directory_path, project_root_directory_path);
    }

    #[test]
    fn resolve_selected_directory_path_uses_selected_directory_when_inside_hidden_root() {
        let project_directory_path = Path::new("C:/Projects/TestProject");
        let project_root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
        let target_directory_relative_path = format!("{}/Addresses", Project::PROJECT_DIR);
        let project_items = create_directory_item_map(&[Project::PROJECT_DIR, &target_directory_relative_path], project_directory_path);
        let target_directory_path = Some(PathBuf::from(target_directory_relative_path.clone()));

        let resolved_directory_path =
            resolve_selected_directory_path(project_directory_path, &project_root_directory_path, &project_items, &target_directory_path);

        assert_eq!(resolved_directory_path, project_directory_path.join(target_directory_relative_path));
    }

    #[test]
    fn resolve_selected_directory_path_uses_parent_directory_for_selected_file() {
        let project_directory_path = Path::new("C:/Projects/TestProject");
        let project_root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
        let target_directory_relative_path = format!("{}/Addresses", Project::PROJECT_DIR);
        let mut project_items = create_directory_item_map(&[Project::PROJECT_DIR, &target_directory_relative_path], project_directory_path);
        let selected_file_path = project_directory_path.join(format!("{}/health.json", target_directory_relative_path));
        let selected_file_ref = ProjectItemRef::new(selected_file_path.clone());
        let selected_file_item = ProjectItemTypeAddress::new_project_item(
            "Health",
            0x1234,
            "",
            "",
            squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8::get_value_from_primitive(0),
        );
        project_items.insert(selected_file_ref, selected_file_item);
        let target_directory_path = Some(selected_file_path);

        let resolved_directory_path =
            resolve_selected_directory_path(project_directory_path, &project_root_directory_path, &project_items, &target_directory_path);

        assert_eq!(resolved_directory_path, project_directory_path.join(target_directory_relative_path));
    }

    fn create_scan_result(
        module_name: &str,
        module_offset: u64,
        address: u64,
        scan_result_global_index: u64,
    ) -> ScanResult {
        let scan_result_valued = ScanResultValued::new(
            address,
            DataTypeRef::new("u8"),
            String::new(),
            Some(DataTypeU8::get_value_from_primitive(0x7F)),
            Vec::new(),
            None,
            Vec::new(),
            ScanResultRef::new(scan_result_global_index),
        );

        ScanResult::new(
            scan_result_valued,
            module_name.to_string(),
            module_offset,
            ModuleAddressDisplay::ModuleRelative,
            None,
            Vec::new(),
            false,
        )
    }

    struct TestEngineBindings;

    impl EngineApiUnprivilegedBindings for TestEngineBindings {
        fn dispatch_privileged_command(
            &self,
            _engine_command: PrivilegedCommand,
            _callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable("project-items add test privileged dispatch"))
        }

        fn dispatch_unprivileged_command(
            &self,
            _engine_command: squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand,
            _engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
            _callback: Box<dyn FnOnce(squalr_engine_api::commands::unprivileged_command_response::UnprivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable("project-items add test unprivileged dispatch"))
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();
            Ok(event_receiver)
        }
    }

    fn create_test_engine_execution_context() -> Arc<dyn EngineExecutionContext> {
        let engine_unprivileged_state =
            squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState::new(Arc::new(RwLock::new(TestEngineBindings)));

        engine_unprivileged_state as Arc<dyn EngineExecutionContext>
    }

    #[test]
    fn build_project_item_file_stem_uses_module_and_sanitizes_special_characters() {
        let scan_result = create_scan_result("game.exe (x64)", 0x20, 0x1000, 1);

        let file_stem = build_project_item_file_stem(&scan_result);

        assert_eq!(file_stem, String::from("game_exe_x64_0x20"));
    }

    #[test]
    fn build_project_item_file_stem_uses_address_for_non_module_scan_result() {
        let scan_result = create_scan_result("", 0, 0x401020, 2);

        let file_stem = build_project_item_file_stem(&scan_result);

        assert_eq!(file_stem, String::from("address_0x401020"));
    }

    #[test]
    fn generate_unique_project_item_file_path_adds_numeric_suffix_when_name_collides() {
        let project_directory_path = Path::new("C:/Projects/TestProject");
        let directory_relative_path = Path::new("project_items/Addresses");
        let existing_item_path = project_directory_path.join("project_items/Addresses/address_0x401000.json");
        let existing_item_ref = ProjectItemRef::new(existing_item_path);
        let existing_item = ProjectItemTypeAddress::new_project_item("Existing", 0x401000, "", "", DataTypeU8::get_value_from_primitive(0));
        let mut project_items = HashMap::new();

        project_items.insert(existing_item_ref, existing_item);

        let generated_path = generate_unique_project_item_file_path(project_directory_path, directory_relative_path, &project_items, "address_0x401000");

        assert_eq!(generated_path, project_directory_path.join("project_items/Addresses/address_0x401000_1.json"));
    }

    #[test]
    fn build_symbolic_field_definition_string_uses_fixed_array_for_multi_element_scan_result() {
        let engine_execution_context = create_test_engine_execution_context();
        let scan_result_valued = ScanResultValued::new(
            0x1000,
            DataTypeRef::new("u8"),
            String::new(),
            Some(DataValue::new(DataTypeRef::new("u8"), vec![0x11, 0x22, 0x33])),
            Vec::new(),
            None,
            Vec::new(),
            ScanResultRef::new(1),
        );
        let scan_result = ScanResult::new(
            scan_result_valued,
            "game.exe".to_string(),
            0x20,
            ModuleAddressDisplay::ModuleRelative,
            None,
            Vec::new(),
            false,
        );

        let symbolic_field_definition = build_symbolic_field_definition_string(&engine_execution_context, &scan_result);

        assert_eq!(symbolic_field_definition, "u8[3]");
    }

    #[test]
    fn build_symbolic_field_definition_string_preserves_single_element_array_container() {
        let engine_execution_context = create_test_engine_execution_context();
        let scan_result_valued = ScanResultValued::new(
            0x1000,
            DataTypeRef::new("u8"),
            String::new(),
            Some(DataValue::new(DataTypeRef::new("u8"), vec![0x11])),
            vec![AnonymousValueString::new(
                "11".to_string(),
                AnonymousValueStringFormat::Hexadecimal,
                ContainerType::ArrayFixed(1),
            )],
            None,
            Vec::new(),
            ScanResultRef::new(2),
        );
        let scan_result = ScanResult::new(
            scan_result_valued,
            "game.exe".to_string(),
            0x20,
            ModuleAddressDisplay::ModuleRelative,
            None,
            Vec::new(),
            false,
        );

        let symbolic_field_definition = build_symbolic_field_definition_string(&engine_execution_context, &scan_result);

        assert_eq!(symbolic_field_definition, "u8[1]");
    }
}
