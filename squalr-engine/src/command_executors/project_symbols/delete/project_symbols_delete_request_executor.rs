use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::command_executors::{
    project_items::convert_symbol_ref::project_items_convert_symbol_ref_request_executor::build_symbol_ref_replacement_project_item,
    project_symbols::{project_symbol_layout_mutation::ProjectSymbolLayoutMutation, project_symbol_store_mutation::save_and_sync_project_symbol_catalog},
};
use squalr_engine_api::commands::project_items::convert_symbol_ref::project_items_convert_symbol_ref_request::ProjectItemSymbolRefConversionTarget;
use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::{ProjectSymbolsDeleteModuleRange, ProjectSymbolsDeleteRequest};
use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_response::ProjectSymbolsDeleteResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_symbol_ref::ProjectItemTypeSymbolRef;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::projects::project_symbol_claim::ProjectSymbolClaim;
use squalr_engine_api::structures::projects::project_symbol_locator::ProjectSymbolLocator;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone)]
struct SymbolRefConversionPlan {
    project_item_ref: ProjectItemRef,
    replacement_project_item: ProjectItem,
}

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
                converted_symbol_ref_count: 0,
                blocked_symbol_ref_count: 0,
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
        let project_symbol_catalog_snapshot = opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .clone();
        let target_symbol_claims_by_locator_key =
            collect_symbol_ref_delete_targets(&project_symbol_catalog_snapshot, &symbol_locator_key_set, &module_name_set, &module_ranges);
        let (symbol_ref_conversion_plan, blocked_symbol_ref_count) = build_symbol_ref_conversion_plan(opened_project, &target_symbol_claims_by_locator_key);
        let referenced_symbol_ref_count = symbol_ref_conversion_plan
            .len()
            .saturating_add(blocked_symbol_ref_count);

        if referenced_symbol_ref_count > 0 && !self.convert_symbol_refs {
            log::warn!(
                "Project symbol delete blocked because {} symbol-ref project item(s) still reference the selected symbol(s).",
                referenced_symbol_ref_count
            );
            return ProjectSymbolsDeleteResponse {
                success: false,
                deleted_symbol_count: 0,
                deleted_module_count: 0,
                deleted_module_range_count: 0,
                converted_symbol_ref_count: 0,
                blocked_symbol_ref_count: referenced_symbol_ref_count as u64,
            };
        }

        if blocked_symbol_ref_count > 0 {
            log::warn!(
                "Project symbol delete blocked because {} symbol-ref project item(s) could not be converted before deletion.",
                blocked_symbol_ref_count
            );
            return ProjectSymbolsDeleteResponse {
                success: false,
                deleted_symbol_count: 0,
                deleted_module_count: 0,
                deleted_module_range_count: 0,
                converted_symbol_ref_count: 0,
                blocked_symbol_ref_count: blocked_symbol_ref_count as u64,
            };
        }

        let converted_symbol_ref_count = apply_symbol_ref_conversion_plan(opened_project, &symbol_ref_conversion_plan);

        if converted_symbol_ref_count != symbol_ref_conversion_plan.len() {
            let failed_conversion_count = symbol_ref_conversion_plan
                .len()
                .saturating_sub(converted_symbol_ref_count);
            log::warn!(
                "Project symbol delete blocked because {} symbol-ref project item(s) disappeared before conversion could complete.",
                failed_conversion_count
            );
            return ProjectSymbolsDeleteResponse {
                success: false,
                deleted_symbol_count: 0,
                deleted_module_count: 0,
                deleted_module_range_count: 0,
                converted_symbol_ref_count: converted_symbol_ref_count as u64,
                blocked_symbol_ref_count: failed_conversion_count as u64,
            };
        }

        let project_symbol_catalog = opened_project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut();
        let symbol_modules = project_symbol_catalog.get_symbol_modules_mut();
        let symbol_module_count_before_delete = symbol_modules.len();

        symbol_modules.retain(|symbol_module| !module_name_set.contains(symbol_module.get_module_name()));

        let deleted_module_count = symbol_module_count_before_delete.saturating_sub(symbol_modules.len()) as u64;
        let delete_module_field_summary = ProjectSymbolLayoutMutation::delete_module_fields_by_locator_key(project_symbol_catalog, &symbol_locator_key_set);
        let delete_module_range_summary = ProjectSymbolLayoutMutation::delete_module_ranges(project_symbol_catalog, &module_ranges, &module_name_set);
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

        let deleted_symbol_count =
            symbol_claim_count_before_delete.saturating_sub(symbol_claims.len()) as u64 + delete_module_field_summary.get_deleted_module_field_count();
        let deleted_module_range_count = delete_module_range_summary.get_deleted_module_range_count();

        if deleted_symbol_count == 0 && deleted_module_count == 0 && deleted_module_range_count == 0 && converted_symbol_ref_count == 0 {
            return ProjectSymbolsDeleteResponse {
                success: true,
                deleted_symbol_count: 0,
                deleted_module_count: 0,
                deleted_module_range_count: 0,
                converted_symbol_ref_count: converted_symbol_ref_count as u64,
                blocked_symbol_ref_count: 0,
            };
        }

        if !save_and_sync_project_symbol_catalog(engine_unprivileged_state, opened_project, &project_directory_path) {
            return ProjectSymbolsDeleteResponse {
                success: false,
                deleted_symbol_count,
                deleted_module_count,
                deleted_module_range_count,
                converted_symbol_ref_count: converted_symbol_ref_count as u64,
                blocked_symbol_ref_count: 0,
            };
        }

        ProjectSymbolsDeleteResponse {
            success: true,
            deleted_symbol_count,
            deleted_module_count,
            deleted_module_range_count,
            converted_symbol_ref_count: converted_symbol_ref_count as u64,
            blocked_symbol_ref_count: 0,
        }
    }
}

