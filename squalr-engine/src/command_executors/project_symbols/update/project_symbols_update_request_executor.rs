use crate::command_executors::project_symbols::{
    project_symbol_layout_mutation::ProjectSymbolLayoutMutation, project_symbol_store_mutation::save_and_sync_project_symbol_catalog,
};
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::update::project_symbols_update_request::ProjectSymbolsUpdateRequest;
use squalr_engine_api::commands::project_symbols::update::project_symbols_update_response::ProjectSymbolsUpdateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsUpdateRequest {
    type ResponseType = ProjectSymbolsUpdateResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-symbols update command: {}", error);
                return ProjectSymbolsUpdateResponse::default();
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot update symbol claims without an opened project.");
            return ProjectSymbolsUpdateResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for project-symbols update command.");
            return ProjectSymbolsUpdateResponse::default();
        };

        let trimmed_display_name = self
            .display_name
            .as_deref()
            .map(str::trim)
            .filter(|display_name| !display_name.is_empty())
            .map(str::to_string);
        let trimmed_struct_layout_id = self
            .struct_layout_id
            .as_deref()
            .map(str::trim)
            .filter(|struct_layout_id| !struct_layout_id.is_empty())
            .map(str::to_string);

        if trimmed_display_name.is_none() && trimmed_struct_layout_id.is_none() {
            log::warn!("Project-symbols update request requires at least one non-empty update field.");
            return ProjectSymbolsUpdateResponse::default();
        }

        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();
        let local_struct_layout_descriptors = project_symbol_catalog.get_struct_layout_descriptors().to_vec();
        let resolve_field_size_in_bytes = |struct_layout_id: &str| {
            ProjectSymbolLayoutMutation::resolve_struct_layout_id_size_in_bytes(
                struct_layout_id,
                |data_type_ref| {
                    engine_unprivileged_state
                        .get_default_value(data_type_ref)
                        .map(|default_value| default_value.get_size_in_bytes())
                },
                |resolved_struct_layout_id| {
                    local_struct_layout_descriptors
                        .iter()
                        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == resolved_struct_layout_id)
                        .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_definition().clone())
                },
            )
        };
        let did_update = if let Some(symbol_claim) = project_symbol_catalog.find_symbol_claim_mut(&self.symbol_locator_key) {
            if let Some(display_name) = trimmed_display_name.as_ref() {
                symbol_claim.set_display_name(display_name.clone());
            }

            if let Some(struct_layout_id) = trimmed_struct_layout_id.as_ref() {
                symbol_claim.set_struct_layout_id(struct_layout_id.clone());
            }

            true
        } else if let Some((symbol_module, module_field)) = project_symbol_catalog.find_module_field(&self.symbol_locator_key) {
            let module_name = symbol_module.get_module_name().to_string();
            let display_name = trimmed_display_name
                .clone()
                .unwrap_or_else(|| module_field.get_display_name().to_string());
            let offset = module_field.get_offset();
            let struct_layout_id = trimmed_struct_layout_id
                .clone()
                .unwrap_or_else(|| module_field.get_struct_layout_id().to_string());

            match ProjectSymbolLayoutMutation::upsert_module_field(
                project_symbol_catalog,
                &module_name,
                display_name,
                offset,
                struct_layout_id,
                resolve_field_size_in_bytes,
            ) {
                Ok(_) => true,
                Err(error) => {
                    log::warn!("Project-symbols update module-field request failed: {}", error);
                    false
                }
            }
        } else {
            false
        };

        if !did_update {
            log::warn!(
                "Project-symbols update request could not find symbol locator key '{}'.",
                self.symbol_locator_key
            );
            return ProjectSymbolsUpdateResponse::default();
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsUpdateResponse::default();
        }

        ProjectSymbolsUpdateResponse {
            success: true,
            symbol_locator_key: self.symbol_locator_key.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsUpdateRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{
        project::Project, project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_module::ProjectSymbolModule,
        project_symbol_module_field::ProjectSymbolModuleField,
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn update_project_symbol_request_updates_display_name_and_type_and_syncs_catalog() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("i32"),
            )],
        );
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let mock_project_symbols_bindings = MockProjectSymbolsBindings::new();
        let captured_project_symbol_catalogs = mock_project_symbols_bindings.captured_project_symbol_catalogs();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_project_symbols_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_update_response = ProjectSymbolsUpdateRequest {
            symbol_locator_key: String::from("absolute:1234"),
            display_name: Some(String::from("Player Manager")),
            struct_layout_id: Some(String::from("u64")),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_update_response.success);
        assert_eq!(project_symbols_update_response.symbol_locator_key, "absolute:1234");

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected updated-symbol project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(symbol_claims[0].get_display_name(), "Player Manager");
        assert_eq!(symbol_claims[0].get_struct_layout_id(), "u64");

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
    }

    #[test]
    fn update_project_symbol_request_retypes_module_field_through_layout_mutation() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Health"), 0x08, String::from("u32")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_update_response = ProjectSymbolsUpdateRequest {
            symbol_locator_key: String::from("module:game.exe:8"),
            display_name: Some(String::from("Health64")),
            struct_layout_id: Some(String::from("u64")),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_update_response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected updated-module-field project to load from disk.");
        let module_fields = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules()[0]
            .get_fields();

        assert_eq!(module_fields.len(), 1);
        assert_eq!(module_fields[0].get_display_name(), "Health64");
        assert_eq!(module_fields[0].get_struct_layout_id(), "u64");
    }

    #[test]
    fn update_project_symbol_request_rejects_module_field_retype_overlap() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Health"), 0x08, String::from("u32")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Ammo"), 0x0C, String::from("u32")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_update_response = ProjectSymbolsUpdateRequest {
            symbol_locator_key: String::from("module:game.exe:8"),
            display_name: None,
            struct_layout_id: Some(String::from("u64")),
        }
        .execute(&engine_execution_context);

        assert!(!project_symbols_update_response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected overlap-rejected project to load from disk.");
        let module_fields = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules()[0]
            .get_fields();

        assert_eq!(module_fields[0].get_struct_layout_id(), "u32");
        assert_eq!(module_fields[1].get_struct_layout_id(), "u32");
    }
}
