use crate::commands::project_symbols::create::project_symbols_create_request::ProjectSymbolsCreateRequest;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType};
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use crate::structures::structs::symbolic_field_definition::SymbolicFieldDefinition;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct DefineFieldPlanRequest {
    pub display_name: String,
    pub relative_offset_text: String,
    pub relative_offset_format: AnonymousValueStringFormat,
    pub container_type: ContainerType,
    pub data_type_ref: DataTypeRef,
}

#[derive(Clone, Debug)]
pub struct DefineFieldPlan {
    pub project_symbols_create_request: ProjectSymbolsCreateRequest,
}

pub fn filter_registered_pointer_sizes(registered_data_type_refs: &[DataTypeRef]) -> Vec<PointerScanPointerSize> {
    let registered_data_type_ids = registered_data_type_refs
        .iter()
        .map(|data_type_ref| data_type_ref.get_data_type_id().to_string())
        .collect::<HashSet<_>>();

    PointerScanPointerSize::ALL
        .iter()
        .copied()
        .filter(|pointer_size| registered_data_type_ids.contains(pointer_size.to_data_type_ref().get_data_type_id()))
        .collect()
}

pub fn parse_define_field_relative_offset(
    relative_offset_text: &str,
    relative_offset_format: AnonymousValueStringFormat,
) -> Result<u64, String> {
    let trimmed_relative_offset_text = relative_offset_text.trim();

    if trimmed_relative_offset_text.is_empty() {
        return Err(String::from("Offset is required."));
    }

    let normalized_binary_text = trimmed_relative_offset_text
        .strip_prefix("0b")
        .or_else(|| trimmed_relative_offset_text.strip_prefix("0B"));

    if let Some(binary_text) = normalized_binary_text {
        if binary_text.is_empty() {
            return Err(String::from("Binary offset is missing digits."));
        }

        return u64::from_str_radix(binary_text, 2).map_err(|_| format!("Invalid binary offset: {}.", trimmed_relative_offset_text));
    }

    let normalized_hex_text = trimmed_relative_offset_text
        .strip_prefix("0x")
        .or_else(|| trimmed_relative_offset_text.strip_prefix("0X"));

    if let Some(hex_text) = normalized_hex_text {
        if hex_text.is_empty() {
            return Err(String::from("Hex offset is missing digits."));
        }

        return u64::from_str_radix(hex_text, 16).map_err(|_| format!("Invalid hex offset: {}.", trimmed_relative_offset_text));
    }

    match relative_offset_format {
        AnonymousValueStringFormat::Binary => {
            u64::from_str_radix(trimmed_relative_offset_text, 2).map_err(|_| format!("Invalid binary offset: {}.", trimmed_relative_offset_text))
        }
        AnonymousValueStringFormat::Hexadecimal | AnonymousValueStringFormat::Address => {
            u64::from_str_radix(trimmed_relative_offset_text, 16).map_err(|_| format!("Invalid hex offset: {}.", trimmed_relative_offset_text))
        }
        _ => trimmed_relative_offset_text
            .parse::<u64>()
            .map_err(|_| format!("Invalid decimal offset: {}.", trimmed_relative_offset_text)),
    }
}

pub fn build_define_field_symbol_layout_id(
    data_type_ref: &DataTypeRef,
    container_type: ContainerType,
) -> String {
    let symbolic_field_definition = SymbolicFieldDefinition::new(data_type_ref.clone(), container_type);

    symbolic_field_definition.to_string()
}

pub fn build_define_field_plan(
    define_field_plan_request: &DefineFieldPlanRequest,
    module_name: &str,
    segment_offset: u64,
    segment_length: u64,
    resolve_type_size: impl Fn(&str) -> Option<u64>,
) -> Result<DefineFieldPlan, String> {
    let display_name = define_field_plan_request.display_name.trim();

    if display_name.is_empty() {
        return Err(String::from("Field name is required."));
    }

    let relative_offset = parse_define_field_relative_offset(
        &define_field_plan_request.relative_offset_text,
        define_field_plan_request.relative_offset_format,
    )?;
    let struct_layout_id = build_define_field_symbol_layout_id(&define_field_plan_request.data_type_ref, define_field_plan_request.container_type);
    let Some(field_size) = resolve_type_size(&struct_layout_id) else {
        return Err(format!("Cannot resolve byte size for `{}`.", struct_layout_id));
    };

    if field_size == 0 {
        return Err(format!("`{}` has no byte size.", struct_layout_id));
    }

    let Some(relative_field_end) = relative_offset.checked_add(field_size) else {
        return Err(String::from("Field range is too large."));
    };

    if relative_field_end > segment_length {
        return Err(format!(
            "`{}` is {} byte(s), which does not fit inside this unassigned segment at offset 0x{:X}.",
            struct_layout_id, field_size, relative_offset
        ));
    }

    let Some(absolute_offset) = segment_offset.checked_add(relative_offset) else {
        return Err(String::from("Module offset is too large."));
    };

    Ok(DefineFieldPlan {
        project_symbols_create_request: ProjectSymbolsCreateRequest {
            display_name: display_name.to_string(),
            struct_layout_id,
            address: None,
            module_name: Some(module_name.to_string()),
            offset: Some(absolute_offset),
            metadata: Default::default(),
        },
    })
}
