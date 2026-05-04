use crate::command_executors::project_symbols::project_symbol_store_mutation::save_and_sync_project_symbol_catalog;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::rename_module::project_symbols_rename_module_request::ProjectSymbolsRenameModuleRequest;
use squalr_engine_api::commands::project_symbols::rename_module::project_symbols_rename_module_response::ProjectSymbolsRenameModuleResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsRenameModuleRequest {
    type ResponseType = ProjectSymbolsRenameModuleResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-symbols rename-module command: {}", error);
                return ProjectSymbolsRenameModuleResponse::default();
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot rename Symbol Tree modules without an opened project.");
            return ProjectSymbolsRenameModuleResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for project-symbols rename-module command.");
            return ProjectSymbolsRenameModuleResponse::default();
        };
        let old_module_name = self.module_name.trim();
        let new_module_name = self.new_module_name.trim();

        if old_module_name.is_empty() || new_module_name.is_empty() {
            log::warn!("Project-symbols rename-module request requires non-empty module names.");
            return ProjectSymbolsRenameModuleResponse::default();
        }

        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();

        if old_module_name != new_module_name
            && project_symbol_catalog
                .find_symbol_module(new_module_name)
                .is_some()
        {
            log::warn!(
                "Project-symbols rename-module request cannot rename '{}' to duplicate module '{}'.",
                old_module_name,
                new_module_name
            );
            return ProjectSymbolsRenameModuleResponse::default();
        }

        let Some(existing_symbol_module) = project_symbol_catalog.find_symbol_module_mut(old_module_name) else {
            log::warn!("Project-symbols rename-module request could not find module '{}'.", old_module_name);
            return ProjectSymbolsRenameModuleResponse::default();
        };

        existing_symbol_module.set_module_name(new_module_name.to_string());

        for symbol_claim in project_symbol_catalog.get_symbol_claims_mut() {
            symbol_claim
                .get_locator_mut()
                .rename_module(old_module_name, new_module_name);
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsRenameModuleResponse::default();
        }

        ProjectSymbolsRenameModuleResponse {
            success: true,
            module_name: new_module_name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsRenameModuleRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{
        project::Project, project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
        project_symbol_module::ProjectSymbolModule,
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn rename_module_request_updates_module_and_module_relative_claims() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(String::from("game.exe"), 0x2000)],
            Vec::new(),
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Health"),
                String::from("game.exe"),
                0x1234,
                String::from("u32"),
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
        let rename_module_response = ProjectSymbolsRenameModuleRequest {
            module_name: String::from("game.exe"),
            new_module_name: String::from("patched.exe"),
        }
        .execute(&engine_execution_context);

        assert!(rename_module_response.success);
        assert_eq!(rename_module_response.module_name, "patched.exe");

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected renamed-module project to load from disk.");
        let project_symbol_catalog = loaded_project.get_project_info().get_project_symbol_catalog();
        let symbol_modules = project_symbol_catalog.get_symbol_modules();
        let symbol_claims = project_symbol_catalog.get_symbol_claims();

        assert_eq!(symbol_modules.len(), 1);
        assert_eq!(symbol_modules[0].get_module_name(), "patched.exe");
        assert_eq!(symbol_modules[0].get_size(), 0x2000);
        assert_eq!(
            symbol_claims[0].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("patched.exe"), 0x1234)
        );
        assert_eq!(symbol_claims[0].get_symbol_locator_key(), "module:patched.exe:1234");

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_modules(), symbol_modules);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
    }

    #[test]
    fn rename_module_request_rejects_duplicate_module_name() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![
                ProjectSymbolModule::new(String::from("game.exe"), 0x2000),
                ProjectSymbolModule::new(String::from("engine.dll"), 0x4000),
            ],
            Vec::new(),
            Vec::new(),
        );
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let rename_module_response = ProjectSymbolsRenameModuleRequest {
            module_name: String::from("game.exe"),
            new_module_name: String::from("engine.dll"),
        }
        .execute(&engine_execution_context);

        assert!(!rename_module_response.success);
    }
}
