use crate::services::projects::project_item_file_mutation::resolve_project_item_path;
use crate::services::projects::project_item_symbol_resolution::{
    is_promotable_project_item, resolve_project_item_locator, resolve_project_item_struct_layout_id, resolve_project_item_type_id,
};
use crate::services::projects::project_symbol_name_scope::ProjectSymbolNameScope;
use squalr_engine_api::commands::memory::query::memory_query_request::MemoryQueryRequest;
use squalr_engine_api::commands::memory::query::memory_query_response::MemoryQueryResponse;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_response::ProjectItemsPromoteSymbolConflict;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::memory::{pointer::Pointer, pointer_chain_segment::PointerChainSegment};
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::projects::project_symbol_claim::ProjectSymbolClaim;
use squalr_engine_api::structures::projects::project_symbol_locator::ProjectSymbolLocator;
use squalr_engine_api::structures::structs::symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution};
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::time::Duration;

pub const NO_OPENED_PROCESS_STATUS_MESSAGE: &str = "Cannot promote project items to symbols without an opened process.";

#[derive(Clone, Debug, Default)]
pub struct ProjectItemSymbolPromotionChangeSet {
    pub should_save_project: bool,
    pub updated_project_symbol_catalog: Option<ProjectSymbolCatalog>,
    pub promoted_symbol_count: u64,
    pub reused_symbol_count: u64,
    pub promoted_symbol_locator_keys: Vec<String>,
    pub conflicts: Vec<ProjectItemsPromoteSymbolConflict>,
}

pub fn apply_project_item_symbol_promotion(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    opened_project: &mut Project,
    project_directory_path: &Path,
    requested_project_item_paths: &[PathBuf],
    overwrite_conflicting_symbols: bool,
) -> ProjectItemSymbolPromotionChangeSet {
    let mut existing_symbol_claims = opened_project
        .get_project_info()
        .get_project_symbol_catalog()
        .get_symbol_claims()
        .to_vec();
    let mut project_item_replacements = Vec::new();
    let mut change_set = ProjectItemSymbolPromotionChangeSet::default();
    let mut did_mutate_symbol_catalog = false;
    let mut module_size_hints_by_name: BTreeMap<String, u64> = BTreeMap::new();
    let mut module_layout_symbols_to_upsert = Vec::new();

    for requested_project_item_path in requested_project_item_paths {
        let project_item_path = resolve_project_item_path(project_directory_path, requested_project_item_path);
        let project_item_ref = ProjectItemRef::new(project_item_path.clone());
        let Some(project_item) = opened_project
            .get_project_items()
            .get(&project_item_ref)
            .cloned()
        else {
            log::warn!("Skipping promote-symbol request for missing project item: {:?}", project_item_path);
            continue;
        };

        let Some(mut promoted_symbol_candidate) = build_promoted_symbol(
            engine_execution_context,
            opened_project.get_project_info().get_project_symbol_catalog(),
            &project_item_path,
            &project_item,
        ) else {
            log::warn!("Skipping non-promotable project item during promote-symbol request: {:?}", project_item_path);
            continue;
        };
        if let Some((module_name, module_size_hint)) = resolve_promoted_symbol_module_size_hint(
            engine_execution_context,
            opened_project.get_project_info().get_project_symbol_catalog(),
            &promoted_symbol_candidate,
        ) {
            module_size_hints_by_name
                .entry(module_name)
                .and_modify(|existing_size_hint| *existing_size_hint = (*existing_size_hint).max(module_size_hint))
                .or_insert(module_size_hint);
        }

        if let Some(existing_exact_symbol) = find_exact_symbol_claim(&existing_symbol_claims, &promoted_symbol_candidate).cloned() {
            project_item_replacements.push((project_item_ref, build_promoted_project_item(&project_item, &existing_exact_symbol)));
            module_layout_symbols_to_upsert.push(existing_exact_symbol);
            change_set.reused_symbol_count = change_set.reused_symbol_count.saturating_add(1);
            continue;
        }

        let symbol_locator_key = promoted_symbol_candidate.get_symbol_locator_key();
        if let Some(conflicting_symbol_index) = find_symbol_claim_index_by_locator_key(&existing_symbol_claims, &symbol_locator_key) {
            if !overwrite_conflicting_symbols {
                change_set.conflicts.push(ProjectItemsPromoteSymbolConflict {
                    project_item_path: project_item_path.clone(),
                    symbol_locator_key: symbol_locator_key.clone(),
                    existing_display_name: existing_symbol_claims[conflicting_symbol_index]
                        .get_display_name()
                        .to_string(),
                    existing_locator_display: existing_symbol_claims[conflicting_symbol_index]
                        .get_locator()
                        .to_string(),
                    requested_display_name: promoted_symbol_candidate.get_display_name().to_string(),
                });
                continue;
            }

            let deduplicated_display_name = ProjectSymbolNameScope::deduplicate_display_name(
                opened_project.get_project_info().get_project_symbol_catalog(),
                &existing_symbol_claims,
                promoted_symbol_candidate.get_locator(),
                promoted_symbol_candidate.get_display_name(),
                Some(&symbol_locator_key),
            );
            promoted_symbol_candidate.set_display_name(deduplicated_display_name);
            existing_symbol_claims[conflicting_symbol_index] = promoted_symbol_candidate.clone();
        } else {
            let deduplicated_display_name = ProjectSymbolNameScope::deduplicate_display_name(
                opened_project.get_project_info().get_project_symbol_catalog(),
                &existing_symbol_claims,
                promoted_symbol_candidate.get_locator(),
                promoted_symbol_candidate.get_display_name(),
                None,
            );
            promoted_symbol_candidate.set_display_name(deduplicated_display_name);
            existing_symbol_claims.push(promoted_symbol_candidate.clone());
        }

        did_mutate_symbol_catalog = true;
        change_set.promoted_symbol_count = change_set.promoted_symbol_count.saturating_add(1);
        change_set
            .promoted_symbol_locator_keys
            .push(promoted_symbol_candidate.get_symbol_locator_key().to_string());
        module_layout_symbols_to_upsert.push(promoted_symbol_candidate.clone());
        project_item_replacements.push((project_item_ref, build_promoted_project_item(&project_item, &promoted_symbol_candidate)));
    }

    if project_item_replacements.is_empty() && !did_mutate_symbol_catalog {
        return change_set;
    }

    for (project_item_ref, replacement_project_item) in &project_item_replacements {
        if let Some(project_item) = opened_project.get_project_items_mut().get_mut(project_item_ref) {
            *project_item = replacement_project_item.clone();
        }
    }

    let should_update_symbol_catalog = did_mutate_symbol_catalog || !module_layout_symbols_to_upsert.is_empty();
    change_set.updated_project_symbol_catalog = if should_update_symbol_catalog {
        let project_info = opened_project.get_project_info_mut();
        let updated_project_symbol_catalog = {
            let project_symbol_catalog = project_info.get_project_symbol_catalog_mut();
            if did_mutate_symbol_catalog {
                project_symbol_catalog.set_symbol_claims(existing_symbol_claims.clone());
            }
            for (module_name, module_size_hint) in &module_size_hints_by_name {
                project_symbol_catalog.ensure_symbol_module(module_name, *module_size_hint);
                project_symbol_catalog.ensure_module_root_struct_layout(module_name, *module_size_hint);
            }
            for promoted_module_symbol in &module_layout_symbols_to_upsert {
                if let Err(error) = upsert_promoted_module_symbol_layout_field(project_symbol_catalog, promoted_module_symbol) {
                    log::warn!("Failed to mirror promoted symbol into module root layout: {}", error);
                }
            }
            project_symbol_catalog.clone()
        };
        project_info.set_has_unsaved_changes(true);

        Some(updated_project_symbol_catalog)
    } else {
        opened_project
            .get_project_info_mut()
            .set_has_unsaved_changes(true);
        None
    };
    change_set.should_save_project = true;

    change_set
}

