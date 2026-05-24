use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::project_item_file_mutation::resolve_project_item_path;
use squalr_engine_api::commands::project_items::strip_symbol::project_items_strip_symbol_request::ProjectItemsStripSymbolRequest;
use squalr_engine_api::commands::project_items::strip_symbol::project_items_strip_symbol_response::ProjectItemsStripSymbolResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsStripSymbolRequest {
    type ResponseType = ProjectItemsStripSymbolResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.project_item_paths.is_empty() {
            return ProjectItemsStripSymbolResponse {
                success: true,
                stripped_project_item_count: 0,
                error: None,
            };
        }

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project_lock.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                let error = format!("Failed to acquire opened project lock for strip-symbol command: {}.", error);
                log::error!("{}", error);

                return ProjectItemsStripSymbolResponse {
                    success: false,
                    stripped_project_item_count: 0,
                    error: Some(error),
                };
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            let error = String::from("Cannot strip project item symbol information without an opened project.");
            log::warn!("{}", error);

            return ProjectItemsStripSymbolResponse {
                success: false,
                stripped_project_item_count: 0,
                error: Some(error),
            };
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            let error = String::from("Failed to resolve opened project directory for strip-symbol operation.");
            log::error!("{}", error);

            return ProjectItemsStripSymbolResponse {
                success: false,
                stripped_project_item_count: 0,
                error: Some(error),
            };
        };
        let project_symbol_catalog = opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .clone();
        let mut stripped_project_item_count = 0_u64;

        for project_item_path in &self.project_item_paths {
            let resolved_project_item_path = resolve_project_item_path(&project_directory_path, project_item_path);
            let project_item_ref = ProjectItemRef::new(resolved_project_item_path.clone());
            let Some(project_item) = opened_project.get_project_item_mut(&project_item_ref) else {
                log::warn!("Cannot strip symbol information, project item was not found: {:?}.", resolved_project_item_path);
                continue;
            };

            if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
                continue;
            }

            let address_target = ProjectItemTypeAddress::get_address_target(project_item);

            if !address_target.has_symbolic_offsets() {
                continue;
            }

            let Some(stripped_address_target) = address_target.strip_symbolic_offsets(&project_symbol_catalog) else {
                log::warn!(
                    "Cannot strip unresolved symbol information from project item: {:?}.",
                    resolved_project_item_path
                );
                continue;
            };

            if stripped_address_target == address_target {
                continue;
            }

            ProjectItemTypeAddress::set_address_target(project_item, stripped_address_target);
            project_item.set_has_unsaved_changes(true);
            stripped_project_item_count += 1;
        }

        if stripped_project_item_count == 0 {
            return ProjectItemsStripSymbolResponse {
                success: true,
                stripped_project_item_count,
                error: None,
            };
        }

        opened_project
            .get_project_info_mut()
            .set_has_unsaved_changes(true);
        if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
            let error = format!("Failed to save project after strip-symbol operation: {}.", error);
            log::error!("{}", error);

            return ProjectItemsStripSymbolResponse {
                success: false,
                stripped_project_item_count,
                error: Some(error),
            };
        }
        drop(opened_project_guard);

        project_manager.notify_project_items_changed();

        ProjectItemsStripSymbolResponse {
            success: true,
            stripped_project_item_count,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemsStripSymbolRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::{
        data_types::built_in_types::u32::data_type_u32::DataTypeU32,
        memory::pointer_chain_segment::PointerChainSegment,
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::{
            project::Project,
            project_items::{
                built_in_types::{project_item_type_address::ProjectItemTypeAddress, project_item_type_address_target::ProjectItemAddressTarget},
                project_item_ref::ProjectItemRef,
            },
            project_symbol_catalog::ProjectSymbolCatalog,
            project_symbol_module::ProjectSymbolModule,
            project_symbol_module_field::ProjectSymbolModuleField,
        },
    };
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn strip_symbol_request_replaces_resolved_symbolic_offsets() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x1000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Health"), 0x240, String::from("u32")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let mut project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let project_item_relative_path = PathBuf::from(Project::PROJECT_DIR).join("health.json");
        let project_item_absolute_path = temp_directory.path().join(&project_item_relative_path);
        let project_item_ref = ProjectItemRef::new(project_item_absolute_path.clone());
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0, "game.exe", "", DataTypeU32::get_value_from_primitive(0));
        let address_target = ProjectItemAddressTarget::new(
            String::from("game.exe"),
            vec![
                PointerChainSegment::Symbol(String::from("Health")),
                PointerChainSegment::Offset(0x10),
            ],
            PointerScanPointerSize::Pointer64,
        );

        ProjectItemTypeAddress::set_address_target(&mut project_item, address_target);
        project
            .get_project_items_mut()
            .insert(project_item_ref.clone(), project_item);

        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());
        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_items_strip_symbol_response = ProjectItemsStripSymbolRequest {
            project_item_paths: vec![project_item_relative_path],
        }
        .execute(&engine_execution_context);

        assert!(project_items_strip_symbol_response.success);
        assert_eq!(project_items_strip_symbol_response.stripped_project_item_count, 1);

        let opened_project_lock = engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project_guard = opened_project_lock
            .read()
            .expect("Expected opened project read lock in test.");
        let opened_project = opened_project_guard
            .as_ref()
            .expect("Expected opened project in test.");
        let mut stripped_project_item = opened_project
            .get_project_items()
            .get(&project_item_ref)
            .cloned()
            .expect("Expected stripped project item in test.");
        let stripped_address_target = ProjectItemTypeAddress::get_address_target(&mut stripped_project_item);

        assert_eq!(
            stripped_address_target.get_pointer_offsets(),
            &[
                PointerChainSegment::Offset(0x240),
                PointerChainSegment::Offset(0x10)
            ]
        );
    }
}
