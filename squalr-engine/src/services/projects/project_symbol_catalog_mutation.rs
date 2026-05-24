use crate::services::projects::{project_symbol_layout_mutation::ProjectSymbolLayoutMutation, project_symbol_name_scope::ProjectSymbolNameScope};
use squalr_engine_api::commands::project_symbols::{
    create::project_symbols_create_request::ProjectSymbolsCreateRequest,
    delete::project_symbols_delete_request::{ProjectSymbolsDeleteModuleRange, ProjectSymbolsDeleteRequest},
    update::project_symbols_update_request::ProjectSymbolsUpdateRequest,
};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectSymbolsDeleteMutationSummary {
    pub deleted_symbol_count: u64,
    pub deleted_module_count: u64,
    pub deleted_module_range_count: u64,
}

impl ProjectSymbolsDeleteMutationSummary {
    pub fn did_delete_anything(&self) -> bool {
        self.deleted_symbol_count > 0 || self.deleted_module_count > 0 || self.deleted_module_range_count > 0
    }
}

pub fn create_project_symbol(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    project_symbols_create_request: &ProjectSymbolsCreateRequest,
) -> Result<String, String> {
    let locator =
        build_create_locator(project_symbols_create_request).ok_or_else(|| String::from("Project-symbols create request did not provide a valid locator."))?;
    let trimmed_display_name = project_symbols_create_request.display_name.trim();

    if trimmed_display_name.is_empty()
        || project_symbols_create_request
            .struct_layout_id
            .trim()
            .is_empty()
    {
        return Err(String::from("Project-symbols create request requires a non-empty display name and type id."));
    }

    let struct_layout_id = project_symbols_create_request
        .struct_layout_id
        .trim()
        .to_string();
    let created_symbol_locator_key = locator.to_locator_key();
    let display_name = ProjectSymbolNameScope::deduplicate_display_name(
        project_symbol_catalog,
        project_symbol_catalog.get_symbol_claims(),
        &locator,
        trimmed_display_name,
        None,
    );
    let local_struct_layout_descriptors = project_symbol_catalog.get_struct_layout_descriptors().to_vec();
    let resolve_field_size_in_bytes =
        |struct_layout_id: &str| resolve_struct_layout_id_size_in_bytes(engine_execution_context, &local_struct_layout_descriptors, struct_layout_id);

    match locator {
        ProjectSymbolLocator::ModuleOffset { module_name, offset } => {
            ProjectSymbolLayoutMutation::upsert_module_field(
                project_symbol_catalog,
                &module_name,
                display_name,
                offset,
                struct_layout_id,
                resolve_field_size_in_bytes,
            )?;
        }
        ProjectSymbolLocator::AbsoluteAddress { .. } => {
            let mut created_symbol = ProjectSymbolClaim::new(display_name, locator, struct_layout_id);
            *created_symbol.get_metadata_mut() = project_symbols_create_request.metadata.clone();
            project_symbol_catalog
                .get_symbol_claims_mut()
                .push(created_symbol);
        }
    }

    Ok(created_symbol_locator_key)
}

pub fn update_project_symbol(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    project_symbols_update_request: &ProjectSymbolsUpdateRequest,
) -> Result<(), String> {
    let trimmed_display_name = project_symbols_update_request
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|display_name| !display_name.is_empty())
        .map(str::to_string);
    let trimmed_struct_layout_id = project_symbols_update_request
        .struct_layout_id
        .as_deref()
        .map(str::trim)
        .filter(|struct_layout_id| !struct_layout_id.is_empty())
        .map(str::to_string);

    if trimmed_display_name.is_none() && trimmed_struct_layout_id.is_none() {
        return Err(String::from("Project-symbols update request requires at least one non-empty update field."));
    }

    let local_struct_layout_descriptors = project_symbol_catalog.get_struct_layout_descriptors().to_vec();
    let resolve_field_size_in_bytes =
        |struct_layout_id: &str| resolve_struct_layout_id_size_in_bytes(engine_execution_context, &local_struct_layout_descriptors, struct_layout_id);
    let did_update = if let Some(symbol_claim) = project_symbol_catalog.find_symbol_claim(&project_symbols_update_request.symbol_locator_key) {
        let locator = symbol_claim.get_locator().clone();
        let deduplicated_display_name = trimmed_display_name.as_ref().map(|display_name| {
            ProjectSymbolNameScope::deduplicate_display_name(
                project_symbol_catalog,
                project_symbol_catalog.get_symbol_claims(),
                &locator,
                display_name,
                Some(&project_symbols_update_request.symbol_locator_key),
            )
        });

        if let Some(symbol_claim) = project_symbol_catalog.find_symbol_claim_mut(&project_symbols_update_request.symbol_locator_key) {
            if let Some(display_name) = deduplicated_display_name {
                symbol_claim.set_display_name(display_name);
            }

            if let Some(struct_layout_id) = trimmed_struct_layout_id.as_ref() {
                symbol_claim.set_struct_layout_id(struct_layout_id.clone());
            }

            true
        } else {
            false
        }
    } else if let Some((symbol_module, module_field)) = project_symbol_catalog.find_module_field(&project_symbols_update_request.symbol_locator_key) {
        let module_name = symbol_module.get_module_name().to_string();
        let locator = ProjectSymbolLocator::new_module_offset(module_name.clone(), module_field.get_offset());
        let display_name = trimmed_display_name
            .clone()
            .unwrap_or_else(|| module_field.get_display_name().to_string());
        let display_name = ProjectSymbolNameScope::deduplicate_display_name(
            project_symbol_catalog,
            project_symbol_catalog.get_symbol_claims(),
            &locator,
            &display_name,
            Some(&project_symbols_update_request.symbol_locator_key),
        );
        let offset = module_field.get_offset();
        let struct_layout_id = trimmed_struct_layout_id
            .clone()
            .unwrap_or_else(|| module_field.get_struct_layout_id().to_string());

        ProjectSymbolLayoutMutation::upsert_module_field(
            project_symbol_catalog,
            &module_name,
            display_name,
            offset,
            struct_layout_id,
            resolve_field_size_in_bytes,
        )
        .map(|_| true)?
    } else {
        false
    };

    if did_update {
        Ok(())
    } else {
        Err(format!(
            "Project-symbols update request could not find symbol locator key '{}'.",
            project_symbols_update_request.symbol_locator_key
        ))
    }
}

