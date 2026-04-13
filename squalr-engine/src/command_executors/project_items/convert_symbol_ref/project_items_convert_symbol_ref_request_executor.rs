use crate::command_executors::project_items::project_item_symbol_resolution::resolve_project_item_rooted_symbol;
use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::convert_symbol_ref::project_items_convert_symbol_ref_request::{
    ProjectItemSymbolRefConversionTarget, ProjectItemsConvertSymbolRefRequest,
};
use squalr_engine_api::commands::project_items::convert_symbol_ref::project_items_convert_symbol_ref_response::ProjectItemsConvertSymbolRefResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
    project_item_type_symbol_ref::ProjectItemTypeSymbolRef,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::projects::project_root_symbol::ProjectRootSymbol;
use squalr_engine_api::structures::projects::project_root_symbol_locator::ProjectRootSymbolLocator;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsConvertSymbolRefRequest {
    type ResponseType = ProjectItemsConvertSymbolRefResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.project_item_paths.is_empty() {
            return ProjectItemsConvertSymbolRefResponse {
                success: true,
                converted_project_item_count: 0,
            };
        }

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for symbol-ref conversion: {}", error);
                return ProjectItemsConvertSymbolRefResponse::default();
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            log::warn!("Cannot convert symbol-ref project items without an opened project.");
            return ProjectItemsConvertSymbolRefResponse::default();
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            log::error!("Failed to resolve opened project directory for symbol-ref conversion.");
            return ProjectItemsConvertSymbolRefResponse::default();
        };

        let project_symbol_catalog = opened_project
            .get_project_info()
            .get_project_symbol_catalog()
            .clone();
        let mut converted_project_item_count = 0_u64;

        for requested_project_item_path in &self.project_item_paths {
            let project_item_path = resolve_project_item_path(&project_directory_path, requested_project_item_path);
            let project_item_ref = ProjectItemRef::new(project_item_path.clone());
            let Some(project_item) = opened_project
                .get_project_items()
                .get(&project_item_ref)
                .cloned()
            else {
                log::warn!("Skipping symbol-ref conversion for missing project item: {:?}", project_item_path);
                continue;
            };

            let Some(rooted_symbol) = resolve_project_item_rooted_symbol(&project_symbol_catalog, &project_item).cloned() else {
                log::warn!("Skipping symbol-ref conversion for non-symbol project item: {:?}", project_item_path);
                continue;
            };
            let Some(replacement_project_item) = build_replacement_project_item(&project_item, &rooted_symbol, self.target) else {
                log::warn!(
                    "Skipping symbol-ref conversion for project item without {:?} conversion data: {:?}",
                    self.target,
                    project_item_path
                );
                continue;
            };
            let Some(project_item_to_replace) = opened_project
                .get_project_items_mut()
                .get_mut(&project_item_ref)
            else {
                continue;
            };

            *project_item_to_replace = replacement_project_item;
            converted_project_item_count = converted_project_item_count.saturating_add(1);
        }

        if converted_project_item_count == 0 {
            return ProjectItemsConvertSymbolRefResponse {
                success: true,
                converted_project_item_count,
            };
        }

        opened_project
            .get_project_info_mut()
            .set_has_unsaved_changes(true);

        if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
            log::error!("Failed to save project after symbol-ref conversion: {}", error);
            return ProjectItemsConvertSymbolRefResponse::default();
        }

        ProjectItemsConvertSymbolRefResponse {
            success: true,
            converted_project_item_count,
        }
    }
}

fn build_replacement_project_item(
    source_project_item: &ProjectItem,
    rooted_symbol: &ProjectRootSymbol,
    conversion_target: ProjectItemSymbolRefConversionTarget,
) -> Option<ProjectItem> {
    match conversion_target {
        ProjectItemSymbolRefConversionTarget::Address => Some(build_address_project_item(source_project_item, rooted_symbol)),
        ProjectItemSymbolRefConversionTarget::Pointer => build_pointer_project_item(source_project_item, rooted_symbol),
    }
}

