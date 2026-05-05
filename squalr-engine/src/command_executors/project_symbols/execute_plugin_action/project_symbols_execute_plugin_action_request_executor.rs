use crate::command_executors::project_symbols::project_symbol_plugin_store::EngineProjectSymbolStore;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::{
    commands::project_symbols::execute_plugin_action::{
        project_symbols_execute_plugin_action_request::ProjectSymbolsExecutePluginActionRequest,
        project_symbols_execute_plugin_action_response::ProjectSymbolsExecutePluginActionResponse,
    },
    engine::engine_execution_context::EngineExecutionContext,
    plugins::symbol_tree::symbol_tree_action::{ProjectSymbolStore, SymbolTreeActionServices, SymbolTreeWindowStore},
};
use squalr_engine_session::plugins::plugin_registry::PluginRegistry;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsExecutePluginActionRequest {
    type ResponseType = ProjectSymbolsExecutePluginActionResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let plugin_registry = PluginRegistry::new();
        let Some((plugin_id, symbol_tree_action)) = plugin_registry
            .get_enabled_symbol_tree_actions()
            .into_iter()
            .find(|(plugin_id, symbol_tree_action)| plugin_id == &self.plugin_id && symbol_tree_action.action_id() == self.action_id)
        else {
            return ProjectSymbolsExecutePluginActionResponse {
                success: false,
                error: Some(format!(
                    "Could not resolve enabled Symbol Tree action `{}` from plugin `{}`.",
                    self.action_id, self.plugin_id
                )),
            };
        };

        if !plugin_registry.plugin_action_has_required_permissions(&plugin_id, symbol_tree_action.as_ref()) {
            return ProjectSymbolsExecutePluginActionResponse {
                success: false,
                error: Some(format!(
                    "Plugin `{}` does not declare the permissions required by `{}`.",
                    plugin_id, self.action_id
                )),
            };
        }

        let symbol_tree_action_services = EngineSymbolTreeActionServices::new(engine_unprivileged_state.clone());

        match symbol_tree_action.execute(&self.context, &symbol_tree_action_services) {
            Ok(()) => ProjectSymbolsExecutePluginActionResponse { success: true, error: None },
            Err(error) => ProjectSymbolsExecutePluginActionResponse {
                success: false,
                error: Some(error),
            },
        }
    }
}

struct EngineSymbolTreeActionServices {
    project_symbol_store: EngineProjectSymbolStore,
    symbol_tree_window_store: EngineSymbolTreeWindowStore,
}

impl EngineSymbolTreeActionServices {
    fn new(engine_unprivileged_state: Arc<dyn EngineExecutionContext>) -> Self {
        Self {
            project_symbol_store: EngineProjectSymbolStore::new(engine_unprivileged_state),
            symbol_tree_window_store: EngineSymbolTreeWindowStore,
        }
    }
}

impl SymbolTreeActionServices for EngineSymbolTreeActionServices {
    fn symbol_store(&self) -> &dyn ProjectSymbolStore {
        &self.project_symbol_store
    }

    fn symbol_tree_window(&self) -> &dyn SymbolTreeWindowStore {
        &self.symbol_tree_window_store
    }
}

struct EngineSymbolTreeWindowStore;

impl SymbolTreeWindowStore for EngineSymbolTreeWindowStore {
    fn request_refresh(&self) {}

    fn focus_tree_node(
        &self,
        _tree_node_key: &str,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsExecutePluginActionRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::{
        engine::engine_execution_context::EngineExecutionContext,
        plugins::symbol_tree::symbol_tree_action::{SymbolTreeActionContext, SymbolTreeActionSelection},
        structures::projects::{project::Project, project_symbol_catalog::ProjectSymbolCatalog, project_symbol_module::ProjectSymbolModule},
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    #[test]
    fn execute_plugin_action_populates_pe_symbols() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![ProjectSymbolModule::new(String::from("game.exe"), 0x2000)], Vec::new(), Vec::new());
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let response = ProjectSymbolsExecutePluginActionRequest {
            plugin_id: String::from("builtin.symbols.pe"),
            action_id: String::from("builtin.symbols.pe.populate-pe-symbols"),
            context: SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
                module_name: String::from("game.exe"),
            }),
        }
        .execute(&engine_execution_context);

        assert!(response.success, "Expected plugin action to succeed: {:?}", response.error);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected plugin-updated project to load from disk.");
        let symbol_module = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .find_symbol_module("game.exe")
            .expect("Expected module to remain in catalog.");

        assert_eq!(symbol_module.get_fields()[0].get_display_name(), "DOS Header");
        assert_eq!(symbol_module.get_fields()[0].get_struct_layout_id(), "win.pe.IMAGE_DOS_HEADER");
    }
}