pub fn query_has_opened_process(engine_execution_context: &Arc<dyn EngineExecutionContext>) -> bool {
    let memory_query_command = MemoryQueryRequest::default().to_engine_command();
    let (memory_query_response_sender, memory_query_response_receiver) = mpsc::channel();

    let dispatch_result = match engine_execution_context.get_bindings().read() {
        Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
            memory_query_command,
            Box::new(move |engine_response| {
                let conversion_result = MemoryQueryResponse::from_engine_response(engine_response);
                let _ = memory_query_response_sender.send(conversion_result);
            }),
        ),
        Err(error) => {
            log::error!("Failed to acquire engine bindings lock for promote-symbol process query: {}", error);
            return false;
        }
    };

    if let Err(error) = dispatch_result {
        log::error!("Failed to dispatch promote-symbol process query: {}", error);
        return false;
    }

    memory_query_response_receiver
        .recv_timeout(Duration::from_secs(1))
        .ok()
        .and_then(Result::ok)
        .is_some_and(|memory_query_response| memory_query_response.success)
}

fn resolve_promoted_symbol_module_size_hint(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    promoted_symbol: &ProjectSymbolClaim,
) -> Option<(String, u64)> {
    let ProjectSymbolLocator::ModuleOffset { module_name, offset } = promoted_symbol.get_locator() else {
        return None;
    };
    let claim_size_in_bytes = estimate_symbol_claim_size_in_bytes(project_symbol_catalog, promoted_symbol).max(1);
    let minimum_module_size = offset.saturating_add(claim_size_in_bytes);
    let queried_module_size = query_loaded_module_size(engine_execution_context, module_name);

    Some((
        module_name.clone(),
        queried_module_size
            .unwrap_or(minimum_module_size)
            .max(minimum_module_size),
    ))
}

fn upsert_promoted_module_symbol_layout_field(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    promoted_symbol: &ProjectSymbolClaim,
) -> Result<(), String> {
    let ProjectSymbolLocator::ModuleOffset { module_name, offset } = promoted_symbol.get_locator() else {
        return Ok(());
    };
    let field_size_in_bytes = estimate_symbol_claim_size_in_bytes(project_symbol_catalog, promoted_symbol).max(1);
    let mut visited_layout_ids = HashSet::new();

    upsert_symbol_layout_field_at_offset(
        project_symbol_catalog,
        module_name,
        *offset,
        promoted_symbol.get_display_name(),
        promoted_symbol.get_struct_layout_id(),
        field_size_in_bytes,
        &mut visited_layout_ids,
    )
}