fn build_address_project_item(
    source_project_item: &ProjectItem,
    rooted_symbol: &ProjectRootSymbol,
) -> ProjectItem {
    let (address, module_name) = match rooted_symbol.get_root_locator() {
        ProjectRootSymbolLocator::AbsoluteAddress { address } => (*address, String::new()),
        ProjectRootSymbolLocator::ModuleOffset { module_name, offset } => (*offset, module_name.clone()),
    };
    let mut address_project_item = ProjectItemTypeAddress::new_project_item(
        source_project_item.get_field_name().as_str(),
        address,
        &module_name,
        &source_project_item.get_field_description(),
        DataValue::new(DataTypeRef::new(rooted_symbol.get_struct_layout_id()), Vec::new()),
    );

    if source_project_item.get_is_activated() {
        address_project_item.toggle_activated();
    }

    ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(
        &mut address_project_item,
        &ProjectItemTypeSymbolRef::get_field_freeze_data_value_interpreter(source_project_item),
    );

    address_project_item
}

fn build_pointer_project_item(
    source_project_item: &ProjectItem,
    rooted_symbol: &ProjectRootSymbol,
) -> Option<ProjectItem> {
    let pointer = build_source_pointer(rooted_symbol)?;
    let mut pointer_project_item = ProjectItemTypePointer::new_project_item(
        source_project_item.get_field_name().as_str(),
        &pointer,
        &source_project_item.get_field_description(),
        rooted_symbol.get_struct_layout_id(),
    );

    if source_project_item.get_is_activated() {
        pointer_project_item.toggle_activated();
    }

    ProjectItemTypePointer::set_field_freeze_data_value_interpreter(
        &mut pointer_project_item,
        &ProjectItemTypeSymbolRef::get_field_freeze_data_value_interpreter(source_project_item),
    );
    ProjectItemTypePointer::set_field_evaluated_pointer_path(
        &mut pointer_project_item,
        rooted_symbol
            .get_metadata()
            .get("source.evaluated_pointer_path")
            .map(String::as_str)
            .unwrap_or_default(),
    );

    Some(pointer_project_item)
}

fn build_source_pointer(rooted_symbol: &ProjectRootSymbol) -> Option<Pointer> {
    let root_pointer_size = rooted_symbol
        .get_metadata()
        .get("source.pointer_size")
        .and_then(|pointer_size| pointer_size.parse::<PointerScanPointerSize>().ok())
        .unwrap_or_default();
    let pointer_offsets = rooted_symbol
        .get_metadata()
        .get("source.pointer_offsets")
        .and_then(|pointer_offsets| serde_json::from_str::<Vec<i64>>(pointer_offsets).ok())?;
    let (root_address, root_module_name) = resolve_source_pointer_root(rooted_symbol)?;

    Some(Pointer::new_with_size(root_address, pointer_offsets, root_module_name, root_pointer_size))
}

fn resolve_source_pointer_root(rooted_symbol: &ProjectRootSymbol) -> Option<(u64, String)> {
    let rooted_symbol_metadata = rooted_symbol.get_metadata();

    if let Some(root_module_name) = rooted_symbol_metadata.get("source.pointer_root_module") {
        let root_offset = rooted_symbol_metadata
            .get("source.pointer_root_offset")
            .and_then(|root_offset| parse_u64_string(root_offset))?;

        return Some((root_offset, root_module_name.clone()));
    }

    rooted_symbol_metadata
        .get("source.pointer_root")
        .and_then(|pointer_root| parse_pointer_root_display_text(pointer_root))
}

fn parse_pointer_root_display_text(pointer_root_display_text: &str) -> Option<(u64, String)> {
    let trimmed_pointer_root_display_text = pointer_root_display_text.trim();

    if let Some((module_name, root_offset_text)) = trimmed_pointer_root_display_text.rsplit_once("+0x") {
        let root_offset = u64::from_str_radix(root_offset_text, 16).ok()?;

        return Some((root_offset, module_name.to_string()));
    }

    parse_u64_string(trimmed_pointer_root_display_text).map(|root_address| (root_address, String::new()))
}

fn parse_u64_string(source: &str) -> Option<u64> {
    let trimmed_source = source.trim();

    if let Some(hexadecimal_source) = trimmed_source
        .strip_prefix("0x")
        .or_else(|| trimmed_source.strip_prefix("0X"))
    {
        u64::from_str_radix(hexadecimal_source, 16).ok()
    } else {
        u64::from_str_radix(trimmed_source, 16)
            .ok()
            .or_else(|| trimmed_source.parse::<u64>().ok())
    }
}