pub fn delete_project_symbols(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    project_symbols_delete_request: &ProjectSymbolsDeleteRequest,
) -> ProjectSymbolsDeleteMutationSummary {
    let symbol_locator_key_set = project_symbols_delete_request
        .symbol_locator_keys
        .iter()
        .map(|symbol_locator_key| symbol_locator_key.trim())
        .filter(|symbol_locator_key| !symbol_locator_key.is_empty())
        .map(str::to_string)
        .collect::<HashSet<String>>();
    let module_name_set = project_symbols_delete_request
        .module_names
        .iter()
        .map(|module_name| module_name.trim())
        .filter(|module_name| !module_name.is_empty())
        .map(str::to_string)
        .collect::<HashSet<String>>();
    let mut module_ranges = project_symbols_delete_request
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

    let (deleted_module_names, deleted_module_count) = {
        let symbol_modules = project_symbol_catalog.get_symbol_modules_mut();
        let symbol_module_count_before_delete = symbol_modules.len();
        let deleted_module_names = symbol_modules
            .iter()
            .filter(|symbol_module| module_name_set.contains(symbol_module.get_module_name()))
            .map(|symbol_module| symbol_module.get_module_name().to_string())
            .collect::<HashSet<String>>();

        symbol_modules.retain(|symbol_module| !module_name_set.contains(symbol_module.get_module_name()));

        (
            deleted_module_names,
            symbol_module_count_before_delete.saturating_sub(symbol_modules.len()) as u64,
        )
    };

    project_symbol_catalog.delete_module_root_struct_layouts(&deleted_module_names);

    let delete_module_field_summary = ProjectSymbolLayoutMutation::delete_module_fields_by_locator_key(project_symbol_catalog, &symbol_locator_key_set);
    let delete_module_range_summary = ProjectSymbolLayoutMutation::delete_module_ranges(project_symbol_catalog, &module_ranges, &module_name_set);
    let symbol_claims = project_symbol_catalog.get_symbol_claims_mut();
    let symbol_claim_count_before_delete = symbol_claims.len();

    symbol_claims.retain(|symbol_claim| {
        if symbol_locator_key_set.contains(&symbol_claim.get_symbol_locator_key()) {
            return false;
        }

        match symbol_claim.get_locator() {
            ProjectSymbolLocator::ModuleOffset { module_name, .. } => !module_name_set.contains(module_name),
            ProjectSymbolLocator::AbsoluteAddress { .. } => true,
        }
    });

    ProjectSymbolsDeleteMutationSummary {
        deleted_symbol_count: symbol_claim_count_before_delete.saturating_sub(symbol_claims.len()) as u64
            + delete_module_field_summary.get_deleted_module_field_count(),
        deleted_module_count,
        deleted_module_range_count: delete_module_range_summary.get_deleted_module_range_count(),
    }
}

fn build_create_locator(project_symbols_create_request: &ProjectSymbolsCreateRequest) -> Option<ProjectSymbolLocator> {
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

fn resolve_struct_layout_id_size_in_bytes(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    local_struct_layout_descriptors: &[StructLayoutDescriptor],
    struct_layout_id: &str,
) -> Option<u64> {
    ProjectSymbolLayoutMutation::resolve_struct_layout_id_size_in_bytes(
        struct_layout_id,
        |data_type_ref| {
            engine_execution_context
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
