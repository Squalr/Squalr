use crate::command_executors::project_symbols::{
    project_symbol_name_scope::ProjectSymbolNameScope, project_symbol_store_mutation::save_and_sync_project_symbol_catalog,
};
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::rename::project_symbols_rename_request::ProjectSymbolsRenameRequest;
use squalr_engine_api::commands::project_symbols::rename::project_symbols_rename_response::ProjectSymbolsRenameResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_symbol_locator::ProjectSymbolLocator;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsRenameRequest {
    type ResponseType = ProjectSymbolsRenameResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-symbols rename command: {}", error);
                return ProjectSymbolsRenameResponse::default();
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot rename symbol claims without an opened project.");
            return ProjectSymbolsRenameResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for project-symbols rename command.");
            return ProjectSymbolsRenameResponse::default();
        };
        let trimmed_display_name = self.display_name.trim();

        if trimmed_display_name.is_empty() {
            log::warn!("Project-symbols rename request requires a non-empty display name.");
            return ProjectSymbolsRenameResponse::default();
        }

        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();
        let did_rename = if let Some(symbol_claim) = project_symbol_catalog.find_symbol_claim(&self.symbol_locator_key) {
            let display_name = ProjectSymbolNameScope::deduplicate_display_name(
                project_symbol_catalog,
                project_symbol_catalog.get_symbol_claims(),
                symbol_claim.get_locator(),
                trimmed_display_name,
                Some(&self.symbol_locator_key),
            );

            if let Some(symbol_claim) = project_symbol_catalog.find_symbol_claim_mut(&self.symbol_locator_key) {
                symbol_claim.set_display_name(display_name);
                true
            } else {
                false
            }
        } else if let Some((symbol_module, module_field)) = project_symbol_catalog.find_module_field(&self.symbol_locator_key) {
            let locator = ProjectSymbolLocator::new_module_offset(symbol_module.get_module_name().to_string(), module_field.get_offset());
            let display_name = ProjectSymbolNameScope::deduplicate_display_name(
                project_symbol_catalog,
                project_symbol_catalog.get_symbol_claims(),
                &locator,
                trimmed_display_name,
                Some(&self.symbol_locator_key),
            );
            if let Some(module_field) = project_symbol_catalog.find_module_field_mut(&self.symbol_locator_key) {
                module_field.set_display_name(display_name);
                true
            } else {
                false
            }
        } else {
            false
        };

        if !did_rename {
            log::warn!(
                "Project-symbols rename request could not find symbol locator key '{}'.",
                self.symbol_locator_key
            );
            return ProjectSymbolsRenameResponse::default();
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsRenameResponse::default();
        }

        ProjectSymbolsRenameResponse {
            success: true,
            symbol_locator_key: self.symbol_locator_key.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsRenameRequest;
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
    fn rename_project_symbol_request_updates_display_name_and_syncs_catalog() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player"),
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
        let project_symbols_rename_response = ProjectSymbolsRenameRequest {
            symbol_locator_key: String::from("absolute:1234"),
            display_name: String::from("Player Manager"),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_rename_response.success);
        assert_eq!(project_symbols_rename_response.symbol_locator_key, "absolute:1234");

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected renamed-symbol project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(symbol_claims[0].get_display_name(), "Player Manager");

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
    }

    #[test]
    fn rename_project_symbol_request_deduplicates_module_field_display_name_in_scope() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Timer"), 0x00, String::from("u32")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Clock"), 0x04, String::from("u32")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_rename_response = ProjectSymbolsRenameRequest {
            symbol_locator_key: String::from("module:game.exe:4"),
            display_name: String::from("Timer"),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_rename_response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected deduplicated-module-field project to load from disk.");
        let module_fields = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules()[0]
            .get_fields();

        assert_eq!(module_fields[0].get_display_name(), "Timer");
        assert_eq!(module_fields[1].get_display_name(), "Timer_0");
    }
}
