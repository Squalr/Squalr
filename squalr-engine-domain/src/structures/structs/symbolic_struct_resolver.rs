use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::container_type::ContainerType,
    structs::{
        symbolic_expression::{SymbolicExpression, SymbolicExpressionEvaluationError},
        symbolic_field_definition::{SymbolicFieldCountResolution, SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverEvaluationError},
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
            field_definition,
            next_sequential_offset,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            resolve_resolver_definition,
        );
        let element_count_result = resolve_field_element_count(
            field_definition,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            resolve_resolver_definition,
        );
        let unit_size_in_bytes = resolve_type_size_in_bytes(field_definition.get_data_type_ref());
        let mut field_status = build_field_status(&field_offset_result, &element_count_result, unit_size_in_bytes);
        let displayed_element_count = element_count_result
            .as_ref()
            .ok()
            .and_then(|element_count| *element_count)
            .map(|element_count| element_count.min(options.get_max_dynamic_array_preview_elements()));
        if let (Ok(Some(actual_element_count)), Some(displayed_element_count)) = (&element_count_result, displayed_element_count) {
            if *actual_element_count > displayed_element_count && matches!(field_status, ResolvedSymbolicFieldStatus::Ready) {
                field_status = ResolvedSymbolicFieldStatus::Clamped {
                    actual_element_count: *actual_element_count,
                    displayed_element_count,
                    reason: format!(
                        "Dynamic array count exceeds preview limit of {}.",
                        options.get_max_dynamic_array_preview_elements()
                    ),
                };
            }
        }
        let field_size_in_bytes =
            unit_size_in_bytes.and_then(|unit_size| displayed_element_count.and_then(|element_count| unit_size.checked_mul(element_count)));
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

fn resolve_field_offset<ResolveTypeSize>(
    field_definition: &SymbolicFieldDefinition,
    next_sequential_offset: u64,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
) -> Result<Option<u64>, String>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
{
    match field_definition.get_offset_resolution() {
        SymbolicFieldOffsetResolution::Sequential => Ok(Some(next_sequential_offset)),
        SymbolicFieldOffsetResolution::Expression(offset_expression) => {
            evaluate_u64_expression(offset_expression, scalar_values_by_field_name, resolve_type_size_in_bytes).map(Some)
        }
        SymbolicFieldOffsetResolution::Resolver(resolver_id) => evaluate_u64_resolver(
            resolver_id,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            resolve_resolver_definition,
        )
        .map(Some),
    }
}

fn resolve_field_element_count(
    field_definition: &SymbolicFieldDefinition,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &impl Fn(&DataTypeRef) -> Option<u64>,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
) -> Result<Option<u64>, String> {
    match field_definition.get_count_resolution() {
        SymbolicFieldCountResolution::Expression(count_expression) => {
            evaluate_u64_expression(count_expression, scalar_values_by_field_name, resolve_type_size_in_bytes).map(Some)
        }
        SymbolicFieldCountResolution::Resolver(resolver_id) => evaluate_u64_resolver(
            resolver_id,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            resolve_resolver_definition,
        )
        .map(Some),
        SymbolicFieldCountResolution::Inferred => match field_definition.get_container_type() {
            ContainerType::ArrayFixed(element_count) => Ok(Some(element_count)),
            ContainerType::Array => Ok(None),
            ContainerType::None | ContainerType::Pointer(_) | ContainerType::Pointer32 | ContainerType::Pointer64 => Ok(Some(1)),
        },
    }
}

fn evaluate_u64_resolver<ResolveTypeSize>(
    resolver_id: &str,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
) -> Result<u64, String>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
{
    let resolver_definition = resolve_resolver_definition(resolver_id).ok_or_else(|| format!("Unknown resolver `{}`.", resolver_id))?;
    let value = resolver_definition
        .evaluate(&|field_name| scalar_values_by_field_name.get(field_name).copied(), resolve_type_size_in_bytes)
        .map_err(format_resolver_error)?;

    u64::try_from(value).map_err(|_| format!("Resolver `{}` resolved to negative or too-large value `{}`.", resolver_id, value))
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
    unit_size_in_bytes: Option<u64>,
) -> ResolvedSymbolicFieldStatus {
    if let Err(error) = field_offset_result {
        return ResolvedSymbolicFieldStatus::Unresolved { reason: error.to_string() };
    }

    if let Err(error) = element_count_result {
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
    };
    use crate::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        structs::symbolic_resolver_definition::{SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverNode},
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
        assert_eq!(resolved_fields[1].get_size_in_bytes(), Some(512));
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
}
