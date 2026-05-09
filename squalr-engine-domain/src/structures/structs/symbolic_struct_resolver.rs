use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::container_type::ContainerType,
    structs::{
        symbolic_expression::{SymbolicExpression, SymbolicExpressionEvaluationError},
        symbolic_field_definition::{SymbolicFieldCountResolution, SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverEvaluationError, SymbolicResolverRelativeSymbolPath},
        symbolic_struct_definition::SymbolicStructDefinition,
    },
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSymbolicStruct {
    symbol_namespace: String,
    fields: Vec<ResolvedSymbolicField>,
}

impl ResolvedSymbolicStruct {
    pub fn new(
        symbol_namespace: String,
        fields: Vec<ResolvedSymbolicField>,
    ) -> Self {
        Self { symbol_namespace, fields }
    }

    pub fn get_symbol_namespace(&self) -> &str {
        &self.symbol_namespace
    }

    pub fn get_fields(&self) -> &[ResolvedSymbolicField] {
        &self.fields
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSymbolicField {
    field_name: String,
    data_type_ref: DataTypeRef,
    container_type: ContainerType,
    offset_in_bytes: Option<u64>,
    element_count: Option<u64>,
    displayed_element_count: Option<u64>,
    size_in_bytes: Option<u64>,
    status: ResolvedSymbolicFieldStatus,
}

impl ResolvedSymbolicField {
    pub fn new(
        field_name: String,
        data_type_ref: DataTypeRef,
        container_type: ContainerType,
        offset_in_bytes: Option<u64>,
        element_count: Option<u64>,
        displayed_element_count: Option<u64>,
        size_in_bytes: Option<u64>,
        status: ResolvedSymbolicFieldStatus,
    ) -> Self {
        Self {
            field_name,
            data_type_ref,
            container_type,
            offset_in_bytes,
            element_count,
            displayed_element_count,
            size_in_bytes,
            status,
        }
    }

    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.data_type_ref
    }

    pub fn get_container_type(&self) -> ContainerType {
        self.container_type
    }

    pub fn get_offset_in_bytes(&self) -> Option<u64> {
        self.offset_in_bytes
    }

    pub fn get_element_count(&self) -> Option<u64> {
        self.element_count
    }

    pub fn get_displayed_element_count(&self) -> Option<u64> {
        self.displayed_element_count
    }

    pub fn get_size_in_bytes(&self) -> Option<u64> {
        self.size_in_bytes
    }

    pub fn get_status(&self) -> &ResolvedSymbolicFieldStatus {
        &self.status
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedSymbolicFieldStatus {
    Ready,
    Unresolved {
        reason: String,
    },
    Clamped {
        actual_element_count: u64,
        displayed_element_count: u64,
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolicStructResolverOptions {
    max_dynamic_array_preview_elements: u64,
}

impl SymbolicStructResolverOptions {
    pub fn new(max_dynamic_array_preview_elements: u64) -> Self {
        Self {
            max_dynamic_array_preview_elements,
        }
    }

    pub fn get_max_dynamic_array_preview_elements(&self) -> u64 {
        self.max_dynamic_array_preview_elements
    }
}

impl Default for SymbolicStructResolverOptions {
    fn default() -> Self {
        Self::new(256)
    }
}

pub fn resolve_symbolic_struct_definition<ResolveTypeSize, ReadScalarField>(
    symbolic_struct_definition: &SymbolicStructDefinition,
    resolve_type_size_in_bytes: ResolveTypeSize,
    read_scalar_field: ReadScalarField,
    options: &SymbolicStructResolverOptions,
) -> ResolvedSymbolicStruct
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    resolve_symbolic_struct_definition_with_resolvers(symbolic_struct_definition, resolve_type_size_in_bytes, read_scalar_field, |_| None, options)
}

pub fn resolve_symbolic_struct_definition_with_resolvers<ResolveTypeSize, ReadScalarField, ResolveResolverDefinition>(
    symbolic_struct_definition: &SymbolicStructDefinition,
    resolve_type_size_in_bytes: ResolveTypeSize,
    read_scalar_field: ReadScalarField,
    resolve_resolver_definition: ResolveResolverDefinition,
    options: &SymbolicStructResolverOptions,
) -> ResolvedSymbolicStruct
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
    ResolveResolverDefinition: Fn(&str) -> Option<SymbolicResolverDefinition>,
{
    resolve_symbolic_struct_definition_with_resolvers_and_relative_symbol_fields(
        symbolic_struct_definition,
        resolve_type_size_in_bytes,
        read_scalar_field,
        resolve_resolver_definition,
        |_| None,
        options,
    )
}

pub fn resolve_symbolic_struct_definition_with_resolvers_and_relative_symbol_fields<
    ResolveTypeSize,
    ReadScalarField,
    ResolveResolverDefinition,
    ResolveStructDefinition,
>(
    symbolic_struct_definition: &SymbolicStructDefinition,
    resolve_type_size_in_bytes: ResolveTypeSize,
    read_scalar_field: ReadScalarField,
    resolve_resolver_definition: ResolveResolverDefinition,
    resolve_struct_definition: ResolveStructDefinition,
    options: &SymbolicStructResolverOptions,
) -> ResolvedSymbolicStruct
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
    ResolveResolverDefinition: Fn(&str) -> Option<SymbolicResolverDefinition>,
    ResolveStructDefinition: Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
{
    resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields(
        symbolic_struct_definition,
        resolve_type_size_in_bytes,
        read_scalar_field,
        resolve_resolver_definition,
        resolve_struct_definition,
        |module_name, symbol_path| {
            Err(SymbolicResolverEvaluationError::UnknownGlobalSymbolPath(format!(
                "{}.{}",
                module_name, symbol_path
            )))
        },
        options,
    )
}

pub fn resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields<
    ResolveTypeSize,
    ReadScalarField,
    ResolveResolverDefinition,
    ResolveStructDefinition,
    ResolveGlobalSymbolField,
>(
    symbolic_struct_definition: &SymbolicStructDefinition,
    resolve_type_size_in_bytes: ResolveTypeSize,
    read_scalar_field: ReadScalarField,
    resolve_resolver_definition: ResolveResolverDefinition,
    resolve_struct_definition: ResolveStructDefinition,
    resolve_global_symbol_field: ResolveGlobalSymbolField,
    options: &SymbolicStructResolverOptions,
) -> ResolvedSymbolicStruct
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
    ResolveResolverDefinition: Fn(&str) -> Option<SymbolicResolverDefinition>,
    ResolveStructDefinition: Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    ResolveGlobalSymbolField: Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
{
    let mut scalar_values_by_field_name = BTreeMap::new();
    let mut resolved_fields = Vec::new();
    let resolve_pass_count = symbolic_struct_definition
        .get_fields()
        .len()
        .saturating_add(1)
        .max(1);

    for _ in 0..resolve_pass_count {
        let resolve_pass = resolve_symbolic_struct_definition_pass(
            symbolic_struct_definition,
            &resolve_type_size_in_bytes,
            &read_scalar_field,
            &resolve_resolver_definition,
            &resolve_struct_definition,
            &resolve_global_symbol_field,
            options,
            &mut scalar_values_by_field_name,
        );
        resolved_fields = resolve_pass.resolved_fields;

        if !resolve_pass.did_update_scalar_value {
            break;
        }
    }

    ResolvedSymbolicStruct::new(symbolic_struct_definition.get_symbol_namespace().to_string(), resolved_fields)
}

struct SymbolicStructResolvePass {
    resolved_fields: Vec<ResolvedSymbolicField>,
    did_update_scalar_value: bool,
}

fn resolve_symbolic_struct_definition_pass<ResolveTypeSize, ReadScalarField>(
    symbolic_struct_definition: &SymbolicStructDefinition,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
    resolve_struct_definition: &impl Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    resolve_global_symbol_field: &impl Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    options: &SymbolicStructResolverOptions,
    scalar_values_by_field_name: &mut BTreeMap<String, i128>,
) -> SymbolicStructResolvePass
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    let mut resolved_fields = Vec::new();
    let mut did_update_scalar_value = false;
    let mut next_sequential_offset = 0_u64;

    for field_definition in symbolic_struct_definition.get_fields() {
        let field_offset_result = resolve_field_offset(
            symbolic_struct_definition,
            field_definition,
            next_sequential_offset,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            &mut Vec::new(),
        );
        let element_count_result = resolve_field_element_count(
            symbolic_struct_definition,
            field_definition,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            &mut Vec::new(),
        );
        let display_element_count_result = resolve_field_display_element_count(
            symbolic_struct_definition,
            field_definition,
            &element_count_result,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            &mut Vec::new(),
        );
        let unit_size_in_bytes = resolve_type_size_in_bytes(field_definition.get_data_type_ref());
        let mut field_status = build_field_status(&field_offset_result, &element_count_result, &display_element_count_result, unit_size_in_bytes);
        let display_element_count = display_element_count_result
            .as_ref()
            .ok()
            .and_then(|element_count| *element_count)
            .map(|display_element_count| {
                element_count_result
                    .as_ref()
                    .ok()
                    .and_then(|element_count| *element_count)
                    .map(|storage_element_count| display_element_count.min(storage_element_count))
                    .unwrap_or(display_element_count)
            });
        let displayed_element_count = display_element_count.map(|element_count| element_count.min(options.get_max_dynamic_array_preview_elements()));
        if let (Some(display_element_count), Some(displayed_element_count)) = (display_element_count, displayed_element_count) {
            if display_element_count > displayed_element_count && matches!(field_status, ResolvedSymbolicFieldStatus::Ready) {
                field_status = ResolvedSymbolicFieldStatus::Clamped {
                    actual_element_count: display_element_count,
                    displayed_element_count,
                    reason: format!(
                        "Dynamic array count exceeds preview limit of {}.",
                        options.get_max_dynamic_array_preview_elements()
                    ),
                };
            }
        }
        let field_size_in_bytes = unit_size_in_bytes.and_then(|unit_size| {
            element_count_result
                .as_ref()
                .ok()
                .and_then(|element_count| *element_count)
                .and_then(|element_count| unit_size.checked_mul(element_count))
        });
        let field_offset = field_offset_result.as_ref().ok().copied().flatten();

        if let Some(field_size) = field_size_in_bytes {
            if let Some(field_offset) = field_offset {
                next_sequential_offset = field_offset.saturating_add(field_size);
            }
        }

        if maybe_capture_scalar_field_value(
            field_definition,
            field_offset,
            field_size_in_bytes,
            read_scalar_field,
            scalar_values_by_field_name,
        ) {
            did_update_scalar_value = true;
        }

        resolved_fields.push(ResolvedSymbolicField::new(
            field_definition.get_field_name().to_string(),
            field_definition.get_data_type_ref().clone(),
            field_definition.get_container_type(),
            field_offset,
            element_count_result
                .as_ref()
                .ok()
                .and_then(|element_count| *element_count),
            displayed_element_count,
            field_size_in_bytes,
            field_status,
        ));
    }

    SymbolicStructResolvePass {
        resolved_fields,
        did_update_scalar_value,
    }
}

fn resolve_field_offset<ResolveTypeSize, ReadScalarField>(
    root_struct_definition: &SymbolicStructDefinition,
    field_definition: &SymbolicFieldDefinition,
    next_sequential_offset: u64,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
    resolve_struct_definition: &impl Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    resolve_global_symbol_field: &impl Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<Option<u64>, String>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    match field_definition.get_offset_resolution() {
        SymbolicFieldOffsetResolution::Sequential => Ok(Some(next_sequential_offset)),
        SymbolicFieldOffsetResolution::Expression(offset_expression) => {
            evaluate_u64_expression(offset_expression, scalar_values_by_field_name, resolve_type_size_in_bytes).map(Some)
        }
        SymbolicFieldOffsetResolution::Resolver(resolver_id) => evaluate_u64_resolver(
            root_struct_definition,
            resolver_id,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            resolver_stack,
        )
        .map(Some),
    }
}

fn resolve_field_element_count<ReadScalarField>(
    root_struct_definition: &SymbolicStructDefinition,
    field_definition: &SymbolicFieldDefinition,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &impl Fn(&DataTypeRef) -> Option<u64>,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
    resolve_struct_definition: &impl Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    resolve_global_symbol_field: &impl Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<Option<u64>, String>
where
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    match field_definition.get_count_resolution() {
        SymbolicFieldCountResolution::Expression(count_expression) => {
            evaluate_u64_expression(count_expression, scalar_values_by_field_name, resolve_type_size_in_bytes).map(Some)
        }
        SymbolicFieldCountResolution::Resolver(resolver_id) => evaluate_u64_resolver(
            root_struct_definition,
            resolver_id,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            resolver_stack,
        )
        .map(Some),
        SymbolicFieldCountResolution::Inferred => match field_definition.get_container_type() {
            ContainerType::ArrayFixed(element_count) => Ok(Some(element_count)),
            ContainerType::Array => Ok(None),
            ContainerType::None | ContainerType::Pointer(_) | ContainerType::Pointer32 | ContainerType::Pointer64 => Ok(Some(1)),
        },
    }
}

fn resolve_field_display_element_count<ReadScalarField>(
    root_struct_definition: &SymbolicStructDefinition,
    field_definition: &SymbolicFieldDefinition,
    element_count_result: &Result<Option<u64>, String>,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &impl Fn(&DataTypeRef) -> Option<u64>,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
    resolve_struct_definition: &impl Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    resolve_global_symbol_field: &impl Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<Option<u64>, String>
where
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    match field_definition.get_display_count_resolution() {
        SymbolicFieldCountResolution::Inferred => element_count_result.clone(),
        SymbolicFieldCountResolution::Expression(count_expression) => {
            evaluate_u64_expression(count_expression, scalar_values_by_field_name, resolve_type_size_in_bytes).map(Some)
        }
        SymbolicFieldCountResolution::Resolver(resolver_id) => evaluate_u64_resolver(
            root_struct_definition,
            resolver_id,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            resolver_stack,
        )
        .map(Some),
    }
}

fn evaluate_u64_resolver<ResolveTypeSize, ReadScalarField>(
    root_struct_definition: &SymbolicStructDefinition,
    resolver_id: &str,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
    resolve_struct_definition: &impl Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    resolve_global_symbol_field: &impl Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<u64, String>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    if resolver_stack
        .iter()
        .any(|stacked_resolver_id| stacked_resolver_id == resolver_id)
    {
        return Err(SymbolicResolverEvaluationError::ResolverCycle(resolver_id.to_string()).to_string());
    }

    let resolver_definition = resolve_resolver_definition(resolver_id).ok_or_else(|| format!("Unknown resolver `{}`.", resolver_id))?;
    resolver_stack.push(resolver_id.to_string());
    let value = {
        let mut resolve_relative_symbol_field = |symbol_path: &SymbolicResolverRelativeSymbolPath| {
            resolve_relative_symbol_path_value(
                symbol_path,
                root_struct_definition,
                scalar_values_by_field_name,
                resolve_type_size_in_bytes,
                read_scalar_field,
                resolve_resolver_definition,
                resolve_struct_definition,
                resolve_global_symbol_field,
                resolver_stack,
            )
        };

        resolver_definition
            .evaluate_with_symbol_fields(
                &|field_name| scalar_values_by_field_name.get(field_name).copied(),
                resolve_type_size_in_bytes,
                &mut resolve_relative_symbol_field,
                resolve_global_symbol_field,
            )
            .map_err(format_resolver_error)
    };
    resolver_stack.pop();
    let value = value?;

    u64::try_from(value).map_err(|_| format!("Resolver `{}` resolved to negative or too-large value `{}`.", resolver_id, value))
}

fn resolve_relative_symbol_path_value<ResolveTypeSize, ReadScalarField>(
    symbol_path: &SymbolicResolverRelativeSymbolPath,
    root_struct_definition: &SymbolicStructDefinition,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
    resolve_struct_definition: &impl Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    resolve_global_symbol_field: &impl Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<i128, SymbolicResolverEvaluationError>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    if symbol_path.is_empty() {
        return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()));
    }

    let mut current_struct_definition = root_struct_definition.clone();
    let mut current_base_offset = 0_u64;

    for (symbol_path_segment_index, symbol_path_segment) in symbol_path.get_segments().iter().enumerate() {
        let (field_definition, field_offset) = resolve_named_field_offset_in_struct(
            symbol_path,
            symbol_path_segment,
            &current_struct_definition,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            resolver_stack,
        )?;
        let resolved_field_offset = current_base_offset.saturating_add(field_offset);
        let is_terminal_segment = symbol_path_segment_index + 1 == symbol_path.get_segments().len();

        if is_terminal_segment {
            if !matches!(field_definition.get_container_type(), ContainerType::None) {
                return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(format!(
                    "{} is not a scalar field.",
                    symbol_path
                )));
            }

            let field_size_in_bytes = resolve_type_size_in_bytes(field_definition.get_data_type_ref())
                .ok_or_else(|| SymbolicResolverEvaluationError::UnknownTypeSize(field_definition.get_data_type_ref().to_string()))?;

            return read_scalar_field(field_definition, resolved_field_offset, field_size_in_bytes)
                .map_err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath)?
                .ok_or_else(|| SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()));
        }

        current_struct_definition = resolve_struct_definition(field_definition.get_data_type_ref())
            .ok_or_else(|| SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()))?;
        current_base_offset = resolved_field_offset;
    }

    Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()))
}