fn collect_symbol_ref_delete_targets(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_locator_key_set: &HashSet<String>,
    module_name_set: &HashSet<String>,
    module_ranges: &[ProjectSymbolsDeleteModuleRange],
) -> HashMap<String, ProjectSymbolClaim> {
    let mut target_symbol_claims = HashMap::new();

    for symbol_locator_key in symbol_locator_key_set {
        if let Some(symbol_claim) = resolve_symbol_ref_conversion_claim(project_symbol_catalog, symbol_locator_key) {
            target_symbol_claims.insert(symbol_locator_key.to_string(), symbol_claim);
        }
    }

    for symbol_claim in project_symbol_catalog.get_symbol_claims() {
        let ProjectSymbolLocator::ModuleOffset { module_name, .. } = symbol_claim.get_locator() else {
            continue;
        };

        if module_name_set.contains(module_name)
            || module_ranges
                .iter()
                .filter(|module_range| module_range_can_mutate_symbols(project_symbol_catalog, module_range))
                .any(|module_range| symbol_claim_matches_module_range(symbol_claim, module_range))
        {
            target_symbol_claims.insert(symbol_claim.get_symbol_locator_key(), symbol_claim.clone());
        }
    }

    for symbol_module in project_symbol_catalog.get_symbol_modules() {
        let module_name = symbol_module.get_module_name();
        for module_field in symbol_module.get_fields() {
            let symbol_locator_key = module_field.get_symbol_locator_key(module_name);
            let module_field_claim = ProjectSymbolClaim::new_module_offset(
                module_field.get_display_name().to_string(),
                module_name.to_string(),
                module_field.get_offset(),
                module_field.get_struct_layout_id().to_string(),
            );

            if module_name_set.contains(module_name)
                || module_ranges
                    .iter()
                    .filter(|module_range| module_range_can_mutate_symbols(project_symbol_catalog, module_range))
                    .any(|module_range| symbol_claim_matches_module_range(&module_field_claim, module_range))
            {
                target_symbol_claims.insert(symbol_locator_key, module_field_claim);
            }
        }
    }

    target_symbol_claims
}

fn module_range_can_mutate_symbols(
    project_symbol_catalog: &ProjectSymbolCatalog,
    module_range: &ProjectSymbolsDeleteModuleRange,
) -> bool {
    project_symbol_catalog
        .find_symbol_module(&module_range.module_name)
        .map(|symbol_module| module_range.offset < symbol_module.get_size())
        .unwrap_or(false)
}

