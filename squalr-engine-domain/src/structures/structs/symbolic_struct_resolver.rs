use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::container_type::ContainerType,
    memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
    structs::{
        symbolic_field_definition::{SymbolicFieldCountResolution, SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverEvaluationError, SymbolicResolverRelativeSymbolPath},
        symbolic_struct_definition::SymbolicStructDefinition,
    },
};
use std::cell::RefCell;
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
    variant_activation: ResolvedSymbolicFieldVariantActivation,
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
        variant_activation: ResolvedSymbolicFieldVariantActivation,
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
            variant_activation,
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

    pub fn get_variant_activation(&self) -> &ResolvedSymbolicFieldVariantActivation {
        &self.variant_activation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedSymbolicFieldVariantActivation {
    NotApplicable,
    Unspecified,
    Active,
    Inactive,
    Ambiguous,
    Unresolved { reason: String },
}

impl ResolvedSymbolicFieldVariantActivation {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn is_inactive(&self) -> bool {
        matches!(self, Self::Inactive)
    }

    pub fn is_specified(&self) -> bool {
        !matches!(self, Self::NotApplicable | Self::Unspecified)
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
    evaluate_union_variant_activations: bool,
}

impl SymbolicStructResolverOptions {
    pub fn new(max_dynamic_array_preview_elements: u64) -> Self {
        Self {
            max_dynamic_array_preview_elements,
            evaluate_union_variant_activations: true,
        }
    }

    pub fn get_max_dynamic_array_preview_elements(&self) -> u64 {
        self.max_dynamic_array_preview_elements
    }

    pub fn get_evaluate_union_variant_activations(&self) -> bool {
        self.evaluate_union_variant_activations
    }

    pub fn with_evaluate_union_variant_activations(
        mut self,
        evaluate_union_variant_activations: bool,
    ) -> Self {
        self.evaluate_union_variant_activations = evaluate_union_variant_activations;
        self
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
    resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields_and_pointer_chains(
        symbolic_struct_definition,
        resolve_type_size_in_bytes,
        read_scalar_field,
        resolve_resolver_definition,
        resolve_struct_definition,
        resolve_global_symbol_field,
        |pointer_chain| Err(SymbolicResolverEvaluationError::UnknownGlobalPointerChain(pointer_chain.to_string())),
        options,
    )
}

pub fn resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields_and_pointer_chains<
    ResolveTypeSize,
    ReadScalarField,
    ResolveResolverDefinition,
    ResolveStructDefinition,
    ResolveGlobalSymbolField,
    ResolveGlobalPointerChain,
>(
    symbolic_struct_definition: &SymbolicStructDefinition,
    resolve_type_size_in_bytes: ResolveTypeSize,
    read_scalar_field: ReadScalarField,
    resolve_resolver_definition: ResolveResolverDefinition,
    resolve_struct_definition: ResolveStructDefinition,
    resolve_global_symbol_field: ResolveGlobalSymbolField,
    resolve_global_pointer_chain: ResolveGlobalPointerChain,
    options: &SymbolicStructResolverOptions,
) -> ResolvedSymbolicStruct
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
    ResolveResolverDefinition: Fn(&str) -> Option<SymbolicResolverDefinition>,
    ResolveStructDefinition: Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    ResolveGlobalSymbolField: Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
{
    resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields_and_relative_pointer_chains(
        symbolic_struct_definition,
        resolve_type_size_in_bytes,
        read_scalar_field,
        resolve_resolver_definition,
        resolve_struct_definition,
        resolve_global_symbol_field,
        |pointer_chain| Err(SymbolicResolverEvaluationError::UnknownRelativePointerChain(pointer_chain.to_string())),
        resolve_global_pointer_chain,
        options,
    )
}

pub fn resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields_and_relative_pointer_chains<
    ResolveTypeSize,
    ReadScalarField,
    ResolveResolverDefinition,
    ResolveStructDefinition,
    ResolveGlobalSymbolField,
    ResolveRelativePointerChain,
    ResolveGlobalPointerChain,
>(
    symbolic_struct_definition: &SymbolicStructDefinition,
    resolve_type_size_in_bytes: ResolveTypeSize,
    read_scalar_field: ReadScalarField,
    resolve_resolver_definition: ResolveResolverDefinition,
    resolve_struct_definition: ResolveStructDefinition,
    resolve_global_symbol_field: ResolveGlobalSymbolField,
    resolve_relative_pointer_chain: ResolveRelativePointerChain,
    resolve_global_pointer_chain: ResolveGlobalPointerChain,
    options: &SymbolicStructResolverOptions,
) -> ResolvedSymbolicStruct
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
    ResolveResolverDefinition: Fn(&str) -> Option<SymbolicResolverDefinition>,
    ResolveStructDefinition: Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    ResolveGlobalSymbolField: Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    ResolveRelativePointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
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
            &resolve_relative_pointer_chain,
            &resolve_global_pointer_chain,
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
    resolve_relative_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_global_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
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
    let variant_activations = if options.get_evaluate_union_variant_activations() {
        resolve_union_variant_activations(
            symbolic_struct_definition,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
        )
    } else {
        inactive_union_variant_activations(symbolic_struct_definition)
    };

    for (field_definition, variant_activation) in symbolic_struct_definition
        .get_fields()
        .iter()
        .zip(variant_activations)
    {
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
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
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
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
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
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
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
                .and_then(|element_count| resolve_field_size_in_bytes(field_definition, unit_size, element_count))
        });
        let field_offset = field_offset_result.as_ref().ok().copied().flatten();

        if let Some(field_size) = field_size_in_bytes {
            if let Some(field_offset) = field_offset {
                next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size));
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
            variant_activation,
            field_status,
        ));
    }

    SymbolicStructResolvePass {
        resolved_fields,
        did_update_scalar_value,
    }
}

