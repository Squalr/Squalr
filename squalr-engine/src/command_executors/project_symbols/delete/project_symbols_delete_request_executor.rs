use crate::command_executors::project_symbols::project_symbol_store_mutation::save_and_sync_project_symbol_catalog;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::{ProjectSymbolsDeleteModuleRange, ProjectSymbolsDeleteRequest};
use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_response::ProjectSymbolsDeleteResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_symbol_locator::ProjectSymbolLocator;
use std::collections::HashSet;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectSymbolsDeleteRequest {
    type ResponseType = ProjectSymbolsDeleteResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.symbol_locator_keys.is_empty() && self.module_names.is_empty() && self.module_ranges.is_empty() {
            return ProjectSymbolsDeleteResponse {
                success: true,
                deleted_symbol_count: 0,
                deleted_module_count: 0,
                deleted_module_range_count: 0,
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
        let mut module_ranges = self
            .module_ranges
            .iter()
            .filter_map(normalize_delete_module_range)
            .collect::<Vec<_>>();
        module_ranges.sort_by(|left_range, right_range| {
            right_range
                .module_name
                .cmp(&left_range.module_name)
                .then_with(|| right_range.offset.cmp(&left_range.offset))
        });
        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();
        let symbol_modules = project_symbol_catalog.get_symbol_modules_mut();
        let symbol_module_count_before_delete = symbol_modules.len();

        symbol_modules.retain(|symbol_module| !module_name_set.contains(symbol_module.get_module_name()));

        let deleted_module_count = symbol_module_count_before_delete.saturating_sub(symbol_modules.len()) as u64;
        let deleted_module_field_count = delete_module_fields_by_locator_key(project_symbol_catalog, &symbol_locator_key_set);
        let deleted_module_range_count = apply_module_range_deletes(project_symbol_catalog, &module_ranges, &module_name_set);
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

        let deleted_symbol_count = symbol_claim_count_before_delete.saturating_sub(symbol_claims.len()) as u64 + deleted_module_field_count;

        if deleted_symbol_count == 0 && deleted_module_count == 0 && deleted_module_range_count == 0 {
            return ProjectSymbolsDeleteResponse {
                success: true,
                deleted_symbol_count: 0,
                deleted_module_count: 0,
                deleted_module_range_count: 0,
            };
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsDeleteResponse {
                success: false,
                deleted_symbol_count,
                deleted_module_count,
                deleted_module_range_count,
            };
        }

        ProjectSymbolsDeleteResponse {
            success: true,
            deleted_symbol_count,
            deleted_module_count,
            deleted_module_range_count,
        }
    }
}

fn delete_module_fields_by_locator_key(
    project_symbol_catalog: &mut squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog,
    symbol_locator_key_set: &HashSet<String>,
) -> u64 {
    let mut deleted_module_field_count = 0_u64;

    for symbol_module in project_symbol_catalog.get_symbol_modules_mut() {
        let module_name = symbol_module.get_module_name().to_string();
        let module_field_count_before_delete = symbol_module.get_fields().len();

        symbol_module
            .get_fields_mut()
            .retain(|module_field| !symbol_locator_key_set.contains(&module_field.get_symbol_locator_key(&module_name)));
        deleted_module_field_count =
            deleted_module_field_count.saturating_add(module_field_count_before_delete.saturating_sub(symbol_module.get_fields().len()) as u64);
    }

    deleted_module_field_count
}

fn normalize_delete_module_range(project_symbols_delete_module_range: &ProjectSymbolsDeleteModuleRange) -> Option<ProjectSymbolsDeleteModuleRange> {
    let module_name = project_symbols_delete_module_range.module_name.trim();

    if module_name.is_empty() || project_symbols_delete_module_range.length == 0 {
        return None;
    }

    Some(ProjectSymbolsDeleteModuleRange {
        module_name: module_name.to_string(),
        offset: project_symbols_delete_module_range.offset,
        length: project_symbols_delete_module_range.length,
    })
}

fn apply_module_range_deletes(
    project_symbol_catalog: &mut squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog,
    module_ranges: &[ProjectSymbolsDeleteModuleRange],
    deleted_module_names: &HashSet<String>,
) -> u64 {
    let mut deleted_module_range_count = 0_u64;

    for module_range in module_ranges {
        if deleted_module_names.contains(&module_range.module_name) {
            continue;
        }

        let Some(symbol_module) = project_symbol_catalog.find_symbol_module_mut(&module_range.module_name) else {
            continue;
        };
        let module_size = symbol_module.get_size();

        if module_range.offset >= module_size {
            continue;
        }

        let deleted_length = module_range
            .length
            .min(module_size.saturating_sub(module_range.offset));

        if deleted_length == 0 {
            continue;
        }

        symbol_module.set_size(module_size.saturating_sub(deleted_length));
        delete_or_shift_module_fields(symbol_module.get_fields_mut(), module_range, deleted_length);
        delete_or_shift_module_symbol_claims(project_symbol_catalog.get_symbol_claims_mut(), module_range, deleted_length);
        deleted_module_range_count = deleted_module_range_count.saturating_add(1);
    }

    deleted_module_range_count
}

fn delete_or_shift_module_fields(
    module_fields: &mut Vec<squalr_engine_api::structures::projects::project_symbol_module_field::ProjectSymbolModuleField>,
    module_range: &ProjectSymbolsDeleteModuleRange,
    deleted_length: u64,
) {
    let deleted_range_end = module_range.offset.saturating_add(deleted_length);

    module_fields.retain_mut(|module_field| {
        let offset = module_field.get_offset();

        if offset >= module_range.offset && offset < deleted_range_end {
            return false;
        }

        if offset >= deleted_range_end {
            module_field.set_offset(offset.saturating_sub(deleted_length));
        }

        true
    });
}

fn delete_or_shift_module_symbol_claims(
    symbol_claims: &mut Vec<squalr_engine_api::structures::projects::project_symbol_claim::ProjectSymbolClaim>,
    module_range: &ProjectSymbolsDeleteModuleRange,
    deleted_length: u64,
) {
    let deleted_range_end = module_range.offset.saturating_add(deleted_length);

    symbol_claims.retain_mut(|symbol_claim| {
        let ProjectSymbolLocator::ModuleOffset { module_name, offset } = symbol_claim.get_locator_mut() else {
            return true;
        };

        if module_name != &module_range.module_name {
            return true;
        }

        if *offset >= module_range.offset && *offset < deleted_range_end {
            return false;
        }

        if *offset >= deleted_range_end {
            *offset = offset.saturating_sub(deleted_length);
        }

        true
    });
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
        project_symbol_module_field::ProjectSymbolModuleField,
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
            module_ranges: Vec::new(),
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
    fn delete_project_symbols_request_removes_matching_module_field() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("First"), 0x04, String::from("u8[4]")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Second"), 0x08, String::from("u8[4]")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());
        let project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_symbols_delete_response = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: vec![String::from("module:game.exe:4")],
            module_names: Vec::new(),
            module_ranges: Vec::new(),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_delete_response.success);
        assert_eq!(project_symbols_delete_response.deleted_symbol_count, 1);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected deleted-module-field project to load from disk.");
        let symbol_modules = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules();

        assert_eq!(symbol_modules[0].get_fields().len(), 1);
        assert_eq!(symbol_modules[0].get_fields()[0].get_display_name(), "Second");
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
            module_ranges: Vec::new(),
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

    #[test]
    fn delete_project_symbols_request_removes_module_range_and_shifts_later_claims() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(String::from("game.exe"), 0x20)],
            Vec::new(),
            vec![
                ProjectSymbolClaim::new_module_offset(String::from("Health"), String::from("game.exe"), 0x04, String::from("u32")),
                ProjectSymbolClaim::new_module_offset(String::from("Ammo"), String::from("game.exe"), 0x10, String::from("u32")),
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
            module_names: Vec::new(),
            module_ranges: vec![
                squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteModuleRange {
                    module_name: String::from("game.exe"),
                    offset: 0x04,
                    length: 0x04,
                },
            ],
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_delete_response.success);
        assert_eq!(project_symbols_delete_response.deleted_module_range_count, 1);
        assert_eq!(project_symbols_delete_response.deleted_symbol_count, 0);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected range-deleted project to load from disk.");
        let project_symbol_catalog = loaded_project.get_project_info().get_project_symbol_catalog();
        let symbol_modules = project_symbol_catalog.get_symbol_modules();
        let symbol_claims = project_symbol_catalog.get_symbol_claims();

        assert_eq!(symbol_modules[0].get_size(), 0x1C);
        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(symbol_claims[0].get_display_name(), "Ammo");
        assert_eq!(symbol_claims[0].get_symbol_locator_key(), "module:game.exe:C");

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_modules(), symbol_modules);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
    }
}
