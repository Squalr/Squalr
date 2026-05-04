use crate::command_executors::project_symbols::{
    project_symbol_layout_mutation::ProjectSymbolLayoutMutation, project_symbol_name_scope::ProjectSymbolNameScope,
    project_symbol_store_mutation::save_and_sync_project_symbol_catalog,
};
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::create::project_symbols_create_request::ProjectSymbolsCreateRequest;
use squalr_engine_api::commands::project_symbols::create::project_symbols_create_response::ProjectSymbolsCreateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::{project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsCreateRequest {
    type ResponseType = ProjectSymbolsCreateResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-symbols create command: {}", error);
                return ProjectSymbolsCreateResponse::default();
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot create symbol claims without an opened project.");
            return ProjectSymbolsCreateResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for project-symbols create command.");
            return ProjectSymbolsCreateResponse::default();
        };
        let Some(locator) = build_locator(self) else {
            log::warn!("Project-symbols create request did not provide a valid locator.");
            return ProjectSymbolsCreateResponse::default();
        };
        let trimmed_display_name = self.display_name.trim();

        if trimmed_display_name.is_empty() || self.struct_layout_id.trim().is_empty() {
            log::warn!("Project-symbols create request requires a non-empty display name and type id.");
            return ProjectSymbolsCreateResponse::default();
        }

        let struct_layout_id = self.struct_layout_id.trim().to_string();
        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();
        let created_symbol_locator_key = locator.to_locator_key();
        let display_name = ProjectSymbolNameScope::deduplicate_display_name(
            project_symbol_catalog,
            project_symbol_catalog.get_symbol_claims(),
            &locator,
            trimmed_display_name,
            None,
        );
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

        match locator {
            ProjectSymbolLocator::ModuleOffset { module_name, offset } => {
                if let Err(error) = ProjectSymbolLayoutMutation::upsert_module_field(
                    project_symbol_catalog,
                    &module_name,
                    display_name,
                    offset,
                    struct_layout_id,
                    resolve_field_size_in_bytes,
                ) {
                    log::warn!("Project-symbols create module-field request failed: {}", error);
                    return ProjectSymbolsCreateResponse::default();
                }
            }
            ProjectSymbolLocator::AbsoluteAddress { .. } => {
                let mut created_symbol = ProjectSymbolClaim::new(display_name, locator, struct_layout_id);
                *created_symbol.get_metadata_mut() = self.metadata.clone();
                project_symbol_catalog
                    .get_symbol_claims_mut()
                    .push(created_symbol);
            }
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsCreateResponse::default();
        }

        ProjectSymbolsCreateResponse {
            success: true,
            created_symbol_locator_key,
        }
    }
}