fn upsert_symbol_layout_field_at_offset(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    struct_layout_id: &str,
    offset_in_bytes: u64,
    field_name: &str,
    field_type_id: &str,
    field_size_in_bytes: u64,
    visited_layout_ids: &mut HashSet<String>,
) -> Result<(), String> {
    if !visited_layout_ids.insert(struct_layout_id.to_string()) {
        return Err(format!("Cannot upsert field through recursive layout `{}`.", struct_layout_id));
    }

    let Some(struct_layout_descriptor) = find_struct_layout_descriptor(project_symbol_catalog, struct_layout_id) else {
        visited_layout_ids.remove(struct_layout_id);
        return Err(format!("Cannot find target layout `{}`.", struct_layout_id));
    };
    let struct_layout_definition = struct_layout_descriptor.get_struct_layout_definition().clone();

    if let Some(containing_child_struct_layout) = find_containing_child_struct_layout(
        project_symbol_catalog,
        &struct_layout_definition,
        offset_in_bytes,
        field_size_in_bytes,
        visited_layout_ids,
    ) {
        let result = upsert_symbol_layout_field_at_offset(
            project_symbol_catalog,
            &containing_child_struct_layout.struct_layout_id,
            containing_child_struct_layout.offset_in_child,
            field_name,
            field_type_id,
            field_size_in_bytes,
            visited_layout_ids,
        );
        visited_layout_ids.remove(struct_layout_id);

        return result;
    }

    let updated_struct_layout_definition = upsert_field_in_symbolic_struct_definition(
        project_symbol_catalog,
        &struct_layout_definition,
        offset_in_bytes,
        field_name,
        field_type_id,
        field_size_in_bytes,
    )?;
    replace_struct_layout_definition(project_symbol_catalog, struct_layout_id, updated_struct_layout_definition)?;
    visited_layout_ids.remove(struct_layout_id);

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContainingChildStructLayout {
    struct_layout_id: String,
    offset_in_child: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SymbolicFieldSpan {
    field_index: usize,
    offset_in_bytes: u64,
    size_in_bytes: u64,
}

fn find_containing_child_struct_layout(
    project_symbol_catalog: &ProjectSymbolCatalog,
    struct_layout_definition: &SymbolicStructDefinition,
    offset_in_bytes: u64,
    field_size_in_bytes: u64,
    visited_layout_ids: &HashSet<String>,
) -> Option<ContainingChildStructLayout> {
    let requested_end_offset = offset_in_bytes.checked_add(field_size_in_bytes)?;
    let mut containing_child_layouts = collect_symbolic_field_spans(project_symbol_catalog, struct_layout_definition)
        .into_iter()
        .filter_map(|field_span| {
            let field_end_offset = field_span
                .offset_in_bytes
                .checked_add(field_span.size_in_bytes)?;

            if offset_in_bytes < field_span.offset_in_bytes || requested_end_offset > field_end_offset {
                return None;
            }

            let field_definition = &struct_layout_definition.get_fields()[field_span.field_index];
            let child_struct_layout_id = field_definition.get_data_type_ref().get_data_type_id();
            if visited_layout_ids.contains(child_struct_layout_id) || find_struct_layout_descriptor(project_symbol_catalog, child_struct_layout_id).is_none() {
                return None;
            }

            let containing_child_struct_layout = match field_definition.get_container_type() {
                ContainerType::None => Some(ContainingChildStructLayout {
                    struct_layout_id: child_struct_layout_id.to_string(),
                    offset_in_child: offset_in_bytes.saturating_sub(field_span.offset_in_bytes),
                }),
                ContainerType::Array | ContainerType::ArrayFixed(_) => {
                    let child_struct_size_in_bytes = estimate_symbol_type_size_in_bytes(project_symbol_catalog, child_struct_layout_id, &mut HashSet::new());
                    if child_struct_size_in_bytes == 0 {
                        return None;
                    }

                    Some(ContainingChildStructLayout {
                        struct_layout_id: child_struct_layout_id.to_string(),
                        offset_in_child: offset_in_bytes.saturating_sub(field_span.offset_in_bytes) % child_struct_size_in_bytes,
                    })
                }
                ContainerType::Pointer(_) | ContainerType::PointerArray(_) | ContainerType::PointerArrayFixed(_, _) => None,
            }?;

            Some((containing_child_struct_layout, field_span.size_in_bytes))
        })
        .collect::<Vec<_>>();

    containing_child_layouts.sort_by(|left_layout, right_layout| {
        left_layout.1.cmp(&right_layout.1).then_with(|| {
            left_layout
                .0
                .struct_layout_id
                .cmp(&right_layout.0.struct_layout_id)
        })
    });
    containing_child_layouts
        .into_iter()
        .next()
        .map(|(containing_child_struct_layout, _field_size_in_bytes)| containing_child_struct_layout)
}

fn upsert_field_in_symbolic_struct_definition(
    project_symbol_catalog: &ProjectSymbolCatalog,
    struct_layout_definition: &SymbolicStructDefinition,
    offset_in_bytes: u64,
    field_name: &str,
    field_type_id: &str,
    field_size_in_bytes: u64,
) -> Result<SymbolicStructDefinition, String> {
    let field_end_offset = offset_in_bytes
        .checked_add(field_size_in_bytes)
        .ok_or_else(|| String::from("Promoted field range is too large."))?;
    let mut fields = struct_layout_definition.get_fields().to_vec();
    let field_spans = collect_symbolic_field_spans(project_symbol_catalog, struct_layout_definition);

    for field_span in &field_spans {
        let existing_field_end_offset = field_span
            .offset_in_bytes
            .checked_add(field_span.size_in_bytes)
            .ok_or_else(|| String::from("Existing field range is too large."))?;
        let ranges_overlap = field_span.offset_in_bytes < field_end_offset && offset_in_bytes < existing_field_end_offset;

        if ranges_overlap && field_span.offset_in_bytes != offset_in_bytes {
            let existing_field = &struct_layout_definition.get_fields()[field_span.field_index];

            return Err(format!(
                "Promoted field `{}` overlaps existing field `{}` in `{}`.",
                field_name,
                existing_field.get_field_name(),
                struct_layout_definition.get_symbol_namespace()
            ));
        }
    }

    let promoted_field = build_promoted_symbolic_field(field_name, field_type_id, offset_in_bytes);
    if let Some(existing_field_span) = field_spans
        .iter()
        .find(|field_span| field_span.offset_in_bytes == offset_in_bytes)
    {
        fields[existing_field_span.field_index] = promoted_field;
    } else {
        fields.push(promoted_field);
    }

    fields.sort_by(|left_field, right_field| {
        resolve_static_offset(left_field)
            .unwrap_or(u64::MAX)
            .cmp(&resolve_static_offset(right_field).unwrap_or(u64::MAX))
            .then_with(|| left_field.get_field_name().cmp(right_field.get_field_name()))
    });

    Ok(SymbolicStructDefinition::new_with_layout_kind(
        struct_layout_definition.get_symbol_namespace().to_string(),
        struct_layout_definition.get_layout_kind(),
        fields,
    )
    .with_declared_size_in_bytes(struct_layout_definition.get_declared_size_in_bytes()))
}

fn build_promoted_symbolic_field(
    field_name: &str,
    field_type_id: &str,
    offset_in_bytes: u64,
) -> SymbolicFieldDefinition {
    let symbolic_field_definition =
        SymbolicFieldDefinition::from_str(field_type_id).unwrap_or_else(|_| SymbolicFieldDefinition::new(DataTypeRef::new(field_type_id), ContainerType::None));

    SymbolicFieldDefinition::new_named_with_resolutions_and_display_count(
        field_name.to_string(),
        symbolic_field_definition.get_data_type_ref().clone(),
        symbolic_field_definition.get_container_type(),
        symbolic_field_definition.get_count_resolution().clone(),
        symbolic_field_definition.get_display_count_resolution().clone(),
        SymbolicFieldOffsetResolution::new_static(offset_in_bytes),
    )
    .with_active_when_resolver(symbolic_field_definition.get_active_when_resolver().cloned())
}

fn collect_symbolic_field_spans(
    project_symbol_catalog: &ProjectSymbolCatalog,
    struct_layout_definition: &SymbolicStructDefinition,
) -> Vec<SymbolicFieldSpan> {
    let mut field_spans = Vec::new();
    let mut next_sequential_offset = 0_u64;

    for (field_index, field_definition) in struct_layout_definition.get_fields().iter().enumerate() {
        if field_definition.is_unassigned() {
            next_sequential_offset = next_sequential_offset.saturating_add(field_definition.get_unassigned_size_in_bytes().unwrap_or(0));
            continue;
        }

        let field_offset = match field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) if struct_layout_definition.get_layout_kind().is_union() => {
                0
            }
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
        };
        let field_size_in_bytes = estimate_symbolic_field_size_in_bytes(project_symbol_catalog, field_definition, &mut HashSet::new());

        next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
        field_spans.push(SymbolicFieldSpan {
            field_index,
            offset_in_bytes: field_offset,
            size_in_bytes: field_size_in_bytes,
        });
    }

    field_spans
}

fn resolve_static_offset(symbolic_field_definition: &SymbolicFieldDefinition) -> Option<u64> {
    match symbolic_field_definition.get_offset_resolution() {
        SymbolicFieldOffsetResolution::Static(offset_in_bytes) => Some(*offset_in_bytes),
        SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => None,
    }
}

fn replace_struct_layout_definition(
    project_symbol_catalog: &mut ProjectSymbolCatalog,
    struct_layout_id: &str,
    struct_layout_definition: SymbolicStructDefinition,
) -> Result<(), String> {
    let mut struct_layout_descriptors = project_symbol_catalog.get_struct_layout_descriptors().to_vec();
    let Some(struct_layout_descriptor) = struct_layout_descriptors
        .iter_mut()
        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == struct_layout_id)
    else {
        return Err(format!("Cannot find target layout `{}`.", struct_layout_id));
    };

    *struct_layout_descriptor = StructLayoutDescriptor::new(struct_layout_id.to_string(), struct_layout_definition);
    project_symbol_catalog.set_struct_layout_descriptors(struct_layout_descriptors);

    Ok(())
}

fn find_struct_layout_descriptor<'a>(
    project_symbol_catalog: &'a ProjectSymbolCatalog,
    struct_layout_id: &str,
) -> Option<&'a StructLayoutDescriptor> {
    project_symbol_catalog
        .get_struct_layout_descriptors()
        .iter()
        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == struct_layout_id)
}