fn resolve_named_field_offset_in_struct<'definition, ResolveTypeSize, ReadScalarField>(
    symbol_path: &SymbolicResolverRelativeSymbolPath,
    symbol_path_segment: &str,
    current_struct_definition: &'definition SymbolicStructDefinition,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
    resolve_struct_definition: &impl Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    resolve_global_symbol_field: &impl Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<(&'definition SymbolicFieldDefinition, u64), SymbolicResolverEvaluationError>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    let mut next_sequential_offset = 0_u64;

    for field_definition in current_struct_definition.get_fields() {
        let field_offset = resolve_field_offset(
            current_struct_definition,
            field_definition,
            next_sequential_offset,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            resolver_stack,
        )
        .map_err(|error| SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(format!("{}: {}", symbol_path, error)))?
        .ok_or_else(|| SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()))?;

        if field_definition.get_field_name() == symbol_path_segment {
            return Ok((field_definition, field_offset));
        }

        let field_size_in_bytes = resolve_field_static_size_in_bytes(field_definition, resolve_type_size_in_bytes);

        if let Some(field_size_in_bytes) = field_size_in_bytes {
            next_sequential_offset = field_offset.saturating_add(field_size_in_bytes);
        }
    }

    Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()))
}

fn resolve_field_static_size_in_bytes(
    field_definition: &SymbolicFieldDefinition,
    resolve_type_size_in_bytes: &impl Fn(&DataTypeRef) -> Option<u64>,
) -> Option<u64> {
    let unit_size_in_bytes = resolve_type_size_in_bytes(field_definition.get_data_type_ref())?;

    match field_definition.get_container_type() {
        ContainerType::ArrayFixed(element_count) => unit_size_in_bytes.checked_mul(element_count),
        ContainerType::None | ContainerType::Pointer(_) | ContainerType::Pointer32 | ContainerType::Pointer64 => Some(unit_size_in_bytes),
        ContainerType::Array => None,
    }
}