fn build_locator(project_symbols_create_request: &ProjectSymbolsCreateRequest) -> Option<ProjectSymbolLocator> {
    if let Some(address) = project_symbols_create_request.address {
        return Some(ProjectSymbolLocator::new_absolute_address(address));
    }

    let module_name = project_symbols_create_request
        .module_name
        .as_deref()
        .map(str::trim)
        .filter(|module_name| !module_name.is_empty())?;
    let offset = project_symbols_create_request.offset?;

    Some(ProjectSymbolLocator::new_module_offset(module_name.to_string(), offset))
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsCreateRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_values::container_type::ContainerType;
    use squalr_engine_api::structures::projects::{project::Project, project_symbol_catalog::ProjectSymbolCatalog};
    use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::sync::Arc;

    fn create_player_stats_struct_layout_descriptor() -> StructLayoutDescriptor {
        StructLayoutDescriptor::new(
            String::from("player.stats"),
            SymbolicStructDefinition::new(
                String::from("player.stats"),
                vec![
                    SymbolicFieldDefinition::new_named(String::from("health"), DataTypeRef::new("u32"), ContainerType::None),
                    SymbolicFieldDefinition::new_named(String::from("team"), DataTypeRef::new("u16"), ContainerType::None),
                ],
            ),
        )
    }

    #[test]
    fn create_project_symbol_request_persists_module_field_and_syncs_catalog() {
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
        let project_symbols_create_response = ProjectSymbolsCreateRequest {
            display_name: String::from("Player Manager"),
            struct_layout_id: String::from("u32"),
            address: None,
            module_name: Some(String::from("game.exe")),
            offset: Some(0x1234),
            metadata: std::collections::BTreeMap::default(),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_create_response.success);
        assert_eq!(project_symbols_create_response.created_symbol_locator_key, "module:game.exe:1234");

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected created-symbol project to load from disk.");
        let project_symbol_catalog = loaded_project.get_project_info().get_project_symbol_catalog();
        let symbol_modules = project_symbol_catalog.get_symbol_modules();

        assert_eq!(project_symbol_catalog.get_symbol_claims().len(), 0);
        assert_eq!(symbol_modules.len(), 1);
        assert_eq!(symbol_modules[0].get_module_name(), "game.exe");
        assert_eq!(symbol_modules[0].get_size(), 0x1238);
        assert_eq!(symbol_modules[0].get_fields().len(), 1);
        assert_eq!(symbol_modules[0].get_fields()[0].get_display_name(), "Player Manager");
        assert_eq!(symbol_modules[0].get_fields()[0].get_offset(), 0x1234);
        assert_eq!(symbol_modules[0].get_fields()[0].get_struct_layout_id(), "u32");

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_modules(), symbol_modules);
    }

    #[test]
    fn create_project_symbol_request_carves_existing_module_u8_field() {
        use squalr_engine_api::structures::projects::{project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField};

        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("u8_00000000"), 0x00, String::from("u8[32]")));
        let project = create_project_with_symbol_catalog(
            temp_directory.path(),
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new()),
        );
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_create_response = ProjectSymbolsCreateRequest {
            display_name: String::from("Health"),
            struct_layout_id: String::from("u32"),
            address: None,
            module_name: Some(String::from("game.exe")),
            offset: Some(0x08),
            metadata: std::collections::BTreeMap::default(),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_create_response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected carved-symbol project to load from disk.");
        let symbol_modules = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules();
        let module_fields = symbol_modules[0].get_fields();

        assert_eq!(module_fields.len(), 3);
        assert_eq!(module_fields[0].get_offset(), 0x00);
        assert_eq!(module_fields[0].get_struct_layout_id(), "u8[8]");
        assert_eq!(module_fields[1].get_display_name(), "Health");
        assert_eq!(module_fields[1].get_offset(), 0x08);
        assert_eq!(module_fields[1].get_struct_layout_id(), "u32");
        assert_eq!(module_fields[2].get_offset(), 0x0C);
        assert_eq!(module_fields[2].get_struct_layout_id(), "u8[20]");
    }

    #[test]
    fn create_project_symbol_request_persists_struct_layout_module_field_by_struct_size() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![create_player_stats_struct_layout_descriptor()]);
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_create_response = ProjectSymbolsCreateRequest {
            display_name: String::from("Player Stats"),
            struct_layout_id: String::from("player.stats"),
            address: None,
            module_name: Some(String::from("game.exe")),
            offset: Some(0x10),
            metadata: std::collections::BTreeMap::default(),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_create_response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected struct-backed project to load from disk.");
        let symbol_modules = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules();

        assert_eq!(symbol_modules[0].get_size(), 0x16);
        assert_eq!(symbol_modules[0].get_fields()[0].get_struct_layout_id(), "player.stats");
    }

    #[test]
    fn create_project_symbol_request_deduplicates_module_field_display_name_in_scope() {
        use squalr_engine_api::structures::projects::{project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField};

        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Timer"), 0x00, String::from("u32")));
        let project = create_project_with_symbol_catalog(
            temp_directory.path(),
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new()),
        );
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_create_response = ProjectSymbolsCreateRequest {
            display_name: String::from("Timer"),
            struct_layout_id: String::from("u32"),
            address: None,
            module_name: Some(String::from("game.exe")),
            offset: Some(0x04),
            metadata: std::collections::BTreeMap::default(),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_create_response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected deduplicated-symbol project to load from disk.");
        let module_fields = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules()[0]
            .get_fields();

        assert_eq!(module_fields[0].get_display_name(), "Timer");
        assert_eq!(module_fields[1].get_display_name(), "Timer_0");
    }

    #[test]
    fn create_project_symbol_request_persists_pointer_to_struct_module_field_by_pointer_size() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![create_player_stats_struct_layout_descriptor()]);
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_create_response = ProjectSymbolsCreateRequest {
            display_name: String::from("Player Stats Pointer"),
            struct_layout_id: String::from("player.stats*(u64)"),
            address: None,
            module_name: Some(String::from("game.exe")),
            offset: Some(0x10),
            metadata: std::collections::BTreeMap::default(),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_create_response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected pointer-backed project to load from disk.");
        let symbol_modules = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules();

        assert_eq!(symbol_modules[0].get_size(), 0x18);
        assert_eq!(symbol_modules[0].get_fields()[0].get_struct_layout_id(), "player.stats*(u64)");
    }
}