fn query_loaded_module_size(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    module_name: &str,
) -> Option<u64> {
    let memory_query_command = MemoryQueryRequest::default().to_engine_command();
    let (memory_query_response_sender, memory_query_response_receiver) = mpsc::channel();

    let dispatch_result = match engine_execution_context.get_bindings().read() {
        Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
            memory_query_command,
            Box::new(move |engine_response| {
                let conversion_result = MemoryQueryResponse::from_engine_response(engine_response);
                let _ = memory_query_response_sender.send(conversion_result);
            }),
        ),
        Err(error) => {
            log::error!("Failed to acquire engine bindings lock for promote-symbol module query: {}", error);
            return None;
        }
    };

    if dispatch_result.is_err() {
        return None;
    }

    let memory_query_response = memory_query_response_receiver
        .recv_timeout(Duration::from_secs(1))
        .ok()
        .and_then(Result::ok)?;

    if !memory_query_response.success {
        return None;
    }

    let loaded_module = memory_query_response
        .modules
        .iter()
        .find(|normalized_module| normalized_module.get_module_name() == module_name)?;
    let module_base_address = loaded_module.get_base_address();

    let loaded_module_size = loaded_module.get_region_size();
    let containing_virtual_page_size = memory_query_response
        .virtual_pages
        .iter()
        .find(|virtual_page| virtual_page.contains_address(module_base_address))
        .map(|virtual_page| {
            virtual_page
                .get_end_address()
                .saturating_sub(module_base_address)
        });

    Some(
        containing_virtual_page_size
            .unwrap_or(0)
            .max(loaded_module_size),
    )
}

fn estimate_symbol_claim_size_in_bytes(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_claim: &ProjectSymbolClaim,
) -> u64 {
    estimate_symbol_type_size_in_bytes(project_symbol_catalog, symbol_claim.get_struct_layout_id(), &mut HashSet::new())
}

fn estimate_symbol_type_size_in_bytes(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_type_id: &str,
    visited_type_ids: &mut HashSet<String>,
) -> u64 {
    if let Some(primitive_size_in_bytes) = estimate_primitive_data_type_size_in_bytes(symbol_type_id) {
        return primitive_size_in_bytes;
    }

    if let Some(struct_layout_descriptor) = project_symbol_catalog
        .get_struct_layout_descriptors()
        .iter()
        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == symbol_type_id)
    {
        return estimate_symbolic_struct_size_in_bytes(
            project_symbol_catalog,
            struct_layout_descriptor.get_struct_layout_definition(),
            visited_type_ids,
        );
    }

    if let Ok(symbolic_field_definition) = SymbolicFieldDefinition::from_str(symbol_type_id) {
        return estimate_symbolic_field_size_in_bytes(project_symbol_catalog, &symbolic_field_definition, visited_type_ids);
    }

    if let Ok(symbolic_struct_definition) = SymbolicStructDefinition::from_str(symbol_type_id) {
        return estimate_symbolic_struct_size_in_bytes(project_symbol_catalog, &symbolic_struct_definition, visited_type_ids);
    }

    1
}

fn estimate_symbolic_struct_size_in_bytes(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbolic_struct_definition: &SymbolicStructDefinition,
    visited_type_ids: &mut HashSet<String>,
) -> u64 {
    let mut next_sequential_offset = 0_u64;

    for symbolic_field_definition in symbolic_struct_definition.get_fields() {
        let field_offset = match symbolic_field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                if symbolic_struct_definition.get_layout_kind().is_union() =>
            {
                0
            }
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
        };
        let field_size_in_bytes = estimate_symbolic_field_size_in_bytes(project_symbol_catalog, symbolic_field_definition, visited_type_ids);

        next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
    }

    next_sequential_offset.max(
        symbolic_struct_definition
            .get_declared_size_in_bytes()
            .unwrap_or(0),
    )
}

fn estimate_symbolic_field_size_in_bytes(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbolic_field_definition: &SymbolicFieldDefinition,
    visited_type_ids: &mut HashSet<String>,
) -> u64 {
    let data_type_id = symbolic_field_definition.get_data_type_ref().get_data_type_id();
    let unit_size_in_bytes = match symbolic_field_definition.get_container_type() {
        ContainerType::Pointer(pointer_size) => pointer_size.get_size_in_bytes(),
        _ => {
            if !visited_type_ids.insert(data_type_id.to_string()) {
                return 0;
            }

            let size_in_bytes = estimate_symbol_type_size_in_bytes(project_symbol_catalog, data_type_id, visited_type_ids);

            visited_type_ids.remove(data_type_id);
            size_in_bytes
        }
    };

    symbolic_field_definition
        .get_container_type()
        .get_total_size_in_bytes(unit_size_in_bytes)
}

fn estimate_primitive_data_type_size_in_bytes(data_type_id: &str) -> Option<u64> {
    match data_type_id {
        "bool" | "i8" | "u8" => Some(1),
        "i16" | "u16" | "i16be" | "u16be" => Some(2),
        "i24" | "u24" | "i24be" | "u24be" => Some(3),
        "f32" | "i32" | "u32" | "f32be" | "i32be" | "u32be" => Some(4),
        "f64" | "i64" | "u64" | "f64be" | "i64be" | "u64be" => Some(8),
        "i128" | "u128" | "i128be" | "u128be" => Some(16),
        _ => None,
    }
}

fn build_promoted_symbol(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: &squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog,
    project_item_path: &Path,
    project_item: &ProjectItem,
) -> Option<ProjectSymbolClaim> {
    if !is_promotable_project_item(project_item) {
        return None;
    }

    let display_name = build_display_name(project_item, project_item_path);
    let struct_layout_id = resolve_project_item_struct_layout_id(project_symbol_catalog, project_item)?;
    let locator = resolve_project_item_locator(engine_execution_context, project_symbol_catalog, project_item)?;
    let mut promoted_symbol = ProjectSymbolClaim::new(display_name, locator, struct_layout_id);

    promoted_symbol
        .get_metadata_mut()
        .insert(String::from("source.project_item_path"), project_item_path.to_string_lossy().into_owned());

    if let Some(project_item_type_id) = resolve_project_item_type_id(project_item) {
        promoted_symbol
            .get_metadata_mut()
            .insert(String::from("source.project_item_type"), project_item_type_id.to_string());
    }

    if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
        let pointer = ProjectItemTypePointer::get_field_pointer(project_item);
        append_pointer_metadata(promoted_symbol.get_metadata_mut(), &pointer, project_item);
    }

    Some(promoted_symbol)
}

fn build_promoted_project_item(
    source_project_item: &ProjectItem,
    promoted_symbol: &ProjectSymbolClaim,
) -> ProjectItem {
    let source_project_item_type_id = source_project_item.get_item_type().get_project_item_type_id();
    let mut promoted_project_item = source_project_item.clone();

    if source_project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
        ProjectItemTypeAddress::set_field_symbolic_struct_definition_reference(&mut promoted_project_item, promoted_symbol.get_struct_layout_id());
        apply_promoted_symbol_chain_segment(&mut promoted_project_item, promoted_symbol);
    } else if source_project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
        ProjectItemTypePointer::set_field_symbolic_struct_definition_reference(&mut promoted_project_item, promoted_symbol.get_struct_layout_id());
    }

    if source_project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
        let mut address_project_item = source_project_item.clone();
        let freeze_display_value = ProjectItemTypeAddress::get_field_freeze_data_value_interpreter(&mut address_project_item);

        ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(&mut promoted_project_item, &freeze_display_value);
    } else if source_project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
        ProjectItemTypePointer::set_field_freeze_data_value_interpreter(
            &mut promoted_project_item,
            &ProjectItemTypePointer::get_field_freeze_data_value_interpreter(source_project_item),
        );
    }

    promoted_project_item
}