fn resolve_symbol_ref_conversion_claim(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_locator_key: &str,
) -> Option<ProjectSymbolClaim> {
    project_symbol_catalog.resolve_symbol_claim(symbol_locator_key)
}

fn symbol_claim_matches_module_range(
    symbol_claim: &ProjectSymbolClaim,
    module_range: &ProjectSymbolsDeleteModuleRange,
) -> bool {
    let ProjectSymbolLocator::ModuleOffset { module_name, offset } = symbol_claim.get_locator() else {
        return false;
    };

    if module_name != &module_range.module_name {
        return false;
    }

    let deleted_range_end = module_range.offset.saturating_add(module_range.length);

    match module_range.mode {
        squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteModuleRangeMode::ShiftLeft => {
            *offset >= module_range.offset
        }
        squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteModuleRangeMode::ReplaceWithU8 => {
            *offset >= module_range.offset && *offset < deleted_range_end
        }
    }
}

fn build_symbol_ref_conversion_plan(
    opened_project: &Project,
    target_symbol_claims_by_locator_key: &HashMap<String, ProjectSymbolClaim>,
) -> (Vec<SymbolRefConversionPlan>, usize) {
    let mut symbol_ref_conversion_plan = Vec::new();
    let mut blocked_symbol_ref_count = 0_usize;

    for (project_item_ref, project_item) in opened_project.get_project_items() {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID {
            continue;
        }

        let symbol_locator_key = ProjectItemTypeSymbolRef::get_field_symbol_locator_key(project_item);
        let Some(symbol_claim) = target_symbol_claims_by_locator_key.get(symbol_locator_key.trim()) else {
            continue;
        };
        let Some(replacement_project_item) =
            build_symbol_ref_replacement_project_item(project_item, symbol_claim, ProjectItemSymbolRefConversionTarget::Inferred)
        else {
            blocked_symbol_ref_count = blocked_symbol_ref_count.saturating_add(1);
            continue;
        };

        symbol_ref_conversion_plan.push(SymbolRefConversionPlan {
            project_item_ref: project_item_ref.clone(),
            replacement_project_item,
        });
    }

    (symbol_ref_conversion_plan, blocked_symbol_ref_count)
}

