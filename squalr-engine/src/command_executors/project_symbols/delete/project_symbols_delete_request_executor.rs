use crate::command_executors::project_symbols::project_symbol_store_mutation::save_and_sync_project_symbol_catalog;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest;
use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_response::ProjectSymbolsDeleteResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::collections::HashSet;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsDeleteRequest {
    type ResponseType = ProjectSymbolsDeleteResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.symbol_locator_keys.is_empty() && self.module_names.is_empty() {
            return ProjectSymbolsDeleteResponse {
                success: true,
                deleted_symbol_count: 0,
                deleted_module_count: 0,
            };
        }

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-symbols delete command: {}", error);
                return ProjectSymbolsDeleteResponse::default();
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot delete symbol claims without an opened project.");
            return ProjectSymbolsDeleteResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for project-symbols delete command.");
            return ProjectSymbolsDeleteResponse::default();
        };
        let symbol_locator_key_set = self
            .symbol_locator_keys
            .iter()
            .map(|symbol_locator_key| symbol_locator_key.trim())
            .filter(|symbol_locator_key| !symbol_locator_key.is_empty())
            .map(str::to_string)
            .collect::<HashSet<String>>();
        let module_name_set = self
            .module_names
            .iter()
            .map(|module_name| module_name.trim())
            .filter(|module_name| !module_name.is_empty())
            .map(str::to_string)
            .collect::<HashSet<String>>();
        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();
        let symbol_modules = project_symbol_catalog.get_symbol_modules_mut();
        let symbol_module_count_before_delete = symbol_modules.len();

        symbol_modules.retain(|symbol_module| !module_name_set.contains(symbol_module.get_module_name()));

        let deleted_module_count = symbol_module_count_before_delete.saturating_sub(symbol_modules.len()) as u64;
        let symbol_claims = project_symbol_catalog.get_symbol_claims_mut();
        let symbol_claim_count_before_delete = symbol_claims.len();

        symbol_claims.retain(|symbol_claim| {
            if symbol_locator_key_set.contains(&symbol_claim.get_symbol_locator_key()) {
                return false;
            }

            match symbol_claim.get_locator() {
                squalr_engine_api::structures::projects::project_symbol_locator::ProjectSymbolLocator::ModuleOffset { module_name, .. } => {
                    !module_name_set.contains(module_name)
                }
                squalr_engine_api::structures::projects::project_symbol_locator::ProjectSymbolLocator::AbsoluteAddress { .. } => true,
            }
        });

        let deleted_symbol_count = symbol_claim_count_before_delete.saturating_sub(symbol_claims.len()) as u64;

        if deleted_symbol_count == 0 && deleted_module_count == 0 {
            return ProjectSymbolsDeleteResponse {
                success: true,
                deleted_symbol_count: 0,
                deleted_module_count: 0,
            };
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsDeleteResponse {
                success: false,
                deleted_symbol_count,
                deleted_module_count,
            };
        }

        ProjectSymbolsDeleteResponse {
            success: true,
            deleted_symbol_count,
            deleted_module_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsDeleteRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{
        project::Project, project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_module::ProjectSymbolModule,
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn delete_project_symbols_request_removes_matching_symbol_claims() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![
                ProjectSymbolClaim::new_absolute_address(String::from("Player"), 0x1234, String::from("player")),
                ProjectSymbolClaim::new_absolute_address(String::from("Enemy"), 0x5678, String::from("enemy")),
            ],
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
        let project_symbols_delete_response = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: vec![String::from("absolute:1234")],
            module_names: Vec::new(),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_delete_response.success);
        assert_eq!(project_symbols_delete_response.deleted_symbol_count, 1);
        assert_eq!(project_symbols_delete_response.deleted_module_count, 0);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected deleted-symbol project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(symbol_claims[0].get_symbol_locator_key(), "absolute:5678");

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
    }

    #[test]
    fn delete_project_symbols_request_removes_module_and_module_relative_claims() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![
                ProjectSymbolModule::new(String::from("game.exe"), 0x2000),
                ProjectSymbolModule::new(String::from("engine.dll"), 0x4000),
            ],
            Vec::new(),
            vec![
                ProjectSymbolClaim::new_module_offset(String::from("Health"), String::from("game.exe"), 0x1234, String::from("u32")),
                ProjectSymbolClaim::new_module_offset(String::from("State"), String::from("engine.dll"), 0x20, String::from("u8")),
                ProjectSymbolClaim::new_absolute_address(String::from("Loose"), 0x5678, String::from("u16")),
            ],
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
        let project_symbols_delete_response = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: Vec::new(),
            module_names: vec![String::from("game.exe")],
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_delete_response.success);
        assert_eq!(project_symbols_delete_response.deleted_module_count, 1);
        assert_eq!(project_symbols_delete_response.deleted_symbol_count, 1);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected module-deleted project to load from disk.");
        let project_symbol_catalog = loaded_project.get_project_info().get_project_symbol_catalog();
        let symbol_modules = project_symbol_catalog.get_symbol_modules();
        let symbol_claims = project_symbol_catalog.get_symbol_claims();

        assert_eq!(symbol_modules.len(), 1);
        assert_eq!(symbol_modules[0].get_module_name(), "engine.dll");
        assert_eq!(symbol_claims.len(), 2);
        assert!(symbol_claims.iter().all(|symbol_claim| {
            !symbol_claim
                .get_symbol_locator_key()
                .starts_with("module:game.exe:")
        }));

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_modules(), symbol_modules);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
    }
}