fn resolve_union_variant_activations<ResolveTypeSize, ReadScalarField>(
    root_struct_definition: &SymbolicStructDefinition,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
    resolve_struct_definition: &impl Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    resolve_global_symbol_field: &impl Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_relative_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_global_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
) -> Vec<ResolvedSymbolicFieldVariantActivation>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    if !root_struct_definition.get_layout_kind().is_union() {
        return root_struct_definition
            .get_fields()
            .iter()
            .map(|_| ResolvedSymbolicFieldVariantActivation::NotApplicable)
            .collect();
    }

    let activation_results = root_struct_definition
        .get_fields()
        .iter()
        .map(|field_definition| {
            let Some(active_when_resolver) = field_definition.get_active_when_resolver() else {
                return None;
            };

            Some(
                evaluate_i128_resolver(
                    root_struct_definition,
                    active_when_resolver.get_resolver_id(),
                    scalar_values_by_field_name,
                    resolve_type_size_in_bytes,
                    read_scalar_field,
                    resolve_resolver_definition,
                    resolve_struct_definition,
                    resolve_global_symbol_field,
                    resolve_relative_pointer_chain,
                    resolve_global_pointer_chain,
                    &mut Vec::new(),
                )
                .map(|value| value != 0),
            )
        })
        .collect::<Vec<_>>();

    if activation_results.iter().all(Option::is_none) {
        return activation_results
            .iter()
            .map(|_| ResolvedSymbolicFieldVariantActivation::Unspecified)
            .collect();
    }

    let active_count = activation_results
        .iter()
        .filter(|activation_result| matches!(activation_result, Some(Ok(true))))
        .count();

    activation_results
        .into_iter()
        .map(|activation_result| match activation_result {
            None => ResolvedSymbolicFieldVariantActivation::Inactive,
            Some(Ok(true)) if active_count == 1 => ResolvedSymbolicFieldVariantActivation::Active,
            Some(Ok(true)) => ResolvedSymbolicFieldVariantActivation::Ambiguous,
            Some(Ok(false)) => ResolvedSymbolicFieldVariantActivation::Inactive,
            Some(Err(reason)) => ResolvedSymbolicFieldVariantActivation::Unresolved { reason },
        })
        .collect()
}