fn evaluate_u64_expression<ResolveTypeSize>(
    expression: &SymbolicExpression,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &ResolveTypeSize,
) -> Result<u64, String>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
{
    let value = expression
        .evaluate(&|identifier| scalar_values_by_field_name.get(identifier).copied(), resolve_type_size_in_bytes)
        .map_err(format_expression_error)?;

    u64::try_from(value).map_err(|_| format!("Expression `{}` resolved to negative or too-large value `{}`.", expression, value))
}

fn build_field_status(
    field_offset_result: &Result<Option<u64>, String>,
    element_count_result: &Result<Option<u64>, String>,
    display_element_count_result: &Result<Option<u64>, String>,
    unit_size_in_bytes: Option<u64>,
) -> ResolvedSymbolicFieldStatus {
    if let Err(error) = field_offset_result {
        return ResolvedSymbolicFieldStatus::Unresolved { reason: error.to_string() };
    }

    if let Err(error) = element_count_result {
        return ResolvedSymbolicFieldStatus::Unresolved { reason: error.to_string() };
    }

    if let Err(error) = display_element_count_result {
        return ResolvedSymbolicFieldStatus::Unresolved { reason: error.to_string() };
    }

    if unit_size_in_bytes.is_none() {
        return ResolvedSymbolicFieldStatus::Unresolved {
            reason: String::from("Type size is unknown."),
        };
    }

    ResolvedSymbolicFieldStatus::Ready
}