fn apply_promoted_symbol_chain_segment(
    promoted_project_item: &mut ProjectItem,
    promoted_symbol: &ProjectSymbolClaim,
) {
    let ProjectSymbolLocator::ModuleOffset { module_name, offset } = promoted_symbol.get_locator() else {
        return;
    };

    if !PointerChainSegment::is_valid_symbol_name(promoted_symbol.get_display_name()) {
        return;
    }

    let mut address_target = ProjectItemTypeAddress::get_address_target(promoted_project_item);

    if address_target.get_module_name() != module_name {
        return;
    }

    let mut pointer_offsets = address_target.get_pointer_offsets().to_vec();
    let Some(first_pointer_offset) = pointer_offsets.first_mut() else {
        return;
    };

    if first_pointer_offset.as_offset() != Some(*offset as i64) {
        return;
    }

    *first_pointer_offset = PointerChainSegment::Symbol(promoted_symbol.get_display_name().to_string());
    address_target.set_pointer_offsets(pointer_offsets);
    ProjectItemTypeAddress::set_address_target(promoted_project_item, address_target);
}

fn build_display_name(
    project_item: &ProjectItem,
    project_item_path: &Path,
) -> String {
    let project_item_name = project_item.get_field_name();
    let trimmed_project_item_name = project_item_name.trim();

    if !trimmed_project_item_name.is_empty() {
        return trimmed_project_item_name.to_string();
    }

    project_item_path
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .map(str::trim)
        .filter(|file_stem| !file_stem.is_empty())
        .unwrap_or("Symbol")
        .to_string()
}

fn append_pointer_metadata(
    metadata: &mut std::collections::BTreeMap<String, String>,
    pointer: &Pointer,
    project_item: &ProjectItem,
) {
    metadata.insert(String::from("source.pointer_root"), pointer.get_root_display_text());
    metadata.insert(String::from("source.pointer_root_module"), pointer.get_module_name().to_string());
    metadata.insert(String::from("source.pointer_root_offset"), format!("0x{:X}", pointer.get_address()));
    metadata.insert(
        String::from("source.pointer_offsets"),
        serde_json::to_string(pointer.get_offset_segments()).unwrap_or_else(|_| String::from("[]")),
    );
    metadata.insert(String::from("source.pointer_size"), pointer.get_pointer_size().to_string());

    let evaluated_pointer_path = ProjectItemTypePointer::get_field_evaluated_pointer_path(project_item);

    if !evaluated_pointer_path.trim().is_empty() {
        metadata.insert(String::from("source.evaluated_pointer_path"), evaluated_pointer_path);
    }
}

fn find_exact_symbol_claim<'a>(
    existing_symbol_claims: &'a [ProjectSymbolClaim],
    promoted_symbol: &ProjectSymbolClaim,
) -> Option<&'a ProjectSymbolClaim> {
    existing_symbol_claims.iter().find(|existing_symbol_claim| {
        existing_symbol_claim.get_display_name() == promoted_symbol.get_display_name()
            && existing_symbol_claim.get_struct_layout_id() == promoted_symbol.get_struct_layout_id()
            && existing_symbol_claim.get_locator() == promoted_symbol.get_locator()
    })
}

fn find_symbol_claim_index_by_locator_key(
    existing_symbol_claims: &[ProjectSymbolClaim],
    symbol_locator_key: &str,
) -> Option<usize> {
    existing_symbol_claims
        .iter()
        .position(|existing_symbol_claim| existing_symbol_claim.get_symbol_locator_key() == symbol_locator_key)
}

