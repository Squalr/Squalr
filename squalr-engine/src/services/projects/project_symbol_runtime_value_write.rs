use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType};
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog, symbol_layouts::symbol_layout_size_resolver::SymbolLayoutSizeResolver,
};
use squalr_engine_api::structures::structs::{
    symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
    symbolic_struct_definition::SymbolicStructDefinition,
};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ProjectSymbolRuntimeValueWritePlanRequest {
    pub address: u64,
    pub module_name: String,
    pub symbol_type_id: String,
    pub container_type: ContainerType,
    pub field_name: String,
    pub anonymous_value_string: AnonymousValueString,
}

#[derive(Clone, Debug)]
struct ResolvedSymbolicFieldWriteTarget {
    symbolic_field_definition: SymbolicFieldDefinition,
    offset: u64,
}

pub fn build_project_symbol_runtime_value_write_request(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    write_plan_request: &ProjectSymbolRuntimeValueWritePlanRequest,
) -> Result<MemoryWriteRequest, String> {
    let symbolic_struct_definition = build_named_symbolic_struct_definition_for_value_edit(
        engine_execution_context,
        project_symbol_catalog,
        &write_plan_request.symbol_type_id,
        write_plan_request.container_type,
    )
    .ok_or_else(|| format!("Unable to resolve symbol type `{}`.", write_plan_request.symbol_type_id))?;
    let field_write_target = resolve_symbol_layout_field_write_target(engine_execution_context, &symbolic_struct_definition, &write_plan_request.field_name)
        .ok_or_else(|| format!("Unable to resolve writable field `{}`.", write_plan_request.field_name))?;
    let data_value = engine_execution_context
        .deanonymize_value_string(
            field_write_target.symbolic_field_definition.get_data_type_ref(),
            &normalize_anonymous_value_container_type(
                &write_plan_request.anonymous_value_string,
                field_write_target
                    .symbolic_field_definition
                    .get_container_type(),
            ),
        )
        .map_err(|error| format!("Failed to parse edited symbol value: {}.", error))?;
    let address = write_plan_request
        .address
        .checked_add(field_write_target.offset)
        .ok_or_else(|| String::from("Edited symbol field address overflowed."))?;

    Ok(MemoryWriteRequest {
        address,
        module_name: write_plan_request.module_name.clone(),
        value: data_value.get_value_bytes().clone(),
    })
}

fn normalize_anonymous_value_container_type(
    anonymous_value_string: &AnonymousValueString,
    field_container_type: ContainerType,
) -> AnonymousValueString {
    if anonymous_value_string.get_container_type() != ContainerType::None || field_container_type == ContainerType::None {
        return anonymous_value_string.clone();
    }

    AnonymousValueString::new(
        anonymous_value_string.get_anonymous_value_string().to_string(),
        anonymous_value_string.get_anonymous_value_string_format(),
        field_container_type,
    )
}

fn build_named_symbolic_struct_definition_for_value_edit(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_type_id: &str,
    container_type: ContainerType,
) -> Option<SymbolicStructDefinition> {
    let symbolic_struct_definition = build_symbolic_struct_definition_for_symbol_type(engine_execution_context, project_symbol_catalog, symbol_type_id)?;

    if !symbolic_struct_definition.get_fields().is_empty() {
        return Some(symbolic_struct_definition);
    }

    Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
        DataTypeRef::new(symbol_type_id),
        container_type,
    )]))
}

fn build_symbolic_struct_definition_for_symbol_type(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_type_id: &str,
) -> Option<SymbolicStructDefinition> {
    if let Some(project_struct_layout_descriptor) = project_symbol_catalog
        .get_struct_layout_descriptors()
        .iter()
        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == symbol_type_id)
    {
        return Some(
            project_struct_layout_descriptor
                .get_struct_layout_definition()
                .clone(),
        );
    }

    if let Ok(symbolic_struct_definition) = SymbolicStructDefinition::from_str(symbol_type_id) {
        return Some(symbolic_struct_definition);
    }

    if let Some(symbolic_struct_definition) = engine_execution_context.resolve_struct_layout_definition(symbol_type_id) {
        return Some(symbolic_struct_definition);
    }

    if let Ok(symbolic_field_definition) = SymbolicFieldDefinition::from_str(symbol_type_id) {
        return Some(SymbolicStructDefinition::new_anonymous(vec![symbolic_field_definition]));
    }

    Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
        DataTypeRef::new(symbol_type_id),
        Default::default(),
    )]))
}

fn resolve_symbol_layout_field_write_target(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    symbolic_struct_definition: &SymbolicStructDefinition,
    edited_field_name: &str,
) -> Option<ResolvedSymbolicFieldWriteTarget> {
    let mut next_sequential_offset = 0_u64;
    let mut value_field_index = 0_usize;

    for symbolic_field_definition in symbolic_struct_definition.get_fields() {
        if symbolic_field_definition.is_unassigned() {
            next_sequential_offset = next_sequential_offset.saturating_add(symbolic_field_definition.get_unassigned_size_in_bytes()?);
            continue;
        }

        let field_offset = match symbolic_field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                if symbolic_struct_definition.get_layout_kind().is_union() =>
            {
                0
            }
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
        };

        if normalize_symbol_value_field_name(symbolic_field_definition.get_field_name(), value_field_index) == edited_field_name {
            return Some(ResolvedSymbolicFieldWriteTarget {
                symbolic_field_definition: symbolic_field_definition.clone(),
                offset: field_offset,
            });
        }

        let field_size_in_bytes = resolve_symbolic_field_size_in_bytes(engine_execution_context, symbolic_field_definition, &mut HashSet::new())?;
        next_sequential_offset = next_sequential_offset.max(field_offset.checked_add(field_size_in_bytes)?);
        value_field_index = value_field_index.saturating_add(1);
    }

    None
}

fn normalize_symbol_value_field_name(
    field_name: &str,
    field_index: usize,
) -> String {
    if field_name.trim().is_empty() {
        if field_index == 0 {
            String::from("value")
        } else {
            format!("value_{}", field_index)
        }
    } else {
        field_name.to_string()
    }
}

fn resolve_symbolic_field_size_in_bytes(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    symbolic_field_definition: &SymbolicFieldDefinition,
    visited_type_ids: &mut HashSet<String>,
) -> Option<u64> {
    SymbolLayoutSizeResolver::resolve_symbolic_field_size_in_bytes(
        symbolic_field_definition,
        |data_type_ref| {
            engine_execution_context
                .get_default_value(data_type_ref)
                .map(|default_value| default_value.get_size_in_bytes())
        },
        |struct_layout_id| engine_execution_context.resolve_struct_layout_definition(struct_layout_id),
        visited_type_ids,
    )
}