fn maybe_capture_scalar_field_value<ReadScalarField>(
    field_definition: &SymbolicFieldDefinition,
    field_offset: Option<u64>,
    field_size_in_bytes: Option<u64>,
    read_scalar_field: &ReadScalarField,
    scalar_values_by_field_name: &mut BTreeMap<String, i128>,
) -> bool
where
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    if field_definition.get_field_name().is_empty() || !field_definition.get_count_resolution().is_inferred() {
        return false;
    }

    if !matches!(field_definition.get_container_type(), ContainerType::None) {
        return false;
    }

    let (Some(field_offset), Some(field_size_in_bytes)) = (field_offset, field_size_in_bytes) else {
        return false;
    };
    let Ok(Some(field_value)) = read_scalar_field(field_definition, field_offset, field_size_in_bytes) else {
        return false;
    };

    match scalar_values_by_field_name.insert(field_definition.get_field_name().to_string(), field_value) {
        Some(previous_field_value) => previous_field_value != field_value,
        None => true,
    }
}

fn format_expression_error(error: SymbolicExpressionEvaluationError) -> String {
    error.to_string()
}

fn format_resolver_error(error: SymbolicResolverEvaluationError) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        ResolvedSymbolicFieldStatus, SymbolicStructResolverOptions, resolve_symbolic_struct_definition, resolve_symbolic_struct_definition_with_resolvers,
        resolve_symbolic_struct_definition_with_resolvers_and_relative_symbol_fields,
    };
    use crate::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        structs::symbolic_resolver_definition::{
            SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverNode, SymbolicResolverRelativeSymbolPath,
        },
    };
    use std::str::FromStr;

    #[test]
    fn resolver_uses_previous_scalar_fields_for_vec_like_dynamic_layout() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![
                SymbolicFieldDefinition::from_str("count:u32 @ +0").expect("Expected count field to parse."),
                SymbolicFieldDefinition::from_str("capacity:u32 @ +4").expect("Expected capacity field to parse."),
                SymbolicFieldDefinition::from_str("elements:Element[count] @ +8").expect("Expected elements field to parse."),
                SymbolicFieldDefinition::from_str("unfilled:Element[capacity - count] @ +8 + count * sizeof(Element)")
                    .expect("Expected unfilled field to parse."),
            ],
        );

        let resolved_struct = resolve_symbolic_struct_definition(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "Element" => Some(32),
                _ => None,
            },
            |field_definition, _, _| match field_definition.get_field_name() {
                "count" => Ok(Some(4)),
                "capacity" => Ok(Some(12)),
                _ => Ok(None),
            },
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[2].get_field_name(), "elements");
        assert_eq!(resolved_fields[2].get_offset_in_bytes(), Some(8));
        assert_eq!(resolved_fields[2].get_element_count(), Some(4));
        assert_eq!(resolved_fields[2].get_size_in_bytes(), Some(128));
        assert_eq!(resolved_fields[3].get_field_name(), "unfilled");
        assert_eq!(resolved_fields[3].get_offset_in_bytes(), Some(136));
        assert_eq!(resolved_fields[3].get_element_count(), Some(8));
        assert_eq!(resolved_fields[3].get_size_in_bytes(), Some(256));
        assert_eq!(resolved_fields[3].get_status(), &ResolvedSymbolicFieldStatus::Ready);
    }

    #[test]
    fn resolver_uses_forward_scalar_fields_when_offsets_are_explicit() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![
                SymbolicFieldDefinition::from_str("elements:Element[count] @ +8").expect("Expected elements field to parse."),
                SymbolicFieldDefinition::from_str("count:u32 @ +4").expect("Expected count field to parse."),
            ],
        );

        let resolved_struct = resolve_symbolic_struct_definition(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "Element" => Some(32),
                _ => None,
            },
            |field_definition, _, _| match field_definition.get_field_name() {
                "count" => Ok(Some(3)),
                _ => Ok(None),
            },
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[0].get_offset_in_bytes(), Some(8));
        assert_eq!(resolved_fields[0].get_element_count(), Some(3));
        assert_eq!(resolved_fields[0].get_size_in_bytes(), Some(96));
        assert_eq!(resolved_fields[0].get_status(), &ResolvedSymbolicFieldStatus::Ready);
        assert_eq!(resolved_fields[1].get_offset_in_bytes(), Some(4));
    }

    #[test]
    fn resolver_reports_expression_diagnostics_on_affected_field() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![
                SymbolicFieldDefinition::from_str("count:u32 @ +0").expect("Expected count field to parse."),
                SymbolicFieldDefinition::from_str("elements:Element[count / divisor] @ +4").expect("Expected elements field to parse."),
            ],
        );

        let resolved_struct = resolve_symbolic_struct_definition(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "Element" => Some(32),
                _ => None,
            },
            |field_definition, _, _| match field_definition.get_field_name() {
                "count" => Ok(Some(4)),
                _ => Ok(None),
            },
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert!(matches!(
            resolved_fields[1].get_status(),
            ResolvedSymbolicFieldStatus::Unresolved { reason } if reason.contains("Unknown identifier `divisor`")
        ));
        assert_eq!(resolved_fields[1].get_offset_in_bytes(), Some(4));
        assert_eq!(resolved_fields[1].get_size_in_bytes(), None);
    }

    #[test]
    fn resolver_clamps_dynamic_array_preview_count() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![
                SymbolicFieldDefinition::from_str("count:u32").expect("Expected count field to parse."),
                SymbolicFieldDefinition::from_str("elements:Element[count]").expect("Expected elements field to parse."),
            ],
        );

        let resolved_struct = resolve_symbolic_struct_definition(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "Element" => Some(32),
                _ => None,
            },
            |field_definition, _, _| match field_definition.get_field_name() {
                "count" => Ok(Some(1000)),
                _ => Ok(None),
            },
            &SymbolicStructResolverOptions::new(16),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[1].get_element_count(), Some(1000));
        assert_eq!(resolved_fields[1].get_displayed_element_count(), Some(16));
        assert_eq!(resolved_fields[1].get_size_in_bytes(), Some(32000));
    }

    #[test]
    fn resolver_uses_display_count_without_shrinking_fixed_storage() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("EntityList"),
            vec![
                SymbolicFieldDefinition::from_str("count:u32").expect("Expected count field to parse."),
                SymbolicFieldDefinition::from_str("entities:u64[1024] display count @ +8").expect("Expected entities field to parse."),
                SymbolicFieldDefinition::from_str("tail:u32").expect("Expected tail field to parse."),
            ],
        );

        let resolved_struct = resolve_symbolic_struct_definition(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "u64" => Some(8),
                _ => None,
            },
            |field_definition, _, _| match field_definition.get_field_name() {
                "count" => Ok(Some(3)),
                _ => Ok(None),
            },
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[1].get_element_count(), Some(1024));
        assert_eq!(resolved_fields[1].get_displayed_element_count(), Some(3));
        assert_eq!(resolved_fields[1].get_size_in_bytes(), Some(8192));
        assert_eq!(resolved_fields[2].get_offset_in_bytes(), Some(8200));
    }

    #[test]
    fn resolver_supports_sizeof_in_offset_expressions() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![SymbolicFieldDefinition::from_str("tail:u32 @ sizeof(Element) * 3").expect("Expected tail field to parse.")],
        );

        let resolved_struct = resolve_symbolic_struct_definition(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "Element" => Some(32),
                _ => None,
            },
            |_, _, _| Ok(None),
            &SymbolicStructResolverOptions::default(),
        );

        assert_eq!(resolved_struct.get_fields()[0].get_offset_in_bytes(), Some(96));
    }

    #[test]
    fn resolver_supports_sizeof_in_count_expressions() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![
                SymbolicFieldDefinition::from_str("count:u32").expect("Expected count field to parse."),
                SymbolicFieldDefinition::from_str("capacity:u32").expect("Expected capacity field to parse."),
                SymbolicFieldDefinition::from_str("unfilled:u8[(capacity - count) * sizeof(Element)]").expect("Expected unfilled field to parse."),
            ],
        );

        let resolved_struct = resolve_symbolic_struct_definition(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u32" => Some(4),
                "Element" => Some(32),
                _ => None,
            },
            |field_definition, _, _| match field_definition.get_field_name() {
                "count" => Ok(Some(4)),
                "capacity" => Ok(Some(12)),
                _ => Ok(None),
            },
            &SymbolicStructResolverOptions::default(),
        );

        assert_eq!(resolved_struct.get_fields()[2].get_element_count(), Some(256));
        assert_eq!(resolved_struct.get_fields()[2].get_size_in_bytes(), Some(256));
    }

    #[test]
    fn resolver_uses_catalog_resolvers_for_dynamic_layout() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![
                SymbolicFieldDefinition::from_str("count:u32").expect("Expected count field to parse."),
                SymbolicFieldDefinition::from_str("elements:Element[resolver(item.count)] @ resolver(item.offset)").expect("Expected resolver field to parse."),
            ],
        );
        let count_resolver = SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("count")));
        let offset_resolver = SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
            SymbolicResolverBinaryOperator::Add,
            SymbolicResolverNode::new_literal(4),
            SymbolicResolverNode::new_type_size(DataTypeRef::new("u32")),
        ));

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "Element" => Some(32),
                _ => None,
            },
            |field_definition, _, _| match field_definition.get_field_name() {
                "count" => Ok(Some(3)),
                _ => Ok(None),
            },
            |resolver_id| match resolver_id {
                "item.count" => Some(count_resolver.clone()),
                "item.offset" => Some(offset_resolver.clone()),
                _ => None,
            },
            &SymbolicStructResolverOptions::default(),
        );

        assert_eq!(resolved_struct.get_fields()[1].get_offset_in_bytes(), Some(8));
        assert_eq!(resolved_struct.get_fields()[1].get_element_count(), Some(3));
    }

    #[test]
    fn resolver_uses_nested_relative_symbol_fields_for_dynamic_layout() {
        let dos_header_definition = SymbolicStructDefinition::new(
            String::from("DosHeader"),
            vec![
                SymbolicFieldDefinition::from_str("padding:u8[60]").expect("Expected padding field to parse."),
                SymbolicFieldDefinition::from_str("e_lfanew:u32").expect("Expected e_lfanew field to parse."),
            ],
        );
        let file_header_definition = SymbolicStructDefinition::new(
            String::from("FileHeader"),
            vec![
                SymbolicFieldDefinition::from_str("Machine:u16").expect("Expected Machine field to parse."),
                SymbolicFieldDefinition::from_str("NumberOfSections:u16").expect("Expected NumberOfSections field to parse."),
                SymbolicFieldDefinition::from_str("TimeDateStamp:u32").expect("Expected TimeDateStamp field to parse."),
                SymbolicFieldDefinition::from_str("PointerToSymbolTable:u32").expect("Expected PointerToSymbolTable field to parse."),
                SymbolicFieldDefinition::from_str("NumberOfSymbols:u32").expect("Expected NumberOfSymbols field to parse."),
                SymbolicFieldDefinition::from_str("SizeOfOptionalHeader:u16").expect("Expected SizeOfOptionalHeader field to parse."),
            ],
        );
        let nt_header_definition = SymbolicStructDefinition::new(
            String::from("NtHeader"),
            vec![
                SymbolicFieldDefinition::from_str("Signature:u32").expect("Expected Signature field to parse."),
                SymbolicFieldDefinition::from_str("FileHeader:FileHeader").expect("Expected FileHeader field to parse."),
            ],
        );
        let pe_headers_definition = SymbolicStructDefinition::new(
            String::from("PeHeaders"),
            vec![
                SymbolicFieldDefinition::from_str("DOSHeader:DosHeader @ +0").expect("Expected DOSHeader field to parse."),
                SymbolicFieldDefinition::from_str("NTHeaders:NtHeader @ resolver(nt_offset)").expect("Expected NTHeaders field to parse."),
                SymbolicFieldDefinition::from_str("SectionHeaders:u8[resolver(section_count)] @ resolver(section_offset)")
                    .expect("Expected SectionHeaders field to parse."),
            ],
        );
        let nt_offset_resolver = SymbolicResolverDefinition::new(SymbolicResolverNode::new_relative_symbol_field(
            SymbolicResolverRelativeSymbolPath::from_dot_path("DOSHeader.e_lfanew"),
        ));
        let section_count_resolver = SymbolicResolverDefinition::new(SymbolicResolverNode::new_relative_symbol_field(
            SymbolicResolverRelativeSymbolPath::from_dot_path("NTHeaders.FileHeader.NumberOfSections"),
        ));
        let section_offset_resolver = SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
            SymbolicResolverBinaryOperator::Add,
            SymbolicResolverNode::new_binary(
                SymbolicResolverBinaryOperator::Add,
                SymbolicResolverNode::new_relative_symbol_field(SymbolicResolverRelativeSymbolPath::from_dot_path("DOSHeader.e_lfanew")),
                SymbolicResolverNode::new_literal(24),
            ),
            SymbolicResolverNode::new_relative_symbol_field(SymbolicResolverRelativeSymbolPath::from_dot_path("NTHeaders.FileHeader.SizeOfOptionalHeader")),
        ));

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers_and_relative_symbol_fields(
            &pe_headers_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u16" => Some(2),
                "u32" => Some(4),
                "DosHeader" => Some(64),
                "FileHeader" => Some(20),
                "NtHeader" => Some(24),
                _ => None,
            },
            |_, field_offset, _| match field_offset {
                0x3C => Ok(Some(0x80)),
                0x86 => Ok(Some(3)),
                0x94 => Ok(Some(0xE0)),
                _ => Ok(None),
            },
            |resolver_id| match resolver_id {
                "nt_offset" => Some(nt_offset_resolver.clone()),
                "section_count" => Some(section_count_resolver.clone()),
                "section_offset" => Some(section_offset_resolver.clone()),
                _ => None,
            },
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "DosHeader" => Some(dos_header_definition.clone()),
                "FileHeader" => Some(file_header_definition.clone()),
                "NtHeader" => Some(nt_header_definition.clone()),
                _ => None,
            },
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[1].get_offset_in_bytes(), Some(0x80));
        assert_eq!(resolved_fields[2].get_element_count(), Some(3));
        assert_eq!(resolved_fields[2].get_offset_in_bytes(), Some(0x178));
    }

    #[test]
    fn resolver_reports_relative_symbol_field_cycles() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Cycle"),
            vec![SymbolicFieldDefinition::from_str("value:u32 @ resolver(loop)").expect("Expected value field to parse.")],
        );
        let cyclic_resolver = SymbolicResolverDefinition::new(SymbolicResolverNode::new_relative_symbol_field(
            SymbolicResolverRelativeSymbolPath::from_dot_path("value"),
        ));

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers_and_relative_symbol_fields(
            &symbolic_struct_definition,
            |data_type_ref| (data_type_ref == &DataTypeRef::new("u32")).then_some(4),
            |_, _, _| Ok(Some(7)),
            |resolver_id| (resolver_id == "loop").then_some(cyclic_resolver.clone()),
            |_| None,
            &SymbolicStructResolverOptions::default(),
        );

        assert!(matches!(
            resolved_struct.get_fields()[0].get_status(),
            ResolvedSymbolicFieldStatus::Unresolved { reason } if reason.contains("Resolver cycle detected")
        ));
    }
}