#[cfg(test)]
mod tests {
    use super::NO_OPENED_PROCESS_STATUS_MESSAGE;
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::commands::{
        memory::{
            memory_command::MemoryCommand, query::memory_query_response::MemoryQueryResponse, read::memory_read_request::MemoryReadRequest,
            read::memory_read_response::MemoryReadResponse,
        },
        privileged_command::PrivilegedCommand,
        privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse},
        project_items::promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest,
        project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest,
        registry::{registry_command::RegistryCommand, set_project_symbols::registry_set_project_symbols_response::RegistrySetProjectSymbolsResponse},
        unprivileged_command::UnprivilegedCommand,
        unprivileged_command_response::UnprivilegedCommandResponse,
    };
    use squalr_engine_api::engine::{
        engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings, engine_binding_error::EngineBindingError, engine_event_envelope::EngineEventEnvelope,
        engine_execution_context::EngineExecutionContext,
    };
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::{
            built_in_types::{u8::data_type_u8::DataTypeU8, u64::data_type_u64::DataTypeU64},
            data_type_ref::DataTypeRef,
        },
        data_values::container_type::ContainerType,
        memory::{normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
        memory::{pointer::Pointer, pointer_chain_segment::PointerChainSegment},
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::{
            project::Project, project_info::ProjectInfo, project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress,
            project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory,
            project_items::built_in_types::project_item_type_pointer::ProjectItemTypePointer, project_items::project_item::ProjectItem,
            project_items::project_item_ref::ProjectItemRef, project_manifest::ProjectManifest, project_symbol_catalog::ProjectSymbolCatalog,
            project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
        },
        structs::{
            symbolic_field_definition::{SymbolicFieldCountResolution, SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
            symbolic_struct_definition::SymbolicStructDefinition,
            valued_struct::ValuedStruct,
        },
    };
    use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
    use squalr_engine_session::engine_unprivileged_state::{EngineUnprivilegedState, EngineUnprivilegedStateOptions};
    use std::{
        collections::HashMap,
        fs::File,
        path::{Path, PathBuf},
        sync::{Arc, Mutex, RwLock},
    };

    struct MockPromoteBindings {
        captured_project_symbol_catalogs: Arc<Mutex<Vec<ProjectSymbolCatalog>>>,
        memory_read_response_factory: Arc<dyn Fn(&MemoryReadRequest) -> MemoryReadResponse + Send + Sync>,
        memory_query_success: bool,
        memory_query_modules: Vec<NormalizedModule>,
        memory_query_virtual_pages: Vec<NormalizedRegion>,
    }

    impl MockPromoteBindings {
        fn new(memory_read_response_factory: impl Fn(&MemoryReadRequest) -> MemoryReadResponse + Send + Sync + 'static) -> Self {
            Self {
                captured_project_symbol_catalogs: Arc::new(Mutex::new(Vec::new())),
                memory_read_response_factory: Arc::new(memory_read_response_factory),
                memory_query_success: true,
                memory_query_modules: Vec::new(),
                memory_query_virtual_pages: Vec::new(),
            }
        }

        fn without_opened_process(mut self) -> Self {
            self.memory_query_success = false;

            self
        }

        fn with_memory_query_modules(
            mut self,
            memory_query_modules: Vec<NormalizedModule>,
        ) -> Self {
            self.memory_query_modules = memory_query_modules;

            self
        }

        fn with_memory_query_virtual_pages(
            mut self,
            memory_query_virtual_pages: Vec<NormalizedRegion>,
        ) -> Self {
            self.memory_query_virtual_pages = memory_query_virtual_pages;

            self
        }

        fn captured_project_symbol_catalogs(&self) -> Arc<Mutex<Vec<ProjectSymbolCatalog>>> {
            self.captured_project_symbol_catalogs.clone()
        }
    }

    impl EngineApiUnprivilegedBindings for MockPromoteBindings {
        fn dispatch_privileged_command(
            &self,
            engine_command: PrivilegedCommand,
            callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            match engine_command {
                PrivilegedCommand::Registry(RegistryCommand::SetProjectSymbols {
                    registry_set_project_symbols_request,
                }) => {
                    let mut captured_project_symbol_catalogs = self
                        .captured_project_symbol_catalogs
                        .lock()
                        .map_err(|error| EngineBindingError::lock_failure("capturing project symbol catalog in tests", error.to_string()))?;

                    captured_project_symbol_catalogs.push(
                        registry_set_project_symbols_request
                            .project_symbol_catalog
                            .clone(),
                    );
                    drop(captured_project_symbol_catalogs);

                    callback(RegistrySetProjectSymbolsResponse { success: true }.to_engine_response());

                    Ok(())
                }
                PrivilegedCommand::Memory(MemoryCommand::Read { memory_read_request }) => {
                    callback((self.memory_read_response_factory)(&memory_read_request).to_engine_response());

                    Ok(())
                }
                PrivilegedCommand::Memory(MemoryCommand::Query { .. }) => {
                    callback(
                        MemoryQueryResponse {
                            virtual_pages: self.memory_query_virtual_pages.clone(),
                            modules: self.memory_query_modules.clone(),
                            success: self.memory_query_success,
                        }
                        .to_engine_response(),
                    );

                    Ok(())
                }
                _ => Err(EngineBindingError::unavailable(
                    "dispatching unsupported privileged command in promote-symbol tests",
                )),
            }
        }

        fn dispatch_unprivileged_command(
            &self,
            _engine_command: UnprivilegedCommand,
            _engine_execution_context: &Arc<dyn EngineExecutionContext>,
            _callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable("dispatching unprivileged commands in promote-symbol tests"))
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();

            Ok(event_receiver)
        }
    }

    fn create_engine_unprivileged_state(mock_promote_bindings: MockPromoteBindings) -> Arc<EngineUnprivilegedState> {
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(mock_promote_bindings));

        EngineUnprivilegedState::new_with_options(engine_bindings, EngineUnprivilegedStateOptions { enable_console_logging: false })
    }

    fn create_project_with_item(
        project_directory_path: &Path,
        project_item_file_name: &str,
        project_item: ProjectItem,
    ) -> (Project, PathBuf) {
        let project_file_path = project_directory_path.join(Project::PROJECT_FILE);
        let root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
        let project_root_ref = ProjectItemRef::new(root_directory_path.clone());
        let project_item_path = root_directory_path.join(project_item_file_name);
        let project_item_ref = ProjectItemRef::new(project_item_path.clone());
        let project_info = ProjectInfo::new(project_file_path, None, ProjectManifest::default());
        let mut project_items = HashMap::new();

        project_items.insert(project_root_ref.clone(), ProjectItemTypeDirectory::new_project_item(&project_root_ref));
        project_items.insert(project_item_ref, project_item);
        std::fs::create_dir_all(&root_directory_path).expect("Expected test project root directory to be created.");
        File::create(&project_item_path).expect("Expected test project item placeholder file to be created.");

        let mut project = Project::new(project_info, project_items, project_root_ref);
        project
            .save_to_path(project_directory_path, true)
            .expect("Expected test project to save.");

        (project, project_item_path)
    }

    fn create_pointer_memory_read_response(pointer_value: u64) -> MemoryReadResponse {
        MemoryReadResponse {
            valued_struct: ValuedStruct::new_anonymous(vec![
                DataTypeU64::get_value_from_primitive(pointer_value).to_named_valued_struct_field(String::from("value"), true),
            ]),
            address: pointer_value,
            success: true,
        }
    }

    #[test]
    fn promote_symbol_request_fails_with_warning_without_opened_process() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default()).without_opened_process();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path],
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(!promote_symbol_response.success);
        assert_eq!(promote_symbol_response.status_message, NO_OPENED_PROCESS_STATUS_MESSAGE);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 0);
        assert_eq!(promote_symbol_response.reused_symbol_count, 0);
        assert!(promote_symbol_response.promoted_symbol_locator_keys.is_empty());
        assert!(promote_symbol_response.conflicts.is_empty());

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected project to load from disk.");
        assert!(
            loaded_project
                .get_project_info()
                .get_project_symbol_catalog()
                .get_symbol_claims()
                .is_empty()
        );
    }

    #[test]
    fn promote_symbol_request_persists_address_item_as_symbol_claim() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default())
            .with_memory_query_modules(vec![NormalizedModule::new("game.exe", 0x10000000, 0x2000)])
            .with_memory_query_virtual_pages(vec![NormalizedRegion::new(0x10000000, 0x5000)]);
        let captured_project_symbol_catalogs = mock_promote_bindings.captured_project_symbol_catalogs();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path.clone()],
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 1);
        assert_eq!(promote_symbol_response.reused_symbol_count, 0);
        assert_eq!(promote_symbol_response.promoted_symbol_locator_keys, vec![String::from("module:game.exe:1234")]);
        assert!(promote_symbol_response.conflicts.is_empty());

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(symbol_claims[0].get_symbol_locator_key(), "module:game.exe:1234");
        assert_eq!(symbol_claims[0].get_display_name(), "Health");
        assert_eq!(
            symbol_claims[0].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x1234)
        );
        assert_eq!(symbol_claims[0].get_struct_layout_id(), "u8");
        assert_eq!(symbol_claims[0].get_metadata().get("source.project_item_type"), Some(&String::from("address")));
        assert_eq!(
            symbol_claims[0].get_metadata().get("source.project_item_path"),
            Some(&project_item_path.to_string_lossy().into_owned())
        );
        let symbol_modules = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules();
        assert_eq!(symbol_modules.len(), 1);
        assert_eq!(symbol_modules[0].get_module_name(), "game.exe");
        assert_eq!(symbol_modules[0].get_size(), 0x5000);
        assert!(
            symbol_modules[0].get_fields().is_empty(),
            "Promoting into a new module should leave unowned bytes as synthesized UNASSIGNED gaps, not persisted u8[] fields."
        );
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
            Some(0x5000)
        );
        let module_root_fields = struct_layout_descriptors[0]
            .get_struct_layout_definition()
            .get_fields();
        assert_eq!(module_root_fields.len(), 1);
        assert_eq!(module_root_fields[0].get_field_name(), "Health");
        assert_eq!(module_root_fields[0].get_data_type_ref().get_data_type_id(), "u8");
        assert_eq!(module_root_fields[0].get_container_type(), ContainerType::None);
        assert_eq!(
            module_root_fields[0].get_offset_resolution(),
            &SymbolicFieldOffsetResolution::new_static(0x1234)
        );
        let promoted_project_item = loaded_project
            .get_project_items()
            .get(&ProjectItemRef::new(project_item_path.clone()))
            .expect("Expected promoted project item to remain in the project.");

        assert_eq!(
            promoted_project_item.get_item_type().get_project_item_type_id(),
            ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID
        );
        let mut promoted_project_item = promoted_project_item.clone();
        assert_eq!(
            ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut promoted_project_item)
                .map(|symbolic_struct_ref| symbolic_struct_ref.get_symbolic_struct_namespace().to_string()),
            Some(String::from("u8"))
        );
        assert_eq!(
            ProjectItemTypeAddress::get_address_target(&mut promoted_project_item).get_pointer_offsets(),
            &[PointerChainSegment::Symbol(String::from("Health"))]
        );

        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
        assert_eq!(
            captured_project_symbol_catalogs[0]
                .get_struct_layout_descriptors()
                .first()
                .map(StructLayoutDescriptor::get_struct_layout_id),
            Some("game.exe")
        );
    }

    #[test]
    fn promote_symbol_request_inserts_layout_field_into_containing_struct() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x120, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (mut project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .set_struct_layout_descriptors(vec![
                StructLayoutDescriptor::new(
                    String::from("game.exe"),
                    SymbolicStructDefinition::new(
                        String::from("game.exe"),
                        vec![SymbolicFieldDefinition::new_named_with_resolutions(
                            String::from("Headers"),
                            DataTypeRef::new("pe.headers"),
                            ContainerType::None,
                            SymbolicFieldCountResolution::Inferred,
                            SymbolicFieldOffsetResolution::new_static(0x100),
                        )],
                    )
                    .with_declared_size_in_bytes(Some(0x5000)),
                ),
                StructLayoutDescriptor::new(
                    String::from("pe.headers"),
                    SymbolicStructDefinition::new(String::from("pe.headers"), Vec::new()).with_declared_size_in_bytes(Some(0x100)),
                ),
            ]);
        project
            .save_to_path(temp_directory.path(), true)
            .expect("Expected test project to save after seeding symbol layouts.");
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default())
            .with_memory_query_modules(vec![NormalizedModule::new("game.exe", 0x10000000, 0x5000)])
            .with_memory_query_virtual_pages(vec![NormalizedRegion::new(0x10000000, 0x5000)]);
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path],
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let struct_layout_descriptors = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_struct_layout_descriptors();
        let module_layout = struct_layout_descriptors
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "game.exe")
            .expect("Expected module root layout.");
        let header_layout = struct_layout_descriptors
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "pe.headers")
            .expect("Expected nested header layout.");

        assert_eq!(module_layout.get_struct_layout_definition().get_fields().len(), 1);
        let header_fields = header_layout.get_struct_layout_definition().get_fields();
        assert_eq!(header_fields.len(), 1);
        assert_eq!(header_fields[0].get_field_name(), "Health");
        assert_eq!(header_fields[0].get_offset_resolution(), &SymbolicFieldOffsetResolution::new_static(0x20));
    }

    #[test]
    fn promote_symbol_request_uses_loaded_module_size_when_base_page_is_smaller() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default())
            .with_memory_query_modules(vec![NormalizedModule::new("game.exe", 0x10000000, 0x5000)])
            .with_memory_query_virtual_pages(vec![NormalizedRegion::new(0x10000000, 0x1000)]);
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path.clone()],
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let symbol_modules = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_modules();

        assert_eq!(symbol_modules.len(), 1);
        assert_eq!(symbol_modules[0].get_module_name(), "game.exe");
        assert_eq!(symbol_modules[0].get_size(), 0x5000);
        assert!(symbol_modules[0].get_fields().is_empty());
    }

    #[test]
    fn promote_symbol_request_deduplicates_display_name_within_module_scope() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Timer", 0x1240, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (mut project, project_item_path) = create_project_with_item(temp_directory.path(), "timer.json", address_project_item);
        project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .set_symbol_claims(vec![ProjectSymbolClaim::new_module_offset(
                String::from("Timer"),
                String::from("game.exe"),
                0x1234,
                String::from("u8"),
            )]);
        project
            .save_to_path(temp_directory.path(), true)
            .expect("Expected test project to save after seeding same-scope symbol.");
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default())
            .with_memory_query_modules(vec![NormalizedModule::new("game.exe", 0x10000000, 0x2000)])
            .with_memory_query_virtual_pages(vec![NormalizedRegion::new(0x10000000, 0x5000)]);
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path.clone()],
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 1);
        assert!(promote_symbol_response.conflicts.is_empty());

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 2);
        assert_eq!(symbol_claims[0].get_display_name(), "Timer");
        assert_eq!(symbol_claims[1].get_display_name(), "Timer_0");

        let promoted_project_item = loaded_project
            .get_project_items()
            .get(&ProjectItemRef::new(project_item_path))
            .expect("Expected promoted project item to remain in the project.");
        let mut promoted_project_item = promoted_project_item.clone();
        assert_eq!(
            ProjectItemTypeAddress::get_address_target(&mut promoted_project_item).get_pointer_offsets(),
            &[PointerChainSegment::Symbol(String::from("Timer_0"))]
        );
    }

    #[test]
    fn promote_symbol_then_delete_removes_symbol_without_converting_project_item() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "", "", DataTypeU8::get_value_from_primitive(0));
        let (project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default());
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path.clone()],
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 1);
        assert_eq!(promote_symbol_response.promoted_symbol_locator_keys, vec![String::from("absolute:1234")]);

        let project_symbols_delete_response = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: vec![String::from("absolute:1234")],
            module_names: Vec::new(),
            module_ranges: Vec::new(),
        }
        .execute(&engine_execution_context);

        assert!(project_symbols_delete_response.success);
        assert_eq!(project_symbols_delete_response.deleted_symbol_count, 1);

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected converted project to load from disk.");
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
        assert_eq!(ProjectItemTypeAddress::get_field_module(&mut converted_project_item), "");
        assert_eq!(
            ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut converted_project_item)
                .map(|symbolic_struct_ref| symbolic_struct_ref.get_symbolic_struct_namespace().to_string()),
            Some(String::from("u8"))
        );
    }

    #[test]
    fn promote_symbol_request_resolves_pointer_item_tail_and_preserves_pointer_metadata() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let pointer = Pointer::new_with_size(0x1000, vec![0x20], String::from("game.exe"), PointerScanPointerSize::Pointer64);
        let mut pointer_project_item = ProjectItemTypePointer::new_project_item("Player Gold", &pointer, "", "u32");
        ProjectItemTypePointer::set_field_evaluated_pointer_path(&mut pointer_project_item, "game.exe+0x1000 -> 0x2020");
        let (project, project_item_path) = create_project_with_item(temp_directory.path(), "player_gold.json", pointer_project_item);
        let mock_promote_bindings = MockPromoteBindings::new(|memory_read_request| {
            assert_eq!(memory_read_request.address, 0x1000);
            assert_eq!(memory_read_request.module_name, "game.exe");

            create_pointer_memory_read_response(0x2000)
        });
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path.clone()],
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 1);
        assert_eq!(promote_symbol_response.reused_symbol_count, 0);
        assert_eq!(promote_symbol_response.promoted_symbol_locator_keys, vec![String::from("absolute:2020")]);
        assert!(promote_symbol_response.conflicts.is_empty());

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(symbol_claims[0].get_locator(), &ProjectSymbolLocator::new_absolute_address(0x2020));
        assert_eq!(symbol_claims[0].get_struct_layout_id(), "u32");
        assert_eq!(symbol_claims[0].get_metadata().get("source.project_item_type"), Some(&String::from("pointer")));
        assert_eq!(
            symbol_claims[0].get_metadata().get("source.pointer_root"),
            Some(&String::from("game.exe+0x1000"))
        );
        assert_eq!(symbol_claims[0].get_metadata().get("source.pointer_offsets"), Some(&String::from("[32]")));
        assert_eq!(
            symbol_claims[0].get_metadata().get("source.pointer_size"),
            Some(&PointerScanPointerSize::Pointer64.to_string())
        );
        assert_eq!(
            symbol_claims[0]
                .get_metadata()
                .get("source.evaluated_pointer_path"),
            Some(&String::from("game.exe+0x1000 -> 0x2020"))
        );
        let promoted_project_item = loaded_project
            .get_project_items()
            .get(&ProjectItemRef::new(project_item_path))
            .expect("Expected pointer project item to remain in the project after promotion.");

        assert_eq!(
            promoted_project_item.get_item_type().get_project_item_type_id(),
            ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID
        );
        assert_eq!(
            ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(promoted_project_item)
                .map(|symbolic_struct_ref| symbolic_struct_ref.get_symbolic_struct_namespace().to_string()),
            Some(String::from("u32"))
        );
        assert_eq!(
            symbol_claims[0]
                .get_metadata()
                .get("source.pointer_root_module"),
            Some(&String::from("game.exe"))
        );
        assert_eq!(
            symbol_claims[0]
                .get_metadata()
                .get("source.pointer_root_offset"),
            Some(&String::from("0x1000"))
        );
    }

    #[test]
    fn promote_symbol_request_reuses_exact_existing_symbol_and_preserves_project_item() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (mut project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .set_symbol_claims(vec![ProjectSymbolClaim::new_module_offset(
                String::from("Health"),
                String::from("game.exe"),
                0x1234,
                String::from("u8"),
            )]);
        project
            .save_to_path(temp_directory.path(), true)
            .expect("Expected test project to save after seeding symbol catalog.");
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default());
        let captured_project_symbol_catalogs = mock_promote_bindings.captured_project_symbol_catalogs();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path.clone()],
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 0);
        assert_eq!(promote_symbol_response.reused_symbol_count, 1);
        assert!(promote_symbol_response.promoted_symbol_locator_keys.is_empty());
        assert!(promote_symbol_response.conflicts.is_empty());

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();
        let promoted_project_item = loaded_project
            .get_project_items()
            .get(&ProjectItemRef::new(project_item_path))
            .expect("Expected project item to remain in project after reuse.");

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(
            promoted_project_item.get_item_type().get_project_item_type_id(),
            ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID
        );
        let mut promoted_project_item = promoted_project_item.clone();
        assert_eq!(
            ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut promoted_project_item)
                .map(|symbolic_struct_ref| symbolic_struct_ref.get_symbolic_struct_namespace().to_string()),
            Some(String::from("u8"))
        );
        let captured_project_symbol_catalogs = captured_project_symbol_catalogs
            .lock()
            .expect("Expected captured symbol catalog lock in test.");
        assert_eq!(captured_project_symbol_catalogs.len(), 1);
        assert_eq!(captured_project_symbol_catalogs[0].get_symbol_claims(), symbol_claims);
        assert_eq!(
            captured_project_symbol_catalogs[0].get_struct_layout_descriptors()[0]
                .get_struct_layout_definition()
                .get_fields()[0]
                .get_field_name(),
            "Health"
        );
    }

    #[test]
    fn promote_symbol_request_reports_conflicts_without_overwriting_existing_symbol() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (mut project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .set_symbol_claims(vec![ProjectSymbolClaim::new_module_offset(
                String::from("Other Health"),
                String::from("game.exe"),
                0x1234,
                String::from("u8"),
            )]);
        project
            .save_to_path(temp_directory.path(), true)
            .expect("Expected test project to save after seeding conflicting symbol.");
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default());
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path.clone()],
            overwrite_conflicting_symbols: false,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 0);
        assert_eq!(promote_symbol_response.reused_symbol_count, 0);
        assert!(promote_symbol_response.promoted_symbol_locator_keys.is_empty());
        assert_eq!(promote_symbol_response.conflicts.len(), 1);
        assert_eq!(promote_symbol_response.conflicts[0].project_item_path, project_item_path);
        assert_eq!(promote_symbol_response.conflicts[0].symbol_locator_key, "module:game.exe:1234");
    }

    #[test]
    fn promote_symbol_request_overwrites_conflicting_symbol_when_requested() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU8::get_value_from_primitive(0));
        let (mut project, project_item_path) = create_project_with_item(temp_directory.path(), "health.json", address_project_item);
        project
            .get_project_info_mut()
            .get_project_symbol_catalog_mut()
            .set_symbol_claims(vec![ProjectSymbolClaim::new_module_offset(
                String::from("Other Health"),
                String::from("game.exe"),
                0x1234,
                String::from("u8"),
            )]);
        project
            .save_to_path(temp_directory.path(), true)
            .expect("Expected test project to save after seeding conflicting symbol.");
        let mock_promote_bindings = MockPromoteBindings::new(|_memory_read_request| MemoryReadResponse::default());
        let captured_project_symbol_catalogs = mock_promote_bindings.captured_project_symbol_catalogs();
        let engine_unprivileged_state = create_engine_unprivileged_state(mock_promote_bindings);

        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let promote_symbol_response = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![project_item_path.clone()],
            overwrite_conflicting_symbols: true,
        }
        .execute(&engine_execution_context);

        assert!(promote_symbol_response.success);
        assert_eq!(promote_symbol_response.promoted_symbol_count, 1);
        assert_eq!(promote_symbol_response.reused_symbol_count, 0);
        assert_eq!(promote_symbol_response.promoted_symbol_locator_keys, vec![String::from("module:game.exe:1234")]);
        assert!(promote_symbol_response.conflicts.is_empty());

        let loaded_project = Project::load_from_path(temp_directory.path()).expect("Expected promoted project to load from disk.");
        let symbol_claims = loaded_project
            .get_project_info()
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 1);
        assert_eq!(
            symbol_claims[0].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x1234)
        );
        assert_eq!(
            loaded_project
                .get_project_items()
                .get(&ProjectItemRef::new(project_item_path))
                .map(|project_item| project_item
                    .get_item_type()
                    .get_project_item_type_id()
                    .to_string()),
            Some(String::from(ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID))
        );
        assert_eq!(
            captured_project_symbol_catalogs
                .lock()
                .expect("Expected captured symbol catalog lock in test.")
                .len(),
            1
        );
    }
}