fn apply_symbol_ref_conversion_plan(
    opened_project: &mut Project,
    symbol_ref_conversion_plan: &[SymbolRefConversionPlan],
) -> usize {
    let mut converted_symbol_ref_count = 0_usize;

    for conversion_plan in symbol_ref_conversion_plan {
        let Some(project_item) = opened_project
            .get_project_items_mut()
            .get_mut(&conversion_plan.project_item_ref)
        else {
            continue;
        };

        *project_item = conversion_plan.replacement_project_item.clone();
        converted_symbol_ref_count = converted_symbol_ref_count.saturating_add(1);
    }

    converted_symbol_ref_count
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
        mode: project_symbols_delete_module_range.mode,
    })
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsDeleteRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::{
        ProjectSymbolsDeleteModuleRange, ProjectSymbolsDeleteModuleRangeMode,
    };
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{
        project::Project,
        project_items::built_in_types::{project_item_type_address::ProjectItemTypeAddress, project_item_type_symbol_ref::ProjectItemTypeSymbolRef},
        project_items::project_item_ref::ProjectItemRef,
        project_symbol_catalog::ProjectSymbolCatalog,
        project_symbol_claim::ProjectSymbolClaim,
        project_symbol_module::ProjectSymbolModule,
        project_symbol_module_field::ProjectSymbolModuleField,
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::{fs::File, sync::Arc};

    fn add_symbol_ref_project_item(
        project: &mut Project,
        project_directory_path: &std::path::Path,
        project_item_file_name: &str,
        symbol_locator_key: &str,
    ) -> std::path::PathBuf {
        let project_item_path = project_directory_path
            .join(Project::PROJECT_DIR)
            .join(project_item_file_name);
        let project_item_ref = ProjectItemRef::new(project_item_path.clone());
        let parent_directory_path = project_item_path
            .parent()
            .expect("Expected project item test path to have a parent.");

        std::fs::create_dir_all(parent_directory_path).expect("Expected symbol-ref project item parent directory to be created.");
        File::create(&project_item_path).expect("Expected symbol-ref project item file to be created.");
        project
            .get_project_items_mut()
            .insert(project_item_ref, ProjectItemTypeSymbolRef::new_project_item("Health", symbol_locator_key, ""));

        project_item_path
    }

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
            convert_symbol_refs: false,
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
    fn delete_project_symbols_request_blocks_referenced_symbol_ref_without_conversion() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Health"),
                0x1234,
                String::from("u32"),
            )],
        );
        let mut project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);

        add_symbol_ref_project_item(&mut project, temp_directory.path(), "health.json", "absolute:1234");

        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

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
            convert_symbol_refs: false,
        }
        .execute(&engine_execution_context);

        assert!(!project_symbols_delete_response.success);
        assert_eq!(project_symbols_delete_response.deleted_symbol_count, 0);
        assert_eq!(project_symbols_delete_response.blocked_symbol_ref_count, 1);

        let opened_project = engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project_guard = opened_project
            .read()
            .expect("Expected opened project read lock in test.");
        let opened_project = opened_project_guard
            .as_ref()
            .expect("Expected opened project to remain loaded.");

        assert_eq!(
            opened_project
                .get_project_info()
                .get_project_symbol_catalog()
                .get_symbol_claims()
                .len(),
            1
        );
    }

    #[test]
    fn delete_project_symbols_request_converts_referenced_symbol_ref_before_delete() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Health"),
                0x1234,
                String::from("u32"),
            )],
        );
        let mut project = create_project_with_symbol_catalog(temp_directory.path(), project_symbol_catalog);
        let project_item_path = add_symbol_ref_project_item(&mut project, temp_directory.path(), "health.json", "absolute:1234");
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

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
            convert_symbol_refs: true,
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_delete_response.success);
        assert_eq!(project_symbols_delete_response.deleted_symbol_count, 1);
        assert_eq!(project_symbols_delete_response.converted_symbol_ref_count, 1);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected converted-and-deleted project to load from disk.");

        assert!(
            loaded_project
                .get_project_info()
                .get_project_symbol_catalog()
                .get_symbol_claims()
                .is_empty()
        );

        let converted_project_item = loaded_project
            .get_project_items()
            .get(&ProjectItemRef::new(project_item_path))
            .expect("Expected converted project item to remain in the project.");

        assert_eq!(
            converted_project_item
                .get_item_type()
                .get_project_item_type_id(),
            ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID
        );

        let mut converted_project_item = converted_project_item.clone();

        assert_eq!(ProjectItemTypeAddress::get_field_address(&mut converted_project_item), 0x1234);
        assert_eq!(
            ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut converted_project_item)
                .map(|symbolic_struct_ref| symbolic_struct_ref.get_symbolic_struct_namespace().to_string()),
            Some(String::from("u32"))
        );
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
            convert_symbol_refs: false,
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
            convert_symbol_refs: false,
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
                    mode: ProjectSymbolsDeleteModuleRangeMode::ShiftLeft,
                },
            ],
            convert_symbol_refs: false,
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

    #[test]
    fn delete_project_symbols_request_replaces_module_range_with_merged_u8_field() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("prefix"), 0x00, String::from("u8[4]")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Health"), 0x04, String::from("u32")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("suffix"), 0x08, String::from("u8[8]")));
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
            symbol_locator_keys: Vec::new(),
            module_names: Vec::new(),
            module_ranges: vec![ProjectSymbolsDeleteModuleRange {
                module_name: String::from("game.exe"),
                offset: 0x04,
                length: 0x04,
                mode: ProjectSymbolsDeleteModuleRangeMode::ReplaceWithU8,
            }],
            convert_symbol_refs: false,
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_delete_response.success);
        assert_eq!(project_symbols_delete_response.deleted_module_range_count, 1);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected replaced-field project to load from disk.");
        let symbol_modules = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules();
        let module_fields = symbol_modules[0].get_fields();

        assert_eq!(symbol_modules[0].get_size(), 0x20);
        assert_eq!(module_fields.len(), 1);
        assert_eq!(module_fields[0].get_offset(), 0x00);
        assert_eq!(module_fields[0].get_struct_layout_id(), "u8[16]");
    }
}
