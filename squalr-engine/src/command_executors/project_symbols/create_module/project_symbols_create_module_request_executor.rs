use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::project_symbol_catalog_persistence::save_and_sync_project_symbol_catalog;
use squalr_engine_api::commands::project_symbols::create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest;
use squalr_engine_api::commands::project_symbols::create_module::project_symbols_create_module_response::ProjectSymbolsCreateModuleResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_symbol_module::ProjectSymbolModule;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsCreateModuleRequest {
    type ResponseType = ProjectSymbolsCreateModuleResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-symbols create-module command: {}", error);
                return ProjectSymbolsCreateModuleResponse::default();
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot create Symbol Tree module roots without an opened project.");
            return ProjectSymbolsCreateModuleResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for project-symbols create-module command.");
            return ProjectSymbolsCreateModuleResponse::default();
        };
        let module_name = self.module_name.trim();

        if module_name.is_empty() {
            log::warn!("Project-symbols create-module request requires a non-empty module name.");
            return ProjectSymbolsCreateModuleResponse::default();
        }

        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();

        if let Some(existing_symbol_module) = project_symbol_catalog.find_symbol_module_mut(module_name) {
            existing_symbol_module.set_size(self.size);
        } else {
            project_symbol_catalog
                .get_symbol_modules_mut()
                .push(ProjectSymbolModule::new(module_name.to_string(), self.size));
        }
        project_symbol_catalog.ensure_module_root_struct_layout(module_name, self.size, |data_type_ref| {
            let size_in_bytes = engine_unprivileged_state.get_unit_size_in_bytes(data_type_ref);

            (size_in_bytes > 0).then_some(size_in_bytes)
        });

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsCreateModuleResponse::default();
        }

        ProjectSymbolsCreateModuleResponse {
            success: true,
            module_name: module_name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsCreateModuleRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{project::Project, project_symbol_catalog::ProjectSymbolCatalog};
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn create_module_request_persists_module_name_and_size() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project = create_project_with_symbol_catalog(temp_directory.path(), ProjectSymbolCatalog::default());
        let mock_project_symbols_bindings = MockProjectSymbolsBindings::new();
        let captured_project_symbol_catalogs = mock_project_symbols_bindings.captured_project_symbol_catalogs();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_project_symbols_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let create_module_response = ProjectSymbolsCreateModuleRequest {
            module_name: String::from("game.exe"),
            size: 0x2000,
        }
        .execute(&engine_execution_context);

        assert!(create_module_response.success);
        assert_eq!(create_module_response.module_name, "game.exe");

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected created-module project to load from disk.");
        let symbol_modules = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules();

        assert_eq!(symbol_modules.len(), 1);
        assert_eq!(symbol_modules[0].get_module_name(), "game.exe");
        assert_eq!(symbol_modules[0].get_size(), 0x2000);
        let struct_layout_descriptors = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_struct_layout_descriptors();
        assert_eq!(struct_layout_descriptors.len(), 1);
        assert_eq!(struct_layout_descriptors[0].get_struct_layout_id(), "game.exe");
        assert_eq!(
            struct_layout_descriptors[0]
                .get_struct_layout_definition()
                .get_declared_size_in_bytes(),
            Some(0x2000)
        );

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_modules(), symbol_modules);
    }
}