fn inactive_union_variant_activations(root_struct_definition: &SymbolicStructDefinition) -> Vec<ResolvedSymbolicFieldVariantActivation> {
    let inactive_variant_activation = if root_struct_definition.get_layout_kind().is_union() {
        ResolvedSymbolicFieldVariantActivation::Unspecified
    } else {
        ResolvedSymbolicFieldVariantActivation::NotApplicable
    };

    root_struct_definition
        .get_fields()
        .iter()
        .map(|_| inactive_variant_activation.clone())
        .collect()
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
    resolve_relative_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_global_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<Option<u64>, String>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    match field_definition.get_offset_resolution() {
        SymbolicFieldOffsetResolution::Sequential if root_struct_definition.get_layout_kind().is_union() => Ok(Some(0)),
        SymbolicFieldOffsetResolution::Sequential => Ok(Some(next_sequential_offset)),
        SymbolicFieldOffsetResolution::Static(offset_in_bytes) => Ok(Some(*offset_in_bytes)),
        SymbolicFieldOffsetResolution::Resolver(resolver_id) => evaluate_u64_resolver(
            root_struct_definition,
            resolver_id,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
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
    resolve_relative_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_global_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<Option<u64>, String>
where
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    match field_definition.get_count_resolution() {
        SymbolicFieldCountResolution::Resolver(resolver_id) => evaluate_u64_resolver(
            root_struct_definition,
            resolver_id,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
            resolver_stack,
        )
        .map(Some),
        SymbolicFieldCountResolution::Inferred => match field_definition.get_container_type() {
            ContainerType::ArrayFixed(element_count) | ContainerType::PointerArrayFixed(_, element_count) => Ok(Some(element_count)),
            ContainerType::Array | ContainerType::PointerArray(_) => Ok(None),
            ContainerType::None | ContainerType::Pointer(_) => Ok(Some(1)),
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
    resolve_relative_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_global_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<Option<u64>, String>
where
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    match field_definition.get_display_count_resolution() {
        SymbolicFieldCountResolution::Inferred => element_count_result.clone(),
        SymbolicFieldCountResolution::Resolver(resolver_id) => evaluate_u64_resolver(
            root_struct_definition,
            resolver_id,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
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
    resolve_relative_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_global_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<u64, String>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    let value = evaluate_i128_resolver(
        root_struct_definition,
        resolver_id,
        scalar_values_by_field_name,
        resolve_type_size_in_bytes,
        read_scalar_field,
        resolve_resolver_definition,
        resolve_struct_definition,
        resolve_global_symbol_field,
        resolve_relative_pointer_chain,
        resolve_global_pointer_chain,
        resolver_stack,
    )?;

    u64::try_from(value).map_err(|_| format!("Resolver `{}` resolved to negative or too-large value `{}`.", resolver_id, value))
}

fn evaluate_i128_resolver<ResolveTypeSize, ReadScalarField>(
    root_struct_definition: &SymbolicStructDefinition,
    resolver_id: &str,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
    resolve_struct_definition: &impl Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    resolve_global_symbol_field: &impl Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_relative_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_global_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<i128, String>
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
        let resolver_stack_cell = RefCell::new(resolver_stack);
        let value = {
            let mut resolve_relative_symbol_field = |symbol_path: &SymbolicResolverRelativeSymbolPath| {
                let mut resolver_stack = resolver_stack_cell.borrow_mut();

                resolve_relative_symbol_path_value(
                    symbol_path,
                    root_struct_definition,
                    scalar_values_by_field_name,
                    resolve_type_size_in_bytes,
                    read_scalar_field,
                    resolve_resolver_definition,
                    resolve_struct_definition,
                    resolve_global_symbol_field,
                    resolve_relative_pointer_chain,
                    resolve_global_pointer_chain,
                    &mut resolver_stack,
                )
            };
            let mut resolve_relative_pointer_chain_node = |pointer_chain: &SymbolicPointerChain| {
                let mut resolver_stack = resolver_stack_cell.borrow_mut();

                resolve_relative_pointer_chain_value(
                    pointer_chain,
                    root_struct_definition,
                    scalar_values_by_field_name,
                    resolve_type_size_in_bytes,
                    read_scalar_field,
                    resolve_resolver_definition,
                    resolve_struct_definition,
                    resolve_global_symbol_field,
                    resolve_relative_pointer_chain,
                    resolve_global_pointer_chain,
                    &mut resolver_stack,
                )
            };

            resolver_definition
                .evaluate_with_symbol_fields_and_pointer_chains(
                    &|field_name| scalar_values_by_field_name.get(field_name).copied(),
                    resolve_type_size_in_bytes,
                    &mut resolve_relative_symbol_field,
                    resolve_global_symbol_field,
                    &mut resolve_relative_pointer_chain_node,
                    resolve_global_pointer_chain,
                )
                .map_err(format_resolver_error)
        };

        resolver_stack_cell.into_inner().pop();

        value
    };
    value
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
    resolve_relative_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_global_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
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

    for (symbol_path_link_index, symbol_path_link) in symbol_path.get_links().iter().enumerate() {
        let is_terminal_link = symbol_path_link_index + 1 == symbol_path.get_links().len();

        let SymbolicPointerChainLink::Symbol(field_name) = symbol_path_link else {
            let Some(resolved_base_offset) = apply_signed_offset(current_base_offset, symbol_path_link.as_offset().unwrap_or_default()) else {
                return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()));
            };

            if is_terminal_link {
                return Err(SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(format!(
                    "{} ends at an offset instead of a scalar field.",
                    symbol_path
                )));
            }

            current_base_offset = resolved_base_offset;
            continue;
        };

        let (field_definition, field_offset) = resolve_named_field_offset_in_struct(
            symbol_path,
            field_name,
            &current_struct_definition,
            scalar_values_by_field_name,
            resolve_type_size_in_bytes,
            read_scalar_field,
            resolve_resolver_definition,
            resolve_struct_definition,
            resolve_global_symbol_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
            resolver_stack,
        )?;
        let resolved_field_offset = current_base_offset.saturating_add(field_offset);

        if is_terminal_link {
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

fn apply_signed_offset(
    base_offset: u64,
    offset_in_bytes: i64,
) -> Option<u64> {
    SymbolicPointerChain::apply_pointer_offset(base_offset, offset_in_bytes)
}

fn resolve_relative_pointer_chain_value<ResolveTypeSize, ReadScalarField>(
    pointer_chain: &SymbolicPointerChain,
    root_struct_definition: &SymbolicStructDefinition,
    scalar_values_by_field_name: &BTreeMap<String, i128>,
    resolve_type_size_in_bytes: &ResolveTypeSize,
    read_scalar_field: &ReadScalarField,
    resolve_resolver_definition: &impl Fn(&str) -> Option<SymbolicResolverDefinition>,
    resolve_struct_definition: &impl Fn(&DataTypeRef) -> Option<SymbolicStructDefinition>,
    resolve_global_symbol_field: &impl Fn(&str, &SymbolicResolverRelativeSymbolPath) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_relative_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_global_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolver_stack: &mut Vec<String>,
) -> Result<i128, SymbolicResolverEvaluationError>
where
    ResolveTypeSize: Fn(&DataTypeRef) -> Option<u64>,
    ReadScalarField: Fn(&SymbolicFieldDefinition, u64, u64) -> Result<Option<i128>, String>,
{
    let mut resolved_links = Vec::with_capacity(pointer_chain.get_links().len());

    for pointer_chain_link in pointer_chain.get_links() {
        match pointer_chain_link {
            SymbolicPointerChainLink::Offset(offset_in_bytes) => resolved_links.push(SymbolicPointerChainLink::Offset(*offset_in_bytes)),
            SymbolicPointerChainLink::Symbol(field_name) => {
                let (_, field_offset) = resolve_named_field_offset_in_struct(
                    &SymbolicResolverRelativeSymbolPath::from_dot_path(field_name),
                    field_name,
                    root_struct_definition,
                    scalar_values_by_field_name,
                    resolve_type_size_in_bytes,
                    read_scalar_field,
                    resolve_resolver_definition,
                    resolve_struct_definition,
                    resolve_global_symbol_field,
                    resolve_relative_pointer_chain,
                    resolve_global_pointer_chain,
                    resolver_stack,
                )?;
                let field_offset =
                    i64::try_from(field_offset).map_err(|_| SymbolicResolverEvaluationError::UnknownRelativePointerChain(pointer_chain.to_string()))?;

                resolved_links.push(SymbolicPointerChainLink::Offset(field_offset));
            }
        }
    }

    let resolved_pointer_chain = SymbolicPointerChain::new_absolute(resolved_links, pointer_chain.get_pointer_size());

    resolve_relative_pointer_chain(&resolved_pointer_chain)
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
    resolve_relative_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
    resolve_global_pointer_chain: &impl Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError>,
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
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
            resolver_stack,
        )
        .map_err(|error| SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(format!("{}: {}", symbol_path, error)))?
        .ok_or_else(|| SymbolicResolverEvaluationError::UnknownRelativeSymbolPath(symbol_path.to_string()))?;

        if field_definition.get_field_name() == symbol_path_segment {
            return Ok((field_definition, field_offset));
        }

        let field_size_in_bytes = resolve_field_static_size_in_bytes(field_definition, resolve_type_size_in_bytes);

        if let Some(field_size_in_bytes) = field_size_in_bytes {
            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
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
        ContainerType::ArrayFixed(element_count) | ContainerType::PointerArrayFixed(_, element_count) => {
            resolve_field_size_in_bytes(field_definition, unit_size_in_bytes, element_count)
        }
        ContainerType::None | ContainerType::Pointer(_) => resolve_field_size_in_bytes(field_definition, unit_size_in_bytes, 1),
        ContainerType::Array | ContainerType::PointerArray(_) => None,
    }
}

fn resolve_field_size_in_bytes(
    field_definition: &SymbolicFieldDefinition,
    unit_size_in_bytes: u64,
    element_count: u64,
) -> Option<u64> {
    match field_definition.get_container_type() {
        ContainerType::Pointer(pointer_size) => Some(pointer_size.get_size_in_bytes()),
        ContainerType::PointerArray(pointer_size) | ContainerType::PointerArrayFixed(pointer_size, _) => {
            pointer_size.get_size_in_bytes().checked_mul(element_count)
        }
        ContainerType::None | ContainerType::Array | ContainerType::ArrayFixed(_) => unit_size_in_bytes.checked_mul(element_count),
    }
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

fn format_resolver_error(error: SymbolicResolverEvaluationError) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        ResolvedSymbolicFieldStatus, ResolvedSymbolicFieldVariantActivation, SymbolicStructResolverOptions, resolve_symbolic_struct_definition,
        resolve_symbolic_struct_definition_with_resolvers, resolve_symbolic_struct_definition_with_resolvers_and_relative_symbol_fields,
        resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields_and_relative_pointer_chains,
    };
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::pointer_scan_pointer_size::PointerScanPointerSize,
        memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
        structs::symbolic_resolver_definition::{
            SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverNode, SymbolicResolverRelativeSymbolPath,
        },
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };
    use std::str::FromStr;

    fn local_field_resolver(field_name: &str) -> SymbolicResolverDefinition {
        SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(field_name.to_string()))
    }

    fn binary_resolver(
        operator: SymbolicResolverBinaryOperator,
        left_node: SymbolicResolverNode,
        right_node: SymbolicResolverNode,
    ) -> SymbolicResolverDefinition {
        SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(operator, left_node, right_node))
    }

    #[test]
    fn resolver_uses_previous_scalar_fields_for_vec_like_dynamic_layout() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![
                SymbolicFieldDefinition::from_str("count:u32 @ +0").expect("Expected count field to parse."),
                SymbolicFieldDefinition::from_str("capacity:u32 @ +4").expect("Expected capacity field to parse."),
                SymbolicFieldDefinition::from_str("elements:Element[resolver(item.count)] @ +8").expect("Expected elements field to parse."),
                SymbolicFieldDefinition::from_str("unfilled:Element[resolver(item.unfilled_count)] @ resolver(item.unfilled_offset)")
                    .expect("Expected unfilled field to parse."),
            ],
        );
        let count_resolver = local_field_resolver("count");
        let unfilled_count_resolver = binary_resolver(
            SymbolicResolverBinaryOperator::Subtract,
            SymbolicResolverNode::new_local_field(String::from("capacity")),
            SymbolicResolverNode::new_local_field(String::from("count")),
        );
        let unfilled_offset_resolver = binary_resolver(
            SymbolicResolverBinaryOperator::Add,
            SymbolicResolverNode::new_literal(8),
            SymbolicResolverNode::new_binary(
                SymbolicResolverBinaryOperator::Multiply,
                SymbolicResolverNode::new_local_field(String::from("count")),
                SymbolicResolverNode::new_type_size(DataTypeRef::new("Element")),
            ),
        );

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers(
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
            |resolver_id| match resolver_id {
                "item.count" => Some(count_resolver.clone()),
                "item.unfilled_count" => Some(unfilled_count_resolver.clone()),
                "item.unfilled_offset" => Some(unfilled_offset_resolver.clone()),
                _ => None,
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
                SymbolicFieldDefinition::from_str("elements:Element[resolver(item.count)] @ +8").expect("Expected elements field to parse."),
                SymbolicFieldDefinition::from_str("count:u32 @ +4").expect("Expected count field to parse."),
            ],
        );
        let count_resolver = local_field_resolver("count");

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
            |resolver_id| (resolver_id == "item.count").then_some(count_resolver.clone()),
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
    fn resolver_reports_resolver_diagnostics_on_affected_field() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![
                SymbolicFieldDefinition::from_str("count:u32 @ +0").expect("Expected count field to parse."),
                SymbolicFieldDefinition::from_str("elements:Element[resolver(item.bad_count)] @ +4").expect("Expected elements field to parse."),
            ],
        );
        let bad_count_resolver = binary_resolver(
            SymbolicResolverBinaryOperator::Divide,
            SymbolicResolverNode::new_local_field(String::from("count")),
            SymbolicResolverNode::new_local_field(String::from("divisor")),
        );

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers(
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
            |resolver_id| (resolver_id == "item.bad_count").then_some(bad_count_resolver.clone()),
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert!(matches!(
            resolved_fields[1].get_status(),
            ResolvedSymbolicFieldStatus::Unresolved { reason } if reason.contains("Unknown local field `divisor`")
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
                SymbolicFieldDefinition::from_str("elements:Element[resolver(item.count)]").expect("Expected elements field to parse."),
            ],
        );
        let count_resolver = local_field_resolver("count");

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers(
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
            |resolver_id| (resolver_id == "item.count").then_some(count_resolver.clone()),
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
                SymbolicFieldDefinition::from_str("entities:u64[1024] display resolver(entity.count) @ +8").expect("Expected entities field to parse."),
                SymbolicFieldDefinition::from_str("tail:u32").expect("Expected tail field to parse."),
            ],
        );
        let count_resolver = local_field_resolver("count");

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers(
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
            |resolver_id| (resolver_id == "entity.count").then_some(count_resolver.clone()),
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[1].get_element_count(), Some(1024));
        assert_eq!(resolved_fields[1].get_displayed_element_count(), Some(3));
        assert_eq!(resolved_fields[1].get_size_in_bytes(), Some(8192));
        assert_eq!(resolved_fields[2].get_offset_in_bytes(), Some(8200));
    }

    #[test]
    fn resolver_evaluates_relative_pointer_chains_with_symbolic_field_links() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("EntityManager"),
            vec![
                SymbolicFieldDefinition::from_str("entity_list:u64 @ +0x10").expect("Expected entity_list field to parse."),
                SymbolicFieldDefinition::from_str("active_count:u32[resolver(entity.count_via_pointer)] @ +0").expect("Expected active_count field to parse."),
            ],
        );
        let relative_pointer_chain = SymbolicPointerChain::new_absolute(
            vec![
                SymbolicPointerChainLink::Symbol(String::from("entity_list")),
                SymbolicPointerChainLink::Offset(0x20),
            ],
            PointerScanPointerSize::Pointer64,
        );
        let count_resolver = SymbolicResolverDefinition::new(SymbolicResolverNode::new_relative_pointer_chain(relative_pointer_chain));

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields_and_relative_pointer_chains(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "u64" => Some(8),
                _ => None,
            },
            |_, _, _| Ok(None),
            |resolver_id| (resolver_id == "entity.count_via_pointer").then_some(count_resolver.clone()),
            |_| None,
            |_, _| panic!("Expected no global symbol field lookup."),
            |resolved_pointer_chain| {
                assert_eq!(
                    resolved_pointer_chain.get_links(),
                    &[
                        SymbolicPointerChainLink::Offset(0x10),
                        SymbolicPointerChainLink::Offset(0x20)
                    ]
                );
                Ok(3)
            },
            |_| panic!("Expected no global pointer chain lookup."),
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[1].get_element_count(), Some(3));
        assert_eq!(resolved_fields[1].get_status(), &ResolvedSymbolicFieldStatus::Ready);
    }

    #[test]
    fn resolver_sizes_fixed_pointer_arrays_by_pointer_storage() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("EntityList"),
            vec![
                SymbolicFieldDefinition::from_str("count:u32").expect("Expected count field to parse."),
                SymbolicFieldDefinition::from_str("entities:Entity*(u64)[1024] display resolver(entity.count) @ +8")
                    .expect("Expected entities field to parse."),
                SymbolicFieldDefinition::from_str("tail:u32").expect("Expected tail field to parse."),
            ],
        );
        let count_resolver = local_field_resolver("count");

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "Entity" => Some(128),
                _ => None,
            },
            |field_definition, _, _| match field_definition.get_field_name() {
                "count" => Ok(Some(3)),
                _ => Ok(None),
            },
            |resolver_id| (resolver_id == "entity.count").then_some(count_resolver.clone()),
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[1].get_element_count(), Some(1024));
        assert_eq!(resolved_fields[1].get_displayed_element_count(), Some(3));
        assert_eq!(resolved_fields[1].get_size_in_bytes(), Some(8192));
        assert_eq!(resolved_fields[2].get_offset_in_bytes(), Some(8200));
    }

    #[test]
    fn resolver_keeps_sequential_offset_after_overlapping_explicit_fields() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("UnionLike"),
            vec![
                SymbolicFieldDefinition::from_str("tag:u32 @ +0").expect("Expected tag field to parse."),
                SymbolicFieldDefinition::from_str("wide:u64 @ +4").expect("Expected wide field to parse."),
                SymbolicFieldDefinition::from_str("narrow:u32 @ +4").expect("Expected narrow field to parse."),
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
            |_, _, _| Ok(None),
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[0].get_offset_in_bytes(), Some(0));
        assert_eq!(resolved_fields[1].get_offset_in_bytes(), Some(4));
        assert_eq!(resolved_fields[2].get_offset_in_bytes(), Some(4));
        assert_eq!(resolved_fields[3].get_offset_in_bytes(), Some(12));
        assert_eq!(resolved_fields[3].get_size_in_bytes(), Some(4));
    }

    #[test]
    fn resolver_defaults_union_fields_to_shared_offset() {
        let symbolic_struct_definition = SymbolicStructDefinition::new_union(
            String::from("VariantPayload"),
            vec![
                SymbolicFieldDefinition::from_str("as_u64:u64").expect("Expected u64 field to parse."),
                SymbolicFieldDefinition::from_str("as_u32:u32").expect("Expected u32 field to parse."),
                SymbolicFieldDefinition::from_str("raw:u8[16]").expect("Expected raw field to parse."),
            ],
        );

        let resolved_struct = resolve_symbolic_struct_definition(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u32" => Some(4),
                "u64" => Some(8),
                _ => None,
            },
            |_, _, _| Ok(None),
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[0].get_offset_in_bytes(), Some(0));
        assert_eq!(resolved_fields[1].get_offset_in_bytes(), Some(0));
        assert_eq!(resolved_fields[2].get_offset_in_bytes(), Some(0));
        assert_eq!(resolved_fields[2].get_size_in_bytes(), Some(16));
    }

    #[test]
    fn resolver_marks_single_truthy_union_variant_active() {
        let symbolic_struct_definition = SymbolicStructDefinition::new_union(
            String::from("State"),
            vec![
                SymbolicFieldDefinition::from_str("alive:Alive active resolver(is_alive)").expect("Expected alive variant to parse."),
                SymbolicFieldDefinition::from_str("dead:Dead active resolver(is_dead)").expect("Expected dead variant to parse."),
            ],
        );
        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers(
            &symbolic_struct_definition,
            |_| Some(4),
            |_, _, _| Ok(None),
            |resolver_id| match resolver_id {
                "is_alive" => Some(SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(1))),
                "is_dead" => Some(SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(0))),
                _ => None,
            },
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert!(resolved_fields[0].get_variant_activation().is_active());
        assert!(resolved_fields[1].get_variant_activation().is_inactive());
    }

    #[test]
    fn resolver_marks_multiple_truthy_union_variants_ambiguous() {
        let symbolic_struct_definition = SymbolicStructDefinition::new_union(
            String::from("State"),
            vec![
                SymbolicFieldDefinition::from_str("alive:Alive active resolver(always)").expect("Expected alive variant to parse."),
                SymbolicFieldDefinition::from_str("dead:Dead active resolver(always)").expect("Expected dead variant to parse."),
            ],
        );
        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers(
            &symbolic_struct_definition,
            |_| Some(4),
            |_, _, _| Ok(None),
            |resolver_id| (resolver_id == "always").then(|| SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(1))),
            &SymbolicStructResolverOptions::default(),
        );

        assert_eq!(
            resolved_struct.get_fields()[0].get_variant_activation(),
            &ResolvedSymbolicFieldVariantActivation::Ambiguous
        );
        assert_eq!(
            resolved_struct.get_fields()[1].get_variant_activation(),
            &ResolvedSymbolicFieldVariantActivation::Ambiguous
        );
    }

    #[test]
    fn relative_symbol_fields_keep_sequential_offset_after_overlapping_explicit_fields() {
        let union_definition = SymbolicStructDefinition::new(
            String::from("UnionLike"),
            vec![
                SymbolicFieldDefinition::from_str("tag:u32 @ +0").expect("Expected tag field to parse."),
                SymbolicFieldDefinition::from_str("wide:u64 @ +4").expect("Expected wide field to parse."),
                SymbolicFieldDefinition::from_str("narrow:u32 @ +4").expect("Expected narrow field to parse."),
                SymbolicFieldDefinition::from_str("tail:u32").expect("Expected tail field to parse."),
            ],
        );
        let wrapper_definition = SymbolicStructDefinition::new(
            String::from("Wrapper"),
            vec![
                SymbolicFieldDefinition::from_str("value:UnionLike @ +0").expect("Expected value field to parse."),
                SymbolicFieldDefinition::from_str("elements:u8[resolver(tail_count)] @ +16").expect("Expected elements field to parse."),
            ],
        );
        let tail_count_resolver = SymbolicResolverDefinition::new(SymbolicResolverNode::new_relative_symbol_field(
            SymbolicResolverRelativeSymbolPath::from_dot_path("value.tail"),
        ));

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers_and_relative_symbol_fields(
            &wrapper_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u32" => Some(4),
                "u64" => Some(8),
                "UnionLike" => Some(16),
                _ => None,
            },
            |field_definition, field_offset, _| match (field_definition.get_field_name(), field_offset) {
                ("tail", 12) => Ok(Some(5)),
                _ => Ok(None),
            },
            |resolver_id| (resolver_id == "tail_count").then_some(tail_count_resolver.clone()),
            |data_type_ref| (data_type_ref.get_data_type_id() == "UnionLike").then_some(union_definition.clone()),
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[1].get_element_count(), Some(5));
        assert_eq!(resolved_fields[1].get_status(), &ResolvedSymbolicFieldStatus::Ready);
    }

    #[test]
    fn relative_symbol_fields_apply_byte_offsets_to_path_segments() {
        let item_definition = SymbolicStructDefinition::new(
            String::from("Item"),
            vec![SymbolicFieldDefinition::from_str("value:u32").expect("Expected value field to parse.")],
        );
        let list_definition = SymbolicStructDefinition::new(
            String::from("List"),
            vec![
                SymbolicFieldDefinition::from_str("padding:u8[4]").expect("Expected padding field to parse."),
                SymbolicFieldDefinition::from_str("items:Item[2]").expect("Expected items field to parse."),
                SymbolicFieldDefinition::from_str("elements:u8[resolver(second_value)]").expect("Expected elements field to parse."),
            ],
        );
        let second_value_resolver = SymbolicResolverDefinition::new(SymbolicResolverNode::new_relative_symbol_field(
            SymbolicResolverRelativeSymbolPath::from_dot_path("items+4.value"),
        ));

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers_and_relative_symbol_fields(
            &list_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u32" => Some(4),
                "Item" => Some(4),
                _ => None,
            },
            |field_definition, field_offset, _| match (field_definition.get_field_name(), field_offset) {
                ("value", 8) => Ok(Some(11)),
                _ => Ok(None),
            },
            |resolver_id| (resolver_id == "second_value").then_some(second_value_resolver.clone()),
            |data_type_ref| (data_type_ref.get_data_type_id() == "Item").then_some(item_definition.clone()),
            &SymbolicStructResolverOptions::default(),
        );
        let resolved_fields = resolved_struct.get_fields();

        assert_eq!(resolved_fields[2].get_element_count(), Some(11));
        assert_eq!(resolved_fields[2].get_status(), &ResolvedSymbolicFieldStatus::Ready);
    }

    #[test]
    fn resolver_supports_sizeof_in_offset_resolvers() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![SymbolicFieldDefinition::from_str("tail:u32 @ resolver(item.tail_offset)").expect("Expected tail field to parse.")],
        );
        let tail_offset_resolver = binary_resolver(
            SymbolicResolverBinaryOperator::Multiply,
            SymbolicResolverNode::new_type_size(DataTypeRef::new("Element")),
            SymbolicResolverNode::new_literal(3),
        );

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers(
            &symbolic_struct_definition,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "Element" => Some(32),
                _ => None,
            },
            |_, _, _| Ok(None),
            |resolver_id| (resolver_id == "item.tail_offset").then_some(tail_offset_resolver.clone()),
            &SymbolicStructResolverOptions::default(),
        );

        assert_eq!(resolved_struct.get_fields()[0].get_offset_in_bytes(), Some(96));
    }

    #[test]
    fn resolver_supports_sizeof_in_count_resolvers() {
        let symbolic_struct_definition = SymbolicStructDefinition::new(
            String::from("Items"),
            vec![
                SymbolicFieldDefinition::from_str("count:u32").expect("Expected count field to parse."),
                SymbolicFieldDefinition::from_str("capacity:u32").expect("Expected capacity field to parse."),
                SymbolicFieldDefinition::from_str("unfilled:u8[resolver(item.unfilled_count)]").expect("Expected unfilled field to parse."),
            ],
        );
        let unfilled_count_resolver = binary_resolver(
            SymbolicResolverBinaryOperator::Multiply,
            SymbolicResolverNode::new_binary(
                SymbolicResolverBinaryOperator::Subtract,
                SymbolicResolverNode::new_local_field(String::from("capacity")),
                SymbolicResolverNode::new_local_field(String::from("count")),
            ),
            SymbolicResolverNode::new_type_size(DataTypeRef::new("Element")),
        );

        let resolved_struct = resolve_symbolic_struct_definition_with_resolvers(
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
            |resolver_id| (resolver_id == "item.unfilled_count").then_some(unfilled_count_resolver.clone()),
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