fn resolve_project_item_path(
    project_directory_path: &Path,
    project_item_path: &Path,
) -> PathBuf {
    if project_item_path.is_absolute() {
        project_item_path.to_path_buf()
    } else {
        project_directory_path.join(project_item_path)
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemsConvertSymbolRefRequest;
    use crate::command_executors::project_symbols::test_support::{MockProjectSymbolsBindings, create_engine_unprivileged_state};
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::commands::project_items::convert_symbol_ref::project_items_convert_symbol_ref_request::ProjectItemSymbolRefConversionTarget;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::projects::{
        project::Project, project_info::ProjectInfo, project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory,
        project_items::built_in_types::project_item_type_pointer::ProjectItemTypePointer,
        project_items::built_in_types::project_item_type_symbol_ref::ProjectItemTypeSymbolRef, project_items::project_item::ProjectItem,
        project_items::project_item_ref::ProjectItemRef, project_manifest::ProjectManifest, project_root_symbol::ProjectRootSymbol,
        project_symbol_catalog::ProjectSymbolCatalog,
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use std::{
        collections::HashMap,
        fs::File,
        path::{Path, PathBuf},
        sync::Arc,
    };

    fn create_project_with_symbol_ref_item(
        project_directory_path: &Path,
        project_item_file_name: &str,
        project_item: ProjectItem,
        project_symbol_catalog: ProjectSymbolCatalog,
    ) -> (Project, PathBuf) {
        let project_file_path = project_directory_path.join(Project::PROJECT_FILE);
        let root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
        let project_root_ref = ProjectItemRef::new(root_directory_path.clone());
        let project_item_path = root_directory_path.join(project_item_file_name);
        let project_item_ref = ProjectItemRef::new(project_item_path.clone());
        let project_info = ProjectInfo::new_with_symbol_catalog(project_file_path, None, ProjectManifest::default(), project_symbol_catalog);
        let mut project_items = HashMap::new();

        project_items.insert(project_root_ref.clone(), ProjectItemTypeDirectory::new_project_item(&project_root_ref));
        project_items.insert(project_item_ref, project_item);
        std::fs::create_dir_all(&root_directory_path).expect("Expected symbol-ref test project root directory to be created.");
        File::create(&project_item_path).expect("Expected symbol-ref test project item file to be created.");

        let mut project = Project::new(project_info, project_items, project_root_ref);
        project
            .save_to_path(project_directory_path, true)
            .expect("Expected symbol-ref test project to save.");

        (project, project_item_path)
    }

    #[test]
    fn convert_symbol_ref_request_rebuilds_address_item_from_rooted_symbol() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let symbol_ref_project_item = ProjectItemTypeSymbolRef::new_project_item("Health", "sym.health", "");
        let rooted_symbol = ProjectRootSymbol::new_module_offset(
            String::from("sym.health"),
            String::from("Health"),
            String::from("game.exe"),
            0x1234,
            String::from("u8"),
        );
        let (project, project_item_path) = create_project_with_symbol_ref_item(
            temp_directory.path(),
            "health.json",
            symbol_ref_project_item,
            ProjectSymbolCatalog::new_with_rooted_symbols(Vec::new(), vec![rooted_symbol]),
        );
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let convert_response = ProjectItemsConvertSymbolRefRequest {
            project_item_paths: vec![project_item_path.clone()],
            target: ProjectItemSymbolRefConversionTarget::Address,
        }
        .execute(&engine_execution_context);

        assert!(convert_response.success);
        assert_eq!(convert_response.converted_project_item_count, 1);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected converted project to load from disk.");
        let converted_project_item = loaded_project
            .get_project_items()
            .get(&ProjectItemRef::new(project_item_path))
            .expect("Expected converted project item to remain in the project.");

        assert_eq!(
            converted_project_item
                .get_item_type()
                .get_project_item_type_id(),
            super::ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID
        );
        let mut converted_project_item = converted_project_item.clone();
        assert_eq!(super::ProjectItemTypeAddress::get_field_address(&mut converted_project_item), 0x1234);
        assert_eq!(super::ProjectItemTypeAddress::get_field_module(&mut converted_project_item), "game.exe");
        assert_eq!(
            super::ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut converted_project_item).map(|symbolic_struct_reference| {
                symbolic_struct_reference
                    .get_symbolic_struct_namespace()
                    .to_string()
            }),
            Some(String::from("u8"))
        );
    }

    #[test]
    fn convert_symbol_ref_request_rebuilds_pointer_item_from_pointer_metadata() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let symbol_ref_project_item = ProjectItemTypeSymbolRef::new_project_item("Gold", "sym.player.gold", "");
        let mut rooted_symbol = ProjectRootSymbol::new_absolute_address(String::from("sym.player.gold"), String::from("Gold"), 0x2020, String::from("u32"));
        rooted_symbol
            .get_metadata_mut()
            .insert(String::from("source.pointer_root_module"), String::from("game.exe"));
        rooted_symbol
            .get_metadata_mut()
            .insert(String::from("source.pointer_root_offset"), String::from("0x1000"));
        rooted_symbol
            .get_metadata_mut()
            .insert(String::from("source.pointer_offsets"), String::from("[32]"));
        rooted_symbol
            .get_metadata_mut()
            .insert(String::from("source.pointer_size"), String::from("u64"));
        rooted_symbol
            .get_metadata_mut()
            .insert(String::from("source.evaluated_pointer_path"), String::from("game.exe+0x1000 -> 0x2020"));
        let (project, project_item_path) = create_project_with_symbol_ref_item(
            temp_directory.path(),
            "gold.json",
            symbol_ref_project_item,
            ProjectSymbolCatalog::new_with_rooted_symbols(Vec::new(), vec![rooted_symbol]),
        );
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let convert_response = ProjectItemsConvertSymbolRefRequest {
            project_item_paths: vec![project_item_path.clone()],
            target: ProjectItemSymbolRefConversionTarget::Pointer,
        }
        .execute(&engine_execution_context);

        assert!(convert_response.success);
        assert_eq!(convert_response.converted_project_item_count, 1);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected converted project to load from disk.");
        let converted_project_item = loaded_project
            .get_project_items()
            .get(&ProjectItemRef::new(project_item_path))
            .expect("Expected converted project item to remain in the project.");

        assert_eq!(
            converted_project_item
                .get_item_type()
                .get_project_item_type_id(),
            ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID
        );
        let rebuilt_pointer = ProjectItemTypePointer::get_field_pointer(converted_project_item);

        assert_eq!(rebuilt_pointer.get_module_name(), "game.exe");
        assert_eq!(rebuilt_pointer.get_address(), 0x1000);
        assert_eq!(rebuilt_pointer.get_offsets(), &[0x20]);
        assert_eq!(rebuilt_pointer.get_pointer_size(), super::PointerScanPointerSize::Pointer64);
        assert_eq!(
            ProjectItemTypePointer::get_field_evaluated_pointer_path(converted_project_item),
            "game.exe+0x1000 -> 0x2020"
        );
    }

    #[test]
    fn convert_symbol_ref_request_skips_pointer_conversion_without_pointer_metadata() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let symbol_ref_project_item = ProjectItemTypeSymbolRef::new_project_item("Health", "sym.health", "");
        let rooted_symbol = ProjectRootSymbol::new_absolute_address(String::from("sym.health"), String::from("Health"), 0x1234, String::from("u8"));
        let (project, project_item_path) = create_project_with_symbol_ref_item(
            temp_directory.path(),
            "health.json",
            symbol_ref_project_item,
            ProjectSymbolCatalog::new_with_rooted_symbols(Vec::new(), vec![rooted_symbol]),
        );
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let convert_response = ProjectItemsConvertSymbolRefRequest {
            project_item_paths: vec![project_item_path.clone()],
            target: ProjectItemSymbolRefConversionTarget::Pointer,
        }
        .execute(&engine_execution_context);

        assert!(convert_response.success);
        assert_eq!(convert_response.converted_project_item_count, 0);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected converted project to load from disk.");
        let untouched_project_item = loaded_project
            .get_project_items()
            .get(&ProjectItemRef::new(project_item_path))
            .expect("Expected untouched project item to remain in the project.");

        assert_eq!(
            untouched_project_item
                .get_item_type()
                .get_project_item_type_id(),
            ProjectItemTypeSymbolRef::PROJECT_ITEM_TYPE_ID
        );
    }
}
