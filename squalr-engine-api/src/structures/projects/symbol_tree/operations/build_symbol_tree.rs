use crate::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::container_type::ContainerType,
    memory::symbolic_pointer_chain::SymbolicPointerChain,
    projects::{
        project_symbol_catalog::ProjectSymbolCatalog,
        project_symbol_claim::ProjectSymbolClaim,
        project_symbol_locator::ProjectSymbolLocator,
        project_symbol_module_field::ProjectSymbolModuleField,
        symbol_tree::symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind},
    },
    structs::{
        symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_global_symbol_resolver::{
            SymbolicGlobalSymbolResolverSession, SymbolicGlobalSymbolRoot, SymbolicGlobalSymbolRootType, resolve_global_symbol_field_value,
        },
        symbolic_resolver_definition::SymbolicResolverEvaluationError,
        symbolic_struct_definition::SymbolicStructDefinition,
        symbolic_struct_resolver::{
            ResolvedSymbolicFieldVariantActivation, SymbolicStructResolverOptions,
            resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields_and_relative_pointer_chains,
        },
    },
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::str::FromStr;

const SYMBOL_TREE_ORIGIN_METADATA_KEY: &str = "symbol_tree.origin";
const SYMBOL_TREE_MODULE_FIELD_ORIGIN: &str = "module_field";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPointerTarget {
    target_locator: ProjectSymbolLocator,
    evaluated_pointer_path: String,
}

impl ResolvedPointerTarget {
    pub fn new(
        target_locator: ProjectSymbolLocator,
        evaluated_pointer_path: String,
    ) -> Self {
        Self {
            target_locator,
            evaluated_pointer_path,
        }
    }

    pub fn get_target_locator(&self) -> &ProjectSymbolLocator {
        &self.target_locator
    }

    pub fn get_evaluated_pointer_path(&self) -> &str {
        &self.evaluated_pointer_path
    }
}

pub fn build_symbol_tree_nodes<ResolvePrimitiveSize>(
    project_symbol_catalog: &ProjectSymbolCatalog,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
) -> Vec<SymbolTreeNode>
where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
{
    build_symbol_tree_nodes_with_scalar_reader(
        project_symbol_catalog,
        expanded_tree_node_keys,
        resolved_pointer_targets_by_node_key,
        resolve_primitive_size_in_bytes,
        |_, _, _| Ok(None),
    )
}

pub fn build_symbol_tree_nodes_with_scalar_reader<ResolvePrimitiveSize, ReadScalarField>(
    project_symbol_catalog: &ProjectSymbolCatalog,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    read_scalar_field: ReadScalarField,
) -> Vec<SymbolTreeNode>
where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
    ReadScalarField: Fn(&ProjectSymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String> + Copy,
{
    build_symbol_tree_nodes_with_scalar_reader_and_pointer_chains(
        project_symbol_catalog,
        expanded_tree_node_keys,
        resolved_pointer_targets_by_node_key,
        resolve_primitive_size_in_bytes,
        read_scalar_field,
        |_, pointer_chain| Err(SymbolicResolverEvaluationError::UnknownRelativePointerChain(pointer_chain.to_string())),
        |pointer_chain| Err(SymbolicResolverEvaluationError::UnknownGlobalPointerChain(pointer_chain.to_string())),
    )
}

pub fn build_symbol_tree_nodes_with_scalar_reader_and_pointer_chains<
    ResolvePrimitiveSize,
    ReadScalarField,
    ResolveRelativePointerChain,
    ResolveGlobalPointerChain,
>(
    project_symbol_catalog: &ProjectSymbolCatalog,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    read_scalar_field: ReadScalarField,
    resolve_relative_pointer_chain: ResolveRelativePointerChain,
    resolve_global_pointer_chain: ResolveGlobalPointerChain,
) -> Vec<SymbolTreeNode>
where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
    ReadScalarField: Fn(&ProjectSymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String> + Copy,
    ResolveRelativePointerChain: Fn(&ProjectSymbolLocator, &SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
    ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
{
    let mut symbol_tree_nodes = Vec::new();
    let mut module_symbol_claims: BTreeMap<String, Vec<ProjectSymbolClaim>> = BTreeMap::new();
    let mut module_fields_by_name: BTreeMap<String, Vec<ProjectSymbolModuleField>> = BTreeMap::new();
    let mut module_sizes_by_name: BTreeMap<String, u64> = BTreeMap::new();
    let mut absolute_symbol_claims = Vec::new();

    for symbol_module in project_symbol_catalog.get_symbol_modules() {
        module_sizes_by_name.insert(symbol_module.get_module_name().to_string(), symbol_module.get_size());
        module_fields_by_name.insert(symbol_module.get_module_name().to_string(), symbol_module.get_fields().to_vec());
    }

    for symbol_claim in project_symbol_catalog.get_symbol_claims() {
        match symbol_claim.get_locator() {
            ProjectSymbolLocator::AbsoluteAddress { .. } => absolute_symbol_claims.push(symbol_claim),
            ProjectSymbolLocator::ModuleOffset { module_name, .. } => {
                module_sizes_by_name.entry(module_name.clone()).or_insert(0);
                module_symbol_claims
                    .entry(module_name.clone())
                    .or_default()
                    .push(symbol_claim.clone());
            }
        }
    }

    for (module_name, module_size) in module_sizes_by_name {
        let mut symbol_claims = module_symbol_claims.remove(&module_name).unwrap_or_default();
        for module_field in module_fields_by_name.remove(&module_name).unwrap_or_default() {
            let mut module_field_symbol_claim = ProjectSymbolClaim::new_module_offset(
                module_field.get_display_name().to_string(),
                module_name.clone(),
                module_field.get_offset(),
                module_field.get_struct_layout_id().to_string(),
            );
            module_field_symbol_claim
                .get_metadata_mut()
                .insert(SYMBOL_TREE_ORIGIN_METADATA_KEY.to_string(), SYMBOL_TREE_MODULE_FIELD_ORIGIN.to_string());
            symbol_claims.push(module_field_symbol_claim);
        }
        symbol_claims.sort_by_key(|symbol_claim| symbol_claim.get_locator().get_focus_address());
        let module_root_layout_descriptor = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == module_name);
        let module_root_layout_size = module_root_layout_descriptor
            .map(|struct_layout_descriptor| {
                resolve_struct_size_in_bytes(
                    project_symbol_catalog,
                    struct_layout_descriptor.get_struct_layout_definition(),
                    resolve_primitive_size_in_bytes,
                    &mut HashSet::new(),
                )
            })
            .unwrap_or(0);
        let effective_module_size = module_size.max(module_root_layout_size).max(
            symbol_claims
                .iter()
                .fold(0_u64, |maximum_extent, symbol_claim| {
                    let claim_size_in_bytes = resolve_symbol_claim_size_in_bytes(project_symbol_catalog, symbol_claim, resolve_primitive_size_in_bytes);
                    maximum_extent.max(
                        symbol_claim
                            .get_locator()
                            .get_focus_address()
                            .saturating_add(claim_size_in_bytes),
                    )
                }),
        );
        let module_node_key = module_node_key(&module_name);
        let can_expand = effective_module_size > 0 || !symbol_claims.is_empty();
        let is_expanded = can_expand && expanded_tree_node_keys.contains(&module_node_key);
        append_module_space_node(&mut symbol_tree_nodes, module_name.clone(), effective_module_size, can_expand);

        if !is_expanded {
            continue;
        }

        if let Some(module_root_layout_descriptor) = module_root_layout_descriptor
            && !module_root_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .is_empty()
        {
            append_module_root_layout_children(
                &mut symbol_tree_nodes,
                project_symbol_catalog,
                &module_name,
                effective_module_size,
                module_root_layout_descriptor.get_struct_layout_definition(),
                &symbol_claims,
                expanded_tree_node_keys,
                resolved_pointer_targets_by_node_key,
                resolve_primitive_size_in_bytes,
                read_scalar_field,
                resolve_relative_pointer_chain,
                resolve_global_pointer_chain,
            );
            continue;
        }

        let mut next_unassigned_span_offset = 0_u64;

        for symbol_claim in symbol_claims {
            let claim_offset = symbol_claim.get_locator().get_focus_address();

            if claim_offset > next_unassigned_span_offset {
                append_unassigned_segment_node(
                    &mut symbol_tree_nodes,
                    &module_name,
                    next_unassigned_span_offset,
                    claim_offset.saturating_sub(next_unassigned_span_offset),
                );
            }

            append_symbol_claim_node(
                &mut symbol_tree_nodes,
                project_symbol_catalog,
                &symbol_claim,
                1,
                expanded_tree_node_keys,
                resolved_pointer_targets_by_node_key,
                resolve_primitive_size_in_bytes,
                read_scalar_field,
                resolve_relative_pointer_chain,
                resolve_global_pointer_chain,
            );

            let claim_size_in_bytes = resolve_symbol_claim_size_in_bytes(project_symbol_catalog, &symbol_claim, resolve_primitive_size_in_bytes);
            next_unassigned_span_offset = next_unassigned_span_offset.max(claim_offset.saturating_add(claim_size_in_bytes));
        }

        if effective_module_size > next_unassigned_span_offset {
            append_unassigned_segment_node(
                &mut symbol_tree_nodes,
                &module_name,
                next_unassigned_span_offset,
                effective_module_size.saturating_sub(next_unassigned_span_offset),
            );
        }
    }

    if !absolute_symbol_claims.is_empty() {
        absolute_symbol_claims.sort_by_key(|symbol_claim| symbol_claim.get_locator().get_focus_address());
        let module_name = String::from("Absolute / Unmapped");
        let module_node_key = module_node_key(&module_name);
        let is_expanded = expanded_tree_node_keys.contains(&module_node_key);
        append_module_space_node(&mut symbol_tree_nodes, module_name, 0, true);

        if !is_expanded {
            return symbol_tree_nodes;
        }

        for symbol_claim in absolute_symbol_claims {
            append_symbol_claim_node(
                &mut symbol_tree_nodes,
                project_symbol_catalog,
                symbol_claim,
                1,
                expanded_tree_node_keys,
                resolved_pointer_targets_by_node_key,
                resolve_primitive_size_in_bytes,
                read_scalar_field,
                resolve_relative_pointer_chain,
                resolve_global_pointer_chain,
            );
        }
    }

    symbol_tree_nodes
}

pub fn resolve_symbol_tree_node_size_in_bytes<ResolvePrimitiveSize>(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_tree_node: &SymbolTreeNode,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
) -> u64
where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
{
    if let SymbolTreeNodeKind::UnassignedSegment { length, .. } = symbol_tree_node.get_kind() {
        return *length;
    }

    let Ok(symbolic_field_definition) = SymbolicFieldDefinition::from_str(&symbol_tree_node.get_display_type_id()) else {
        return 0;
    };

    resolve_field_size_in_bytes(
        project_symbol_catalog,
        &symbolic_field_definition,
        resolve_primitive_size_in_bytes,
        &mut HashSet::new(),
    )
}

fn append_unassigned_segment_node(
    symbol_tree_nodes: &mut Vec<SymbolTreeNode>,
    module_name: &str,
    offset: u64,
    length: u64,
) {
    if length == 0 {
        return;
    }

    let display_name = format!("UNASSIGNED_{:08X}", offset);
    let full_path = format!("{}.{}", module_name, display_name);

    symbol_tree_nodes.push(SymbolTreeNode::new(
        format!("unassigned:{}:{:X}:{:X}", module_name, offset, length),
        SymbolTreeNodeKind::UnassignedSegment {
            module_name: module_name.to_string(),
            offset,
            length,
        },
        1,
        display_name,
        full_path,
        String::new(),
        ProjectSymbolLocator::new_module_offset(module_name.to_string(), offset),
        String::from("UNASSIGNED"),
        ContainerType::ArrayFixed(length),
        false,
    ));
}

fn append_module_root_layout_children<ResolvePrimitiveSize, ReadScalarField, ResolveRelativePointerChain, ResolveGlobalPointerChain>(
    symbol_tree_nodes: &mut Vec<SymbolTreeNode>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    module_name: &str,
    effective_module_size: u64,
    module_root_layout_definition: &SymbolicStructDefinition,
    symbol_claims: &[ProjectSymbolClaim],
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    read_scalar_field: ReadScalarField,
    resolve_relative_pointer_chain: ResolveRelativePointerChain,
    resolve_global_pointer_chain: ResolveGlobalPointerChain,
) where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
    ReadScalarField: Fn(&ProjectSymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String> + Copy,
    ResolveRelativePointerChain: Fn(&ProjectSymbolLocator, &SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
    ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
{
    let mut next_sequential_offset = 0_u64;
    let mut consumed_symbol_claim_offsets = HashSet::new();

    for (field_index, field_definition) in module_root_layout_definition.get_fields().iter().enumerate() {
        if field_definition.is_unassigned() {
            let unassigned_size_in_bytes = field_definition.get_unassigned_size_in_bytes().unwrap_or(0);
            if unassigned_size_in_bytes > 0 {
                append_unassigned_segment_node(symbol_tree_nodes, module_name, next_sequential_offset, unassigned_size_in_bytes);
            }
            next_sequential_offset = next_sequential_offset.saturating_add(unassigned_size_in_bytes);
            continue;
        }

        let field_offset = match field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                if module_root_layout_definition.get_layout_kind().is_union() =>
            {
                0
            }
            SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
        };
        if field_offset > next_sequential_offset {
            append_unassigned_segment_node(
                symbol_tree_nodes,
                module_name,
                next_sequential_offset,
                field_offset.saturating_sub(next_sequential_offset),
            );
        }

        let field_size_in_bytes = resolve_field_size_in_bytes(project_symbol_catalog, field_definition, resolve_primitive_size_in_bytes, &mut HashSet::new());
        next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));

        let matching_symbol_claim = symbol_claims.iter().find(|symbol_claim| {
            matches!(
                symbol_claim.get_locator(),
                ProjectSymbolLocator::ModuleOffset {
                    module_name: claim_module_name,
                    offset
                } if claim_module_name == module_name && *offset == field_offset
            )
        });

        if let Some(symbol_claim) = matching_symbol_claim {
            consumed_symbol_claim_offsets.insert(field_offset);
            append_symbol_claim_node(
                symbol_tree_nodes,
                project_symbol_catalog,
                symbol_claim,
                1,
                expanded_tree_node_keys,
                resolved_pointer_targets_by_node_key,
                resolve_primitive_size_in_bytes,
                read_scalar_field,
                resolve_relative_pointer_chain,
                resolve_global_pointer_chain,
            );
            continue;
        }

        append_module_root_layout_field_node(
            symbol_tree_nodes,
            project_symbol_catalog,
            module_name,
            field_index,
            field_definition,
            field_offset,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
        );
    }

    for symbol_claim in symbol_claims {
        let ProjectSymbolLocator::ModuleOffset { offset, .. } = symbol_claim.get_locator() else {
            continue;
        };
        if consumed_symbol_claim_offsets.contains(offset) {
            continue;
        }
        append_symbol_claim_node(
            symbol_tree_nodes,
            project_symbol_catalog,
            symbol_claim,
            1,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
        );
    }

    if effective_module_size > next_sequential_offset {
        append_unassigned_segment_node(
            symbol_tree_nodes,
            module_name,
            next_sequential_offset,
            effective_module_size.saturating_sub(next_sequential_offset),
        );
    }
}

fn append_module_root_layout_field_node<ResolvePrimitiveSize, ReadScalarField, ResolveRelativePointerChain, ResolveGlobalPointerChain>(
    symbol_tree_nodes: &mut Vec<SymbolTreeNode>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    module_name: &str,
    field_index: usize,
    field_definition: &SymbolicFieldDefinition,
    field_offset: u64,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    read_scalar_field: ReadScalarField,
    resolve_relative_pointer_chain: ResolveRelativePointerChain,
    resolve_global_pointer_chain: ResolveGlobalPointerChain,
) where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
    ReadScalarField: Fn(&ProjectSymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String> + Copy,
    ResolveRelativePointerChain: Fn(&ProjectSymbolLocator, &SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
    ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
{
    if field_definition.is_unassigned() {
        return;
    }

    let field_display_name = if field_definition.get_field_name().is_empty() {
        format!("field_{}", field_index)
    } else {
        field_definition.get_field_name().to_string()
    };
    let field_node_key = format!("module_layout:{}:{:X}:{}", module_name, field_offset, field_display_name);
    let field_locator = ProjectSymbolLocator::new_module_offset(module_name.to_string(), field_offset);
    let can_expand = data_type_ref_can_expand(
        project_symbol_catalog,
        field_definition.get_data_type_ref(),
        field_definition.get_container_type(),
        &mut HashSet::new(),
    );
    let is_expanded = can_expand && expanded_tree_node_keys.contains(&field_node_key);
    let symbol_claim_locator_key = field_locator.to_locator_key();

    symbol_tree_nodes.push(SymbolTreeNode::new(
        field_node_key.clone(),
        SymbolTreeNodeKind::StructField,
        1,
        field_display_name.clone(),
        field_display_name.clone(),
        symbol_claim_locator_key.clone(),
        field_locator.clone(),
        field_definition.get_data_type_ref().to_string(),
        field_definition.get_container_type(),
        can_expand,
    ));

    if is_expanded {
        append_field_children(
            symbol_tree_nodes,
            project_symbol_catalog,
            &symbol_claim_locator_key,
            &field_node_key,
            &field_display_name,
            &field_locator,
            field_definition.get_data_type_ref(),
            field_definition.get_container_type(),
            2,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
            &mut HashSet::new(),
        );
    }
}

fn append_module_space_node(
    symbol_tree_nodes: &mut Vec<SymbolTreeNode>,
    module_name: String,
    size: u64,
    can_expand: bool,
) {
    let node_key = module_node_key(&module_name);

    let symbol_tree_node = SymbolTreeNode::new(
        node_key,
        SymbolTreeNodeKind::ModuleSpace {
            module_name: module_name.clone(),
            size,
        },
        0,
        module_name.clone(),
        module_name,
        String::new(),
        ProjectSymbolLocator::new_absolute_address(0),
        String::from("u8"),
        ContainerType::ArrayFixed(size),
        can_expand,
    );

    symbol_tree_nodes.push(symbol_tree_node);
}

fn module_node_key(module_name: &str) -> String {
    format!("module:{}", module_name)
}

fn append_symbol_claim_node<ResolvePrimitiveSize, ReadScalarField, ResolveRelativePointerChain, ResolveGlobalPointerChain>(
    symbol_tree_nodes: &mut Vec<SymbolTreeNode>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_claim: &ProjectSymbolClaim,
    depth: usize,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    read_scalar_field: ReadScalarField,
    resolve_relative_pointer_chain: ResolveRelativePointerChain,
    resolve_global_pointer_chain: ResolveGlobalPointerChain,
) where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
    ReadScalarField: Fn(&ProjectSymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String> + Copy,
    ResolveRelativePointerChain: Fn(&ProjectSymbolLocator, &SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
    ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
{
    let root_node_key = if symbol_claim
        .get_metadata()
        .get(SYMBOL_TREE_ORIGIN_METADATA_KEY)
        .is_some_and(|origin| origin == SYMBOL_TREE_MODULE_FIELD_ORIGIN)
    {
        format!("module_field:{}", symbol_claim.get_symbol_locator_key())
    } else {
        format!("claim:{}", symbol_claim.get_symbol_locator_key())
    };
    let symbol_claim_type = resolve_symbol_claim_type(project_symbol_catalog, symbol_claim.get_struct_layout_id());
    let can_expand = symbol_claim_type.can_expand(project_symbol_catalog);
    let is_expanded = can_expand && expanded_tree_node_keys.contains(&root_node_key);

    let root_symbol_tree_node = SymbolTreeNode::new(
        root_node_key.clone(),
        SymbolTreeNodeKind::SymbolClaim {
            symbol_locator_key: symbol_claim.get_symbol_locator_key().to_string(),
        },
        depth,
        symbol_claim.get_display_name().to_string(),
        symbol_claim.get_display_name().to_string(),
        symbol_claim.get_symbol_locator_key().to_string(),
        symbol_claim.get_locator().clone(),
        symbol_claim_type.symbol_type_id().to_string(),
        symbol_claim_type.container_type(),
        can_expand,
    );

    symbol_tree_nodes.push(root_symbol_tree_node);

    if !is_expanded {
        return;
    }

    match symbol_claim_type {
        ResolvedSymbolClaimType::Struct { struct_layout_definition, .. } => append_struct_field_nodes(
            symbol_tree_nodes,
            project_symbol_catalog,
            &symbol_claim.get_symbol_locator_key(),
            &root_node_key,
            symbol_claim.get_display_name(),
            symbol_claim.get_locator(),
            &struct_layout_definition,
            depth + 1,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
            &mut HashSet::new(),
        ),
        ResolvedSymbolClaimType::Field {
            data_type_ref, container_type, ..
        } => append_field_children(
            symbol_tree_nodes,
            project_symbol_catalog,
            &symbol_claim.get_symbol_locator_key(),
            &root_node_key,
            symbol_claim.get_display_name(),
            symbol_claim.get_locator(),
            &data_type_ref,
            container_type,
            depth + 1,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
            &mut HashSet::new(),
        ),
    }
}

fn append_struct_field_nodes<ResolvePrimitiveSize, ReadScalarField, ResolveRelativePointerChain, ResolveGlobalPointerChain>(
    symbol_tree_nodes: &mut Vec<SymbolTreeNode>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_claim_locator_key: &str,
    parent_node_key: &str,
    parent_full_path: &str,
    parent_locator: &ProjectSymbolLocator,
    struct_layout_definition: &SymbolicStructDefinition,
    depth: usize,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    read_scalar_field: ReadScalarField,
    resolve_relative_pointer_chain: ResolveRelativePointerChain,
    resolve_global_pointer_chain: ResolveGlobalPointerChain,
    visited_struct_layout_ids: &mut HashSet<String>,
) where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
    ReadScalarField: Fn(&ProjectSymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String> + Copy,
    ResolveRelativePointerChain: Fn(&ProjectSymbolLocator, &SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
    ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
{
    let global_symbol_resolver_session = SymbolicGlobalSymbolResolverSession::default();
    let resolved_symbolic_struct = resolve_symbolic_struct_definition_with_resolvers_and_symbol_fields_and_relative_pointer_chains(
        struct_layout_definition,
        |data_type_ref| {
            Some(resolve_data_type_size_in_bytes(
                project_symbol_catalog,
                data_type_ref,
                resolve_primitive_size_in_bytes,
                &mut HashSet::new(),
            ))
        },
        |field_definition, field_offset, field_size_in_bytes| {
            let field_locator = offset_locator(parent_locator, field_offset);

            read_scalar_field(&field_locator, field_definition, field_size_in_bytes)
        },
        |resolver_id| {
            project_symbol_catalog
                .find_symbolic_resolver_descriptor(resolver_id)
                .map(|resolver_descriptor| resolver_descriptor.get_resolver_definition().clone())
        },
        |data_type_ref| resolve_struct_layout_definition(project_symbol_catalog, data_type_ref.get_data_type_id()),
        |module_name, symbol_path| {
            resolve_global_symbol_field_value(
                &global_symbol_resolver_session,
                module_name,
                symbol_path,
                &|module_name, root_symbol_name| resolve_global_symbol_roots(project_symbol_catalog, module_name, root_symbol_name),
                &|data_type_ref| {
                    Some(resolve_data_type_size_in_bytes(
                        project_symbol_catalog,
                        data_type_ref,
                        resolve_primitive_size_in_bytes,
                        &mut HashSet::new(),
                    ))
                },
                &|field_locator, field_definition, field_size_in_bytes| read_scalar_field(field_locator, field_definition, field_size_in_bytes),
                &|resolver_id| {
                    project_symbol_catalog
                        .find_symbolic_resolver_descriptor(resolver_id)
                        .map(|resolver_descriptor| resolver_descriptor.get_resolver_definition().clone())
                },
                &|data_type_ref| resolve_struct_layout_definition(project_symbol_catalog, data_type_ref.get_data_type_id()),
                &offset_locator,
            )
        },
        |pointer_chain| resolve_relative_pointer_chain(parent_locator, pointer_chain),
        resolve_global_pointer_chain,
        &SymbolicStructResolverOptions::default(),
    );
    let has_single_active_union_variant = if struct_layout_definition.get_layout_kind().is_union() {
        let active_variant_count = resolved_symbolic_struct
            .get_fields()
            .iter()
            .filter(|resolved_field| resolved_field.get_variant_activation().is_active())
            .count();
        let has_unresolved_variant_activation = resolved_symbolic_struct
            .get_fields()
            .iter()
            .any(|resolved_field| {
                matches!(
                    resolved_field.get_variant_activation(),
                    ResolvedSymbolicFieldVariantActivation::Ambiguous | ResolvedSymbolicFieldVariantActivation::Unresolved { .. }
                )
            });

        active_variant_count == 1 && !has_unresolved_variant_activation
    } else {
        false
    };

    for (field_index, (field_definition, resolved_symbolic_field)) in struct_layout_definition
        .get_fields()
        .iter()
        .filter(|field_definition| !field_definition.is_unassigned())
        .zip(resolved_symbolic_struct.get_fields())
        .enumerate()
    {
        if has_single_active_union_variant
            && matches!(
                resolved_symbolic_field.get_variant_activation(),
                ResolvedSymbolicFieldVariantActivation::Inactive
            )
        {
            continue;
        }

        let field_display_name = if field_definition.get_field_name().is_empty() {
            format!("field_{}", field_index)
        } else {
            field_definition.get_field_name().to_string()
        };
        let field_full_path = format!("{}.{}", parent_full_path, field_display_name);
        let field_node_key = format!("{}::{}", parent_node_key, field_display_name);
        let field_locator = offset_locator(
            parent_locator,
            resolved_symbolic_field
                .get_offset_in_bytes()
                .unwrap_or_default(),
        );
        let field_container_type = resolved_symbolic_field
            .get_displayed_element_count()
            .and_then(|element_count| {
                field_definition
                    .get_container_type()
                    .with_fixed_element_count(element_count)
            })
            .unwrap_or_else(|| field_definition.get_container_type());
        let can_expand = data_type_ref_can_expand(
            project_symbol_catalog,
            field_definition.get_data_type_ref(),
            field_container_type,
            &mut HashSet::new(),
        );
        let is_expanded = can_expand && expanded_tree_node_keys.contains(&field_node_key);

        let field_symbol_tree_node = SymbolTreeNode::new(
            field_node_key.clone(),
            SymbolTreeNodeKind::StructField,
            depth,
            field_display_name.clone(),
            field_full_path.clone(),
            symbol_claim_locator_key.to_string(),
            field_locator.clone(),
            field_definition.get_data_type_ref().to_string(),
            field_container_type,
            can_expand,
        );

        symbol_tree_nodes.push(field_symbol_tree_node);

        if is_expanded {
            append_field_children(
                symbol_tree_nodes,
                project_symbol_catalog,
                symbol_claim_locator_key,
                &field_node_key,
                &field_full_path,
                &field_locator,
                field_definition.get_data_type_ref(),
                field_container_type,
                depth + 1,
                expanded_tree_node_keys,
                resolved_pointer_targets_by_node_key,
                resolve_primitive_size_in_bytes,
                read_scalar_field,
                resolve_relative_pointer_chain,
                resolve_global_pointer_chain,
                visited_struct_layout_ids,
            );
        }
    }
}

fn append_field_children<ResolvePrimitiveSize, ReadScalarField, ResolveRelativePointerChain, ResolveGlobalPointerChain>(
    symbol_tree_nodes: &mut Vec<SymbolTreeNode>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_claim_locator_key: &str,
    parent_node_key: &str,
    parent_full_path: &str,
    parent_locator: &ProjectSymbolLocator,
    data_type_ref: &DataTypeRef,
    container_type: ContainerType,
    depth: usize,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    read_scalar_field: ReadScalarField,
    resolve_relative_pointer_chain: ResolveRelativePointerChain,
    resolve_global_pointer_chain: ResolveGlobalPointerChain,
    visited_struct_layout_ids: &mut HashSet<String>,
) where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
    ReadScalarField: Fn(&ProjectSymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String> + Copy,
    ResolveRelativePointerChain: Fn(&ProjectSymbolLocator, &SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
    ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
{
    match container_type {
        ContainerType::ArrayFixed(array_length) => append_fixed_array_element_nodes(
            symbol_tree_nodes,
            project_symbol_catalog,
            symbol_claim_locator_key,
            parent_node_key,
            parent_full_path,
            parent_locator,
            data_type_ref,
            ContainerType::None,
            array_length,
            depth,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
            visited_struct_layout_ids,
        ),
        ContainerType::PointerArrayFixed(pointer_size, array_length) => append_fixed_array_element_nodes(
            symbol_tree_nodes,
            project_symbol_catalog,
            symbol_claim_locator_key,
            parent_node_key,
            parent_full_path,
            parent_locator,
            data_type_ref,
            ContainerType::Pointer(pointer_size),
            array_length,
            depth,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
            visited_struct_layout_ids,
        ),
        ContainerType::None => {
            if let Some(nested_struct_layout_definition) = resolve_struct_layout_definition(project_symbol_catalog, data_type_ref.get_data_type_id()) {
                let type_identifier = data_type_ref.get_data_type_id().to_string();

                if !visited_struct_layout_ids.insert(type_identifier.clone()) {
                    return;
                }

                append_struct_field_nodes(
                    symbol_tree_nodes,
                    project_symbol_catalog,
                    symbol_claim_locator_key,
                    parent_node_key,
                    parent_full_path,
                    parent_locator,
                    &nested_struct_layout_definition,
                    depth,
                    expanded_tree_node_keys,
                    resolved_pointer_targets_by_node_key,
                    resolve_primitive_size_in_bytes,
                    read_scalar_field,
                    resolve_relative_pointer_chain,
                    resolve_global_pointer_chain,
                    visited_struct_layout_ids,
                );

                visited_struct_layout_ids.remove(&type_identifier);
            }
        }
        ContainerType::Pointer(_) => {
            let Some(resolved_pointer_target) = resolved_pointer_targets_by_node_key.get(parent_node_key) else {
                return;
            };
            let pointer_target_node_key = format!("{}::target", parent_node_key);
            let pointer_target_full_path = format!("{}.*", parent_full_path);
            let pointer_target_locator = resolved_pointer_target.get_target_locator().clone();
            let can_expand = data_type_ref_can_expand(project_symbol_catalog, data_type_ref, ContainerType::None, visited_struct_layout_ids);
            let is_expanded = can_expand && expanded_tree_node_keys.contains(&pointer_target_node_key);

            let pointer_target_symbol_tree_node = SymbolTreeNode::new(
                pointer_target_node_key.clone(),
                SymbolTreeNodeKind::PointerTarget,
                depth,
                String::from("*"),
                pointer_target_full_path.clone(),
                symbol_claim_locator_key.to_string(),
                pointer_target_locator.clone(),
                data_type_ref.to_string(),
                ContainerType::None,
                can_expand,
            );

            symbol_tree_nodes.push(pointer_target_symbol_tree_node);

            if is_expanded {
                append_field_children(
                    symbol_tree_nodes,
                    project_symbol_catalog,
                    symbol_claim_locator_key,
                    &pointer_target_node_key,
                    &pointer_target_full_path,
                    &pointer_target_locator,
                    data_type_ref,
                    ContainerType::None,
                    depth + 1,
                    expanded_tree_node_keys,
                    resolved_pointer_targets_by_node_key,
                    resolve_primitive_size_in_bytes,
                    read_scalar_field,
                    resolve_relative_pointer_chain,
                    resolve_global_pointer_chain,
                    visited_struct_layout_ids,
                );
            }
        }
        ContainerType::Array | ContainerType::PointerArray(_) => {}
    }
}

fn append_fixed_array_element_nodes<ResolvePrimitiveSize, ReadScalarField, ResolveRelativePointerChain, ResolveGlobalPointerChain>(
    symbol_tree_nodes: &mut Vec<SymbolTreeNode>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_claim_locator_key: &str,
    parent_node_key: &str,
    parent_full_path: &str,
    parent_locator: &ProjectSymbolLocator,
    data_type_ref: &DataTypeRef,
    element_container_type: ContainerType,
    array_length: u64,
    depth: usize,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    read_scalar_field: ReadScalarField,
    resolve_relative_pointer_chain: ResolveRelativePointerChain,
    resolve_global_pointer_chain: ResolveGlobalPointerChain,
    visited_struct_layout_ids: &mut HashSet<String>,
) where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
    ReadScalarField: Fn(&ProjectSymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String> + Copy,
    ResolveRelativePointerChain: Fn(&ProjectSymbolLocator, &SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
    ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
{
    if !data_type_ref_can_expand(project_symbol_catalog, data_type_ref, element_container_type, visited_struct_layout_ids) {
        return;
    }

    let element_unit_size_in_bytes = resolve_data_type_size_in_bytes(
        project_symbol_catalog,
        data_type_ref,
        resolve_primitive_size_in_bytes,
        visited_struct_layout_ids,
    );
    let element_size_in_bytes = element_container_type.get_total_size_in_bytes(element_unit_size_in_bytes);

    for array_element_index in 0..array_length {
        let element_display_name = format!("[{}]", array_element_index);
        let element_full_path = format!("{}{}", parent_full_path, element_display_name);
        let element_node_key = format!("{}::{}", parent_node_key, element_display_name);
        let element_offset = element_size_in_bytes.saturating_mul(array_element_index);
        let element_locator = offset_locator(parent_locator, element_offset);
        let can_expand = data_type_ref_can_expand(project_symbol_catalog, data_type_ref, element_container_type, visited_struct_layout_ids);
        let is_expanded = can_expand && expanded_tree_node_keys.contains(&element_node_key);

        let element_symbol_tree_node = SymbolTreeNode::new(
            element_node_key.clone(),
            SymbolTreeNodeKind::StructField,
            depth,
            element_display_name,
            element_full_path.clone(),
            symbol_claim_locator_key.to_string(),
            element_locator.clone(),
            data_type_ref.to_string(),
            element_container_type,
            can_expand,
        );

        symbol_tree_nodes.push(element_symbol_tree_node);

        if is_expanded {
            append_field_children(
                symbol_tree_nodes,
                project_symbol_catalog,
                symbol_claim_locator_key,
                &element_node_key,
                &element_full_path,
                &element_locator,
                data_type_ref,
                element_container_type,
                depth + 1,
                expanded_tree_node_keys,
                resolved_pointer_targets_by_node_key,
                resolve_primitive_size_in_bytes,
                read_scalar_field,
                resolve_relative_pointer_chain,
                resolve_global_pointer_chain,
                visited_struct_layout_ids,
            );
        }
    }
}

fn resolve_global_symbol_roots(
    project_symbol_catalog: &ProjectSymbolCatalog,
    module_name: &str,
    root_symbol_name: &str,
) -> Vec<SymbolicGlobalSymbolRoot<ProjectSymbolLocator>> {
    let mut global_symbol_roots = Vec::new();

    if let Some(symbol_module) = project_symbol_catalog.find_symbol_module(module_name) {
        global_symbol_roots.extend(
            symbol_module
                .get_fields()
                .iter()
                .filter(|module_field| module_field.get_display_name() == root_symbol_name)
                .map(|module_field| {
                    SymbolicGlobalSymbolRoot::new(
                        ProjectSymbolLocator::new_module_offset(module_name.to_string(), module_field.get_offset()),
                        resolve_symbol_claim_type(project_symbol_catalog, module_field.get_struct_layout_id()).to_global_symbol_root_type(),
                    )
                }),
        );
    }

    global_symbol_roots.extend(
        project_symbol_catalog
            .get_symbol_claims()
            .iter()
            .filter_map(|symbol_claim| {
                if symbol_claim.get_display_name() != root_symbol_name {
                    return None;
                }

                let ProjectSymbolLocator::ModuleOffset {
                    module_name: claim_module_name,
                    ..
                } = symbol_claim.get_locator()
                else {
                    return None;
                };

                (claim_module_name == module_name).then(|| {
                    SymbolicGlobalSymbolRoot::new(
                        symbol_claim.get_locator().clone(),
                        resolve_symbol_claim_type(project_symbol_catalog, symbol_claim.get_struct_layout_id()).to_global_symbol_root_type(),
                    )
                })
            }),
    );

    global_symbol_roots
}

fn data_type_ref_can_expand(
    project_symbol_catalog: &ProjectSymbolCatalog,
    data_type_ref: &DataTypeRef,
    container_type: ContainerType,
    visited_struct_layout_ids: &mut HashSet<String>,
) -> bool {
    match container_type {
        ContainerType::ArrayFixed(array_length) => {
            array_length > 0 && data_type_ref_can_expand(project_symbol_catalog, data_type_ref, ContainerType::None, visited_struct_layout_ids)
        }
        ContainerType::PointerArrayFixed(pointer_size, array_length) => {
            array_length > 0
                && data_type_ref_can_expand(
                    project_symbol_catalog,
                    data_type_ref,
                    ContainerType::Pointer(pointer_size),
                    visited_struct_layout_ids,
                )
        }
        ContainerType::Pointer(_) => true,
        ContainerType::None => {
            let data_type_id = data_type_ref.get_data_type_id();

            if !visited_struct_layout_ids.insert(data_type_id.to_string()) {
                return false;
            }

            let can_expand = resolve_struct_layout_definition(project_symbol_catalog, data_type_id)
                .map(|struct_layout_definition| {
                    struct_layout_definition
                        .get_fields()
                        .iter()
                        .any(|field_definition| !field_definition.is_unassigned())
                })
                .unwrap_or(false);

            visited_struct_layout_ids.remove(data_type_id);

            can_expand
        }
        ContainerType::Array | ContainerType::PointerArray(_) => false,
    }
}

fn resolve_struct_layout_definition(
    project_symbol_catalog: &ProjectSymbolCatalog,
    struct_layout_id: &str,
) -> Option<SymbolicStructDefinition> {
    resolve_exact_struct_layout_descriptor(project_symbol_catalog, struct_layout_id)
        .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_definition().clone())
        .or_else(|| {
            if struct_layout_id.contains(';') {
                SymbolicStructDefinition::from_str(struct_layout_id).ok()
            } else {
                None
            }
        })
}

fn resolve_exact_struct_layout_descriptor<'a>(
    project_symbol_catalog: &'a ProjectSymbolCatalog,
    struct_layout_id: &str,
) -> Option<&'a StructLayoutDescriptor> {
    project_symbol_catalog
        .get_struct_layout_descriptors()
        .iter()
        .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == struct_layout_id)
}

fn resolve_symbol_claim_size_in_bytes<ResolvePrimitiveSize>(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_claim: &ProjectSymbolClaim,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
) -> u64
where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
{
    match resolve_symbol_claim_type(project_symbol_catalog, symbol_claim.get_struct_layout_id()) {
        ResolvedSymbolClaimType::Struct { struct_layout_definition, .. } => resolve_struct_size_in_bytes(
            project_symbol_catalog,
            &struct_layout_definition,
            resolve_primitive_size_in_bytes,
            &mut HashSet::new(),
        ),
        ResolvedSymbolClaimType::Field {
            data_type_ref, container_type, ..
        } => {
            let unit_size_in_bytes =
                resolve_data_type_size_in_bytes(project_symbol_catalog, &data_type_ref, resolve_primitive_size_in_bytes, &mut HashSet::new());

            container_type.get_total_size_in_bytes(unit_size_in_bytes)
        }
    }
}

fn resolve_field_size_in_bytes<ResolvePrimitiveSize>(
    project_symbol_catalog: &ProjectSymbolCatalog,
    field_definition: &SymbolicFieldDefinition,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    visited_struct_layout_ids: &mut HashSet<String>,
) -> u64
where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
{
    let unit_size_in_bytes = resolve_data_type_size_in_bytes(
        project_symbol_catalog,
        field_definition.get_data_type_ref(),
        resolve_primitive_size_in_bytes,
        visited_struct_layout_ids,
    );

    field_definition
        .get_container_type()
        .get_total_size_in_bytes(unit_size_in_bytes)
}

fn resolve_struct_size_in_bytes<ResolvePrimitiveSize>(
    project_symbol_catalog: &ProjectSymbolCatalog,
    struct_layout_definition: &SymbolicStructDefinition,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    visited_struct_layout_ids: &mut HashSet<String>,
) -> u64
where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
{
    let mut next_sequential_offset = 0_u64;

    for field_definition in struct_layout_definition.get_fields() {
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
        let field_size_in_bytes = resolve_field_size_in_bytes(
            project_symbol_catalog,
            field_definition,
            resolve_primitive_size_in_bytes,
            visited_struct_layout_ids,
        );

        next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
    }

    next_sequential_offset.max(
        struct_layout_definition
            .get_declared_size_in_bytes()
            .unwrap_or(0),
    )
}

fn resolve_data_type_size_in_bytes<ResolvePrimitiveSize>(
    project_symbol_catalog: &ProjectSymbolCatalog,
    data_type_ref: &DataTypeRef,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    visited_struct_layout_ids: &mut HashSet<String>,
) -> u64
where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
{
    let data_type_id = data_type_ref.get_data_type_id();

    if !visited_struct_layout_ids.insert(data_type_id.to_string()) {
        return 0;
    }

    let size_in_bytes = resolve_struct_layout_definition(project_symbol_catalog, data_type_id)
        .map(|struct_layout_definition| {
            resolve_struct_size_in_bytes(
                project_symbol_catalog,
                &struct_layout_definition,
                resolve_primitive_size_in_bytes,
                visited_struct_layout_ids,
            )
        })
        .or_else(|| resolve_primitive_size_in_bytes(data_type_ref))
        .unwrap_or(1);

    visited_struct_layout_ids.remove(data_type_id);

    size_in_bytes
}

fn offset_locator(
    project_symbol_locator: &ProjectSymbolLocator,
    offset: u64,
) -> ProjectSymbolLocator {
    match project_symbol_locator {
        ProjectSymbolLocator::AbsoluteAddress { address } => ProjectSymbolLocator::new_absolute_address(address.saturating_add(offset)),
        ProjectSymbolLocator::ModuleOffset {
            module_name,
            offset: base_offset,
        } => ProjectSymbolLocator::new_module_offset(module_name.clone(), base_offset.saturating_add(offset)),
    }
}

enum ResolvedSymbolClaimType {
    Struct {
        symbol_type_id: String,
        struct_layout_definition: SymbolicStructDefinition,
    },
    Field {
        symbol_type_id: String,
        data_type_ref: DataTypeRef,
        container_type: ContainerType,
    },
}

impl ResolvedSymbolClaimType {
    fn symbol_type_id(&self) -> &str {
        match self {
            Self::Struct { symbol_type_id, .. } | Self::Field { symbol_type_id, .. } => symbol_type_id,
        }
    }

    fn container_type(&self) -> ContainerType {
        match self {
            Self::Struct { .. } => ContainerType::None,
            Self::Field { container_type, .. } => *container_type,
        }
    }

    fn can_expand(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) -> bool {
        match self {
            Self::Struct { struct_layout_definition, .. } => !struct_layout_definition.get_fields().is_empty(),
            Self::Field {
                data_type_ref, container_type, ..
            } => data_type_ref_can_expand(project_symbol_catalog, data_type_ref, *container_type, &mut HashSet::new()),
        }
    }

    fn to_global_symbol_root_type(&self) -> SymbolicGlobalSymbolRootType {
        match self {
            Self::Struct { struct_layout_definition, .. } => SymbolicGlobalSymbolRootType::Struct {
                struct_layout_definition: struct_layout_definition.clone(),
            },
            Self::Field {
                data_type_ref, container_type, ..
            } => SymbolicGlobalSymbolRootType::Field {
                data_type_ref: data_type_ref.clone(),
                container_type: *container_type,
            },
        }
    }
}

fn resolve_symbol_claim_type(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_claim_type_id: &str,
) -> ResolvedSymbolClaimType {
    if let Some(struct_layout_descriptor) = resolve_exact_struct_layout_descriptor(project_symbol_catalog, symbol_claim_type_id) {
        return ResolvedSymbolClaimType::Struct {
            symbol_type_id: symbol_claim_type_id.to_string(),
            struct_layout_definition: struct_layout_descriptor.get_struct_layout_definition().clone(),
        };
    }

    if let Ok(symbolic_field_definition) = SymbolicFieldDefinition::from_str(symbol_claim_type_id) {
        return ResolvedSymbolClaimType::Field {
            symbol_type_id: symbolic_field_definition.get_data_type_ref().to_string(),
            data_type_ref: symbolic_field_definition.get_data_type_ref().clone(),
            container_type: symbolic_field_definition.get_container_type(),
        };
    }

    ResolvedSymbolClaimType::Struct {
        symbol_type_id: symbol_claim_type_id.to_string(),
        struct_layout_definition: SymbolicStructDefinition::from_str(symbol_claim_type_id).unwrap_or_else(|_| SymbolicStructDefinition::new_anonymous(vec![])),
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolvedPointerTarget, build_symbol_tree_nodes, build_symbol_tree_nodes_with_scalar_reader};
    use crate::registries::symbols::{struct_layout_descriptor::StructLayoutDescriptor, symbolic_resolver_descriptor::SymbolicResolverDescriptor};
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::{
            project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
            project_symbol_module::ProjectSymbolModule, symbol_tree::symbol_tree_node::SymbolTreeNodeKind,
        },
        structs::{
            symbolic_field_definition::SymbolicFieldDefinition,
            symbolic_resolver_definition::{
                SymbolicResolverBinaryOperator, SymbolicResolverDefinition, SymbolicResolverEvaluationError, SymbolicResolverNode,
                SymbolicResolverRelativeSymbolPath,
            },
            symbolic_struct_definition::SymbolicStructDefinition,
        },
    };
    use std::collections::{HashMap, HashSet};
    use std::str::FromStr;

    #[test]
    fn build_symbol_tree_nodes_derives_nested_struct_and_array_children() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![
                StructLayoutDescriptor::new(
                    String::from("player"),
                    SymbolicStructDefinition::new(
                        String::from("player"),
                        vec![
                            SymbolicFieldDefinition::new_named(String::from("health"), DataTypeRef::new("u32"), ContainerType::None),
                            SymbolicFieldDefinition::new_named(String::from("position"), DataTypeRef::new("vec2"), ContainerType::None),
                            SymbolicFieldDefinition::new_named(String::from("items"), DataTypeRef::new("u16"), ContainerType::ArrayFixed(2)),
                            SymbolicFieldDefinition::new_named(
                                String::from("next"),
                                DataTypeRef::new("player"),
                                ContainerType::Pointer(PointerScanPointerSize::Pointer64),
                            ),
                        ],
                    ),
                ),
                StructLayoutDescriptor::new(
                    String::from("vec2"),
                    SymbolicStructDefinition::new(
                        String::from("vec2"),
                        vec![
                            SymbolicFieldDefinition::new_named(String::from("x"), DataTypeRef::new("u32"), ContainerType::None),
                            SymbolicFieldDefinition::new_named(String::from("y"), DataTypeRef::new("u32"), ContainerType::None),
                        ],
                    ),
                ),
            ],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x100,
                String::from("player"),
            )],
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:Absolute / Unmapped"),
            String::from("claim:absolute:100"),
            String::from("claim:absolute:100::position"),
            String::from("claim:absolute:100::items"),
        ]);

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u16" => Some(2),
                "u32" => Some(4),
                _ => None,
            },
        );

        assert_eq!(symbol_tree_nodes.len(), 8);
        assert_eq!(symbol_tree_nodes[0].get_display_name(), "Absolute / Unmapped");
        assert_eq!(
            symbol_tree_nodes[0].get_kind(),
            &SymbolTreeNodeKind::ModuleSpace {
                module_name: String::from("Absolute / Unmapped"),
                size: 0,
            }
        );
        assert_eq!(symbol_tree_nodes[1].get_display_name(), "Player");
        assert_eq!(symbol_tree_nodes[1].get_depth(), 1);
        assert_eq!(
            symbol_tree_nodes[1].get_kind(),
            &SymbolTreeNodeKind::SymbolClaim {
                symbol_locator_key: String::from("absolute:100"),
            }
        );
        assert_eq!(symbol_tree_nodes[2].get_full_path(), "Player.health");
        assert_eq!(symbol_tree_nodes[2].get_locator(), &ProjectSymbolLocator::new_absolute_address(0x100));
        assert_eq!(symbol_tree_nodes[3].get_full_path(), "Player.position");
        assert_eq!(symbol_tree_nodes[3].get_locator(), &ProjectSymbolLocator::new_absolute_address(0x104));
        assert_eq!(symbol_tree_nodes[4].get_full_path(), "Player.position.x");
        assert_eq!(symbol_tree_nodes[4].get_locator(), &ProjectSymbolLocator::new_absolute_address(0x104));
        assert_eq!(symbol_tree_nodes[5].get_full_path(), "Player.position.y");
        assert_eq!(symbol_tree_nodes[5].get_locator(), &ProjectSymbolLocator::new_absolute_address(0x108));
        assert_eq!(symbol_tree_nodes[6].get_full_path(), "Player.items");
        assert_eq!(symbol_tree_nodes[6].get_locator(), &ProjectSymbolLocator::new_absolute_address(0x10C));
        assert_eq!(symbol_tree_nodes[6].get_display_type_id(), "u16[2]");
        assert_eq!(symbol_tree_nodes[6].can_expand(), false);
        assert_eq!(symbol_tree_nodes[7].get_full_path(), "Player.next");
        assert_eq!(symbol_tree_nodes[7].can_expand(), true);
    }

    #[test]
    fn build_symbol_tree_nodes_shows_empty_module_root_as_unassigned_segment() {
        use crate::structures::projects::project_symbol_module::ProjectSymbolModule;

        let project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![ProjectSymbolModule::new(String::from("game.exe"), 0x20)], Vec::new(), Vec::new());

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &HashSet::from([String::from("module:game.exe")]),
            &HashMap::new(),
            |_| None,
        );

        assert_eq!(symbol_tree_nodes.len(), 2);
        assert_eq!(
            symbol_tree_nodes[0].get_kind(),
            &SymbolTreeNodeKind::ModuleSpace {
                module_name: String::from("game.exe"),
                size: 0x20,
            }
        );
        assert_eq!(
            symbol_tree_nodes[1].get_kind(),
            &SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 0,
                length: 0x20,
            }
        );
        assert_eq!(symbol_tree_nodes[1].get_display_type_id(), "UNASSIGNED[32]");
        assert_eq!(symbol_tree_nodes[1].can_expand(), false);
    }

    #[test]
    fn build_symbol_tree_nodes_reads_module_fields_as_module_children() {
        use crate::structures::projects::{project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField};

        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x20);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Health"), 0x04, String::from("u32")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![symbol_module], Vec::new(), Vec::new());

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &HashSet::from([String::from("module:game.exe")]),
            &HashMap::new(),
            |data_type_ref| (data_type_ref.get_data_type_id() == "u32").then_some(4),
        );

        assert_eq!(symbol_tree_nodes.len(), 4);
        assert_eq!(
            symbol_tree_nodes[1].get_kind(),
            &SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 0,
                length: 0x04,
            }
        );
        assert_eq!(symbol_tree_nodes[2].get_display_name(), "Health");
        assert_eq!(symbol_tree_nodes[2].get_symbol_type_id(), "u32");
        assert_eq!(
            symbol_tree_nodes[2].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x04)
        );
        assert_eq!(
            symbol_tree_nodes[3].get_kind(),
            &SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 0x08,
                length: 0x18,
            }
        );
    }

    #[test]
    fn build_symbol_tree_nodes_expands_module_field_fixed_arrays_of_structs() {
        use crate::structures::projects::{project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField};

        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x220);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(
                String::from("Section Headers"),
                0x178,
                String::from("section_header[3]"),
            ));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![symbol_module],
            vec![StructLayoutDescriptor::new(
                String::from("section_header"),
                SymbolicStructDefinition::new(
                    String::from("section_header"),
                    vec![
                        SymbolicFieldDefinition::new_named(String::from("Name"), DataTypeRef::new("u8"), ContainerType::ArrayFixed(8)),
                        SymbolicFieldDefinition::new_named(String::from("VirtualSize"), DataTypeRef::new("u32"), ContainerType::None),
                    ],
                ),
            )],
            Vec::new(),
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:game.exe"),
            String::from("module_field:module:game.exe:178"),
            String::from("module_field:module:game.exe:178::[0]"),
        ]);

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u32" => Some(4),
                _ => None,
            },
        );

        assert_eq!(symbol_tree_nodes[2].get_display_name(), "Section Headers");
        assert_eq!(symbol_tree_nodes[2].get_display_type_id(), "section_header[3]");
        assert!(symbol_tree_nodes[2].can_expand());
        assert_eq!(symbol_tree_nodes[3].get_full_path(), "Section Headers[0]");
        assert_eq!(
            symbol_tree_nodes[3].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x178)
        );
        assert_eq!(symbol_tree_nodes[4].get_full_path(), "Section Headers[0].Name");
        assert_eq!(symbol_tree_nodes[5].get_full_path(), "Section Headers[0].VirtualSize");
        assert_eq!(
            symbol_tree_nodes[5].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x180)
        );
        assert_eq!(symbol_tree_nodes[6].get_full_path(), "Section Headers[1]");
        assert_eq!(
            symbol_tree_nodes[6].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x184)
        );
    }

    #[test]
    fn build_symbol_tree_nodes_resolves_pe_shaped_dynamic_section_headers() {
        use crate::structures::projects::{project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField};
        use std::str::FromStr;

        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x220);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("PE Headers"), 0, String::from("pe_headers")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            vec![symbol_module],
            vec![
                StructLayoutDescriptor::new(
                    String::from("pe_headers"),
                    SymbolicStructDefinition::new(
                        String::from("pe_headers"),
                        vec![
                            SymbolicFieldDefinition::from_str("e_lfanew:u32 @ +0x3C").expect("Expected e_lfanew field to parse."),
                            SymbolicFieldDefinition::from_str("NumberOfSections:u16 @ resolver(pe.number_of_sections_offset)")
                                .expect("Expected section count field to parse."),
                            SymbolicFieldDefinition::from_str("SizeOfOptionalHeader:u16 @ resolver(pe.optional_header_size_offset)")
                                .expect("Expected optional header size field to parse."),
                            SymbolicFieldDefinition::from_str(
                                "SectionHeaders:section_header[resolver(pe.number_of_sections)] @ resolver(pe.section_headers_offset)",
                            )
                            .expect("Expected section headers field to parse."),
                        ],
                    ),
                ),
                StructLayoutDescriptor::new(
                    String::from("section_header"),
                    SymbolicStructDefinition::new(
                        String::from("section_header"),
                        vec![
                            SymbolicFieldDefinition::new_named(String::from("Name"), DataTypeRef::new("u8"), ContainerType::ArrayFixed(8)),
                            SymbolicFieldDefinition::new_named(String::from("VirtualSize"), DataTypeRef::new("u32"), ContainerType::None),
                        ],
                    ),
                ),
            ],
            vec![
                SymbolicResolverDescriptor::new(
                    String::from("pe.number_of_sections"),
                    SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("NumberOfSections"))),
                ),
                SymbolicResolverDescriptor::new(
                    String::from("pe.number_of_sections_offset"),
                    SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
                        SymbolicResolverBinaryOperator::Add,
                        SymbolicResolverNode::new_local_field(String::from("e_lfanew")),
                        SymbolicResolverNode::new_literal(6),
                    )),
                ),
                SymbolicResolverDescriptor::new(
                    String::from("pe.optional_header_size_offset"),
                    SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
                        SymbolicResolverBinaryOperator::Add,
                        SymbolicResolverNode::new_local_field(String::from("e_lfanew")),
                        SymbolicResolverNode::new_literal(20),
                    )),
                ),
                SymbolicResolverDescriptor::new(
                    String::from("pe.section_headers_offset"),
                    SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
                        SymbolicResolverBinaryOperator::Add,
                        SymbolicResolverNode::new_binary(
                            SymbolicResolverBinaryOperator::Add,
                            SymbolicResolverNode::new_local_field(String::from("e_lfanew")),
                            SymbolicResolverNode::new_literal(24),
                        ),
                        SymbolicResolverNode::new_local_field(String::from("SizeOfOptionalHeader")),
                    )),
                ),
            ],
            Vec::new(),
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:game.exe"),
            String::from("module_field:module:game.exe:0"),
            String::from("module_field:module:game.exe:0::SectionHeaders"),
            String::from("module_field:module:game.exe:0::SectionHeaders::[0]"),
        ]);

        let symbol_tree_nodes = build_symbol_tree_nodes_with_scalar_reader(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u16" => Some(2),
                "u32" => Some(4),
                _ => None,
            },
            |_, field_definition, _| match field_definition.get_field_name() {
                "e_lfanew" => Ok(Some(0x80)),
                "NumberOfSections" => Ok(Some(3)),
                "SizeOfOptionalHeader" => Ok(Some(0xE0)),
                _ => Ok(None),
            },
        );

        let section_headers_entry = symbol_tree_nodes
            .iter()
            .find(|symbol_tree_node| symbol_tree_node.get_full_path() == "PE Headers.SectionHeaders")
            .expect("Expected dynamic section headers entry.");

        assert_eq!(
            section_headers_entry.get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x178)
        );
        assert_eq!(section_headers_entry.get_display_type_id(), "section_header[3]");
        assert!(section_headers_entry.can_expand());
        assert!(symbol_tree_nodes.iter().any(
            |symbol_tree_node| symbol_tree_node.get_full_path() == "PE Headers.SectionHeaders[0].VirtualSize"
                && symbol_tree_node.get_locator() == &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x180)
        ));
    }

    #[test]
    fn build_symbol_tree_nodes_resolves_dynamic_fields_through_catalog_resolvers() {
        use crate::structures::projects::{project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField};

        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x100);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Items"), 0, String::from("items")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            vec![symbol_module],
            vec![StructLayoutDescriptor::new(
                String::from("items"),
                SymbolicStructDefinition::new(
                    String::from("items"),
                    vec![
                        SymbolicFieldDefinition::from_str("count:u32 @ +0").expect("Expected count field to parse."),
                        SymbolicFieldDefinition::from_str("values:u16[resolver(items.count)] @ resolver(items.values_offset)")
                            .expect("Expected resolver field to parse."),
                    ],
                ),
            )],
            vec![
                SymbolicResolverDescriptor::new(
                    String::from("items.count"),
                    SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("count"))),
                ),
                SymbolicResolverDescriptor::new(
                    String::from("items.values_offset"),
                    SymbolicResolverDefinition::new(SymbolicResolverNode::new_binary(
                        SymbolicResolverBinaryOperator::Add,
                        SymbolicResolverNode::new_literal(2),
                        SymbolicResolverNode::new_literal(2),
                    )),
                ),
            ],
            Vec::new(),
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:game.exe"),
            String::from("module_field:module:game.exe:0"),
        ]);

        let symbol_tree_nodes = build_symbol_tree_nodes_with_scalar_reader(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u16" => Some(2),
                "u32" => Some(4),
                _ => None,
            },
            |_, field_definition, _| match field_definition.get_field_name() {
                "count" => Ok(Some(3)),
                _ => Ok(None),
            },
        );

        let values_entry = symbol_tree_nodes
            .iter()
            .find(|symbol_tree_node| symbol_tree_node.get_full_path() == "Items.values")
            .expect("Expected dynamic values entry.");

        assert_eq!(values_entry.get_display_type_id(), "u16[3]");
        assert_eq!(
            values_entry.get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 4)
        );
    }

    #[test]
    fn build_symbol_tree_nodes_expands_fixed_array_by_display_count_resolver() {
        use crate::structures::projects::{project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField};

        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x3000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("EntityList"), 0, String::from("entity_list")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            vec![symbol_module],
            vec![
                StructLayoutDescriptor::new(
                    String::from("entity_list"),
                    SymbolicStructDefinition::new(
                        String::from("entity_list"),
                        vec![
                            SymbolicFieldDefinition::from_str("count:u32 @ +0").expect("Expected count field to parse."),
                            SymbolicFieldDefinition::from_str("entities:Entity[1024] display resolver(entity_list.count) @ +8")
                                .expect("Expected fixed display-count field to parse."),
                            SymbolicFieldDefinition::from_str("tail:u32").expect("Expected tail field to parse."),
                        ],
                    ),
                ),
                StructLayoutDescriptor::new(
                    String::from("Entity"),
                    SymbolicStructDefinition::new(
                        String::from("Entity"),
                        vec![SymbolicFieldDefinition::from_str("id:u64").expect("Expected id field to parse.")],
                    ),
                ),
            ],
            vec![SymbolicResolverDescriptor::new(
                String::from("entity_list.count"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("count"))),
            )],
            Vec::new(),
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:game.exe"),
            String::from("module_field:module:game.exe:0"),
            String::from("module_field:module:game.exe:0::entities"),
        ]);

        let symbol_tree_nodes = build_symbol_tree_nodes_with_scalar_reader(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                "u64" => Some(8),
                _ => None,
            },
            |_, field_definition, _| match field_definition.get_field_name() {
                "count" => Ok(Some(3)),
                _ => Ok(None),
            },
        );

        let entities_entry = symbol_tree_nodes
            .iter()
            .find(|symbol_tree_node| symbol_tree_node.get_full_path() == "EntityList.entities")
            .expect("Expected fixed array entry.");

        assert_eq!(entities_entry.get_display_type_id(), "Entity[3]");
        assert!(
            symbol_tree_nodes
                .iter()
                .any(|symbol_tree_node| symbol_tree_node.get_full_path() == "EntityList.entities[2]")
        );
        assert!(
            !symbol_tree_nodes
                .iter()
                .any(|symbol_tree_node| symbol_tree_node.get_full_path() == "EntityList.entities[3]")
        );

        let tail_entry = symbol_tree_nodes
            .iter()
            .find(|symbol_tree_node| symbol_tree_node.get_full_path() == "EntityList.tail")
            .expect("Expected tail entry.");

        assert_eq!(
            tail_entry.get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x2008)
        );
    }

    #[test]
    fn build_symbol_tree_nodes_resolves_dynamic_fields_through_global_symbol_resolvers() {
        use crate::structures::projects::{project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField};

        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x300);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Globals"), 0x100, String::from("globals")));
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Items"), 0x200, String::from("items")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            vec![symbol_module],
            vec![
                StructLayoutDescriptor::new(
                    String::from("globals"),
                    SymbolicStructDefinition::new(
                        String::from("globals"),
                        vec![SymbolicFieldDefinition::from_str("item_count:u32 @ +0").expect("Expected item_count field to parse.")],
                    ),
                ),
                StructLayoutDescriptor::new(
                    String::from("items"),
                    SymbolicStructDefinition::new(
                        String::from("items"),
                        vec![SymbolicFieldDefinition::from_str("values:u16[resolver(global.item_count)] @ +0").expect("Expected values field to parse.")],
                    ),
                ),
            ],
            vec![SymbolicResolverDescriptor::new(
                String::from("global.item_count"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_global_symbol_field(
                    String::from("game.exe"),
                    SymbolicResolverRelativeSymbolPath::from_dot_path("Globals.item_count"),
                )),
            )],
            Vec::new(),
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:game.exe"),
            String::from("module_field:module:game.exe:200"),
        ]);

        let symbol_tree_nodes = build_symbol_tree_nodes_with_scalar_reader(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u16" => Some(2),
                "u32" => Some(4),
                _ => None,
            },
            |field_locator, field_definition, _| match (field_locator, field_definition.get_field_name()) {
                (ProjectSymbolLocator::ModuleOffset { module_name, offset }, "item_count") if module_name == "game.exe" && *offset == 0x100 => Ok(Some(5)),
                _ => Ok(None),
            },
        );

        let values_entry = symbol_tree_nodes
            .iter()
            .find(|symbol_tree_node| symbol_tree_node.get_full_path() == "Items.values")
            .expect("Expected dynamic values entry.");

        assert_eq!(values_entry.get_display_type_id(), "u16[5]");
        assert_eq!(
            values_entry.get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x200)
        );
    }

    #[test]
    fn build_symbol_tree_nodes_resolves_dynamic_fields_through_global_pointer_chain_resolvers() {
        use crate::structures::{
            memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
            pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
            projects::{project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField},
        };

        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x300);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Items"), 0x200, String::from("items")));
        let pointer_chain = SymbolicPointerChain::new(
            String::from("game.exe"),
            vec![
                SymbolicPointerChainLink::Symbol(String::from("Items")),
                SymbolicPointerChainLink::Offset(0x20),
            ],
            PointerScanPointerSize::Pointer64,
        );
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            vec![symbol_module],
            vec![StructLayoutDescriptor::new(
                String::from("items"),
                SymbolicStructDefinition::new(
                    String::from("items"),
                    vec![SymbolicFieldDefinition::from_str("values:u16[resolver(items.count_via_pointer)] @ +0").expect("Expected values field to parse.")],
                ),
            )],
            vec![SymbolicResolverDescriptor::new(
                String::from("items.count_via_pointer"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_global_pointer_chain(pointer_chain.clone())),
            )],
            Vec::new(),
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:game.exe"),
            String::from("module_field:module:game.exe:200"),
        ]);

        let symbol_tree_nodes = super::build_symbol_tree_nodes_with_scalar_reader_and_pointer_chains(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u16" => Some(2),
                _ => None,
            },
            |_, _, _| Ok(None),
            |_, pointer_chain| Err(SymbolicResolverEvaluationError::UnknownRelativePointerChain(pointer_chain.to_string())),
            |resolved_pointer_chain| {
                assert_eq!(resolved_pointer_chain, &pointer_chain);
                Ok(4)
            },
        );

        let values_entry = symbol_tree_nodes
            .iter()
            .find(|symbol_tree_node| symbol_tree_node.get_full_path() == "Items.values")
            .expect("Expected dynamic values entry.");

        assert_eq!(values_entry.get_display_type_id(), "u16[4]");
    }

    #[test]
    fn build_symbol_tree_nodes_resolves_dynamic_fields_through_relative_pointer_chain_resolvers() {
        use crate::structures::{
            memory::symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
            pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
            projects::{project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField},
        };

        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x300);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Items"), 0x200, String::from("items")));
        let pointer_chain = SymbolicPointerChain::new_absolute(
            vec![
                SymbolicPointerChainLink::Symbol(String::from("entity_list")),
                SymbolicPointerChainLink::Offset(0x20),
            ],
            PointerScanPointerSize::Pointer64,
        );
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            vec![symbol_module],
            vec![StructLayoutDescriptor::new(
                String::from("items"),
                SymbolicStructDefinition::new(
                    String::from("items"),
                    vec![
                        SymbolicFieldDefinition::from_str("entity_list:u64 @ +0x10").expect("Expected entity_list field to parse."),
                        SymbolicFieldDefinition::from_str("values:u16[resolver(items.count_via_pointer)] @ +0").expect("Expected values field to parse."),
                    ],
                ),
            )],
            vec![SymbolicResolverDescriptor::new(
                String::from("items.count_via_pointer"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_relative_pointer_chain(pointer_chain)),
            )],
            Vec::new(),
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:game.exe"),
            String::from("module_field:module:game.exe:200"),
        ]);

        let symbol_tree_nodes = super::build_symbol_tree_nodes_with_scalar_reader_and_pointer_chains(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u16" => Some(2),
                "u64" => Some(8),
                _ => None,
            },
            |_, _, _| Ok(None),
            |root_locator, resolved_pointer_chain| {
                assert_eq!(root_locator, &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x200));
                assert_eq!(
                    resolved_pointer_chain.get_links(),
                    &[
                        SymbolicPointerChainLink::Offset(0x10),
                        SymbolicPointerChainLink::Offset(0x20)
                    ]
                );
                Ok(4)
            },
            |pointer_chain| Err(SymbolicResolverEvaluationError::UnknownGlobalPointerChain(pointer_chain.to_string())),
        );

        let values_entry = symbol_tree_nodes
            .iter()
            .find(|symbol_tree_node| symbol_tree_node.get_full_path() == "Items.values")
            .expect("Expected dynamic values entry.");

        assert_eq!(values_entry.get_display_type_id(), "u16[4]");
    }

    #[test]
    fn build_symbol_tree_nodes_does_not_expand_collapsed_module_space() {
        use crate::structures::projects::project_symbol_module::ProjectSymbolModule;

        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(String::from("game.exe"), 0x40000000)],
            Vec::new(),
            Vec::new(),
        );

        let symbol_tree_nodes = build_symbol_tree_nodes(&project_symbol_catalog, &HashSet::new(), &HashMap::new(), |_| None);

        assert_eq!(symbol_tree_nodes.len(), 1);
        assert_eq!(
            symbol_tree_nodes[0].get_kind(),
            &SymbolTreeNodeKind::ModuleSpace {
                module_name: String::from("game.exe"),
                size: 0x40000000,
            }
        );
    }

    #[test]
    fn build_symbol_tree_nodes_keeps_large_fixed_array_as_preview_leaf() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Blob"),
                String::from("game.exe"),
                0,
                String::from("u8[300]"),
            )],
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:game.exe"),
            String::from("claim:module:game.exe:0"),
        ]);

        let symbol_tree_nodes = build_symbol_tree_nodes(&project_symbol_catalog, &expanded_tree_node_keys, &HashMap::new(), |data_type_ref| {
            (data_type_ref.get_data_type_id() == "u8").then_some(1)
        });

        assert_eq!(symbol_tree_nodes.len(), 2);
        assert_eq!(symbol_tree_nodes[0].get_display_name(), "game.exe");
        assert_eq!(symbol_tree_nodes[1].get_display_name(), "Blob");
        assert_eq!(symbol_tree_nodes[1].get_display_type_id(), "u8[300]");
        assert_eq!(symbol_tree_nodes[1].can_expand(), false);
    }

    #[test]
    fn build_symbol_tree_nodes_omits_unassigned_storage_fields_but_preserves_offsets() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("header"),
                SymbolicStructDefinition::new(
                    String::from("header"),
                    vec![
                        SymbolicFieldDefinition::new_unassigned(12),
                        SymbolicFieldDefinition::from_str("value:u32").expect("Expected value field to parse."),
                    ],
                ),
            )],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Header"),
                0x100,
                String::from("header"),
            )],
        );

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &HashSet::from([
                String::from("module:Absolute / Unmapped"),
                String::from("claim:absolute:100"),
            ]),
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u32" => Some(4),
                _ => None,
            },
        );

        assert!(
            !symbol_tree_nodes
                .iter()
                .any(|symbol_tree_node| symbol_tree_node.get_full_path() == "Header.reserved")
        );
        let value_entry = symbol_tree_nodes
            .iter()
            .find(|symbol_tree_node| symbol_tree_node.get_full_path() == "Header.value")
            .expect("Expected visible value field.");

        assert_eq!(value_entry.get_locator(), &ProjectSymbolLocator::new_absolute_address(0x10C));
    }

    #[test]
    fn build_symbol_tree_nodes_splits_module_space_into_unassigned_segments_around_symbol_claim() {
        use crate::structures::projects::project_symbol_module::ProjectSymbolModule;

        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(String::from("game.exe"), 0x2000)],
            Vec::new(),
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Health"),
                String::from("game.exe"),
                0x1234,
                String::from("u32"),
            )],
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:game.exe"),
            String::from("claim:module:game.exe:1234"),
        ]);

        let symbol_tree_nodes = build_symbol_tree_nodes(&project_symbol_catalog, &expanded_tree_node_keys, &HashMap::new(), |data_type_ref| {
            (data_type_ref.get_data_type_id() == "u32").then_some(4)
        });

        assert_eq!(symbol_tree_nodes.len(), 4);
        assert_eq!(symbol_tree_nodes[0].get_display_name(), "game.exe");
        assert_eq!(
            symbol_tree_nodes[1].get_kind(),
            &SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 0,
                length: 0x1234,
            }
        );
        assert_eq!(symbol_tree_nodes[1].get_display_type_id(), "UNASSIGNED[4660]");
        assert_eq!(symbol_tree_nodes[1].can_expand(), false);
        assert_eq!(symbol_tree_nodes[2].get_symbol_type_id(), "u32");
        assert_eq!(symbol_tree_nodes[2].get_container_type(), ContainerType::None);
        assert_eq!(symbol_tree_nodes[2].can_expand(), false);
        assert_eq!(
            symbol_tree_nodes[3].get_kind(),
            &SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 0x1238,
                length: 0xDC8,
            }
        );
        assert_eq!(symbol_tree_nodes[3].get_display_type_id(), "UNASSIGNED[3528]");
        assert_eq!(symbol_tree_nodes[3].can_expand(), false);
    }

    #[test]
    fn build_symbol_tree_nodes_sizes_overlapping_static_fields_by_span() {
        let variant_payload = SymbolicStructDefinition::new(
            String::from("variant_payload"),
            vec![
                SymbolicFieldDefinition::from_str("as_u64:u64 @ +0").expect("Expected u64 union field to parse."),
                SymbolicFieldDefinition::from_str("as_u32:u32 @ +0").expect("Expected u32 union field to parse."),
                SymbolicFieldDefinition::from_str("raw:u8[16] @ +0").expect("Expected raw union field to parse."),
            ],
        );
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("variant_payload"),
                variant_payload,
            )],
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Payload"),
                String::from("game.exe"),
                0x10,
                String::from("variant_payload"),
            )],
        );

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &HashSet::from([String::from("module:game.exe")]),
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u32" => Some(4),
                "u64" => Some(8),
                _ => None,
            },
        );

        assert_eq!(symbol_tree_nodes.len(), 3);
        assert_eq!(symbol_tree_nodes[1].get_display_type_id(), "UNASSIGNED[16]");
        assert_eq!(symbol_tree_nodes[2].get_symbol_type_id(), "variant_payload");
        assert_eq!(symbol_tree_nodes[0].get_display_type_id(), "u8[32]");
    }

    #[test]
    fn build_symbol_tree_nodes_sizes_struct_claims_by_declared_size() {
        let declared_payload = SymbolicStructDefinition::new(
            String::from("declared_payload"),
            vec![SymbolicFieldDefinition::from_str("value:u32").expect("Expected u32 field to parse.")],
        )
        .with_declared_size_in_bytes(Some(0x20));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("declared_payload"),
                declared_payload,
            )],
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Payload"),
                String::from("game.exe"),
                0x10,
                String::from("declared_payload"),
            )],
        );

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &HashSet::from([String::from("module:game.exe")]),
            &HashMap::new(),
            |data_type_ref| (data_type_ref.get_data_type_id() == "u32").then_some(4),
        );

        assert_eq!(symbol_tree_nodes.len(), 3);
        assert_eq!(symbol_tree_nodes[0].get_display_type_id(), "u8[48]");
        assert_eq!(symbol_tree_nodes[1].get_display_type_id(), "UNASSIGNED[16]");
        assert_eq!(symbol_tree_nodes[2].get_symbol_type_id(), "declared_payload");
    }

    #[test]
    fn build_symbol_tree_nodes_sizes_union_layout_fields_at_shared_offset() {
        let variant_payload = SymbolicStructDefinition::new_union(
            String::from("variant_payload"),
            vec![
                SymbolicFieldDefinition::from_str("as_u64:u64").expect("Expected u64 union field to parse."),
                SymbolicFieldDefinition::from_str("as_u32:u32").expect("Expected u32 union field to parse."),
                SymbolicFieldDefinition::from_str("raw:u8[16]").expect("Expected raw union field to parse."),
            ],
        );
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("variant_payload"),
                variant_payload,
            )],
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Payload"),
                String::from("game.exe"),
                0x10,
                String::from("variant_payload"),
            )],
        );

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &HashSet::from([
                String::from("module:game.exe"),
                String::from("claim:module:game.exe:10"),
            ]),
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u32" => Some(4),
                "u64" => Some(8),
                _ => None,
            },
        );

        let payload_child_locators = symbol_tree_nodes
            .iter()
            .filter(|symbol_tree_node| symbol_tree_node.get_full_path().starts_with("Payload."))
            .map(|symbol_tree_node| symbol_tree_node.get_locator().clone())
            .collect::<Vec<_>>();

        assert_eq!(
            payload_child_locators,
            vec![
                ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x10),
                ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x10),
                ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x10),
            ]
        );
    }

    #[test]
    fn build_symbol_tree_nodes_shows_only_single_active_union_variant() {
        let variant_payload = SymbolicStructDefinition::new_union(
            String::from("variant_payload"),
            vec![
                SymbolicFieldDefinition::from_str("alive:alive_payload active resolver(is_alive)").expect("Expected alive union field to parse."),
                SymbolicFieldDefinition::from_str("dead:dead_payload active resolver(is_dead)").expect("Expected dead union field to parse."),
            ],
        );
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            Vec::new(),
            vec![StructLayoutDescriptor::new(
                String::from("variant_payload"),
                variant_payload,
            )],
            vec![
                SymbolicResolverDescriptor::new(String::from("is_alive"), SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(0))),
                SymbolicResolverDescriptor::new(String::from("is_dead"), SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(1))),
            ],
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Payload"),
                String::from("game.exe"),
                0x10,
                String::from("variant_payload"),
            )],
        );

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &HashSet::from([
                String::from("module:game.exe"),
                String::from("claim:module:game.exe:10"),
            ]),
            &HashMap::new(),
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "alive_payload" | "dead_payload" => Some(4),
                _ => None,
            },
        );
        let payload_child_names = symbol_tree_nodes
            .iter()
            .filter(|symbol_tree_node| symbol_tree_node.get_full_path().starts_with("Payload."))
            .map(|symbol_tree_node| symbol_tree_node.get_display_name().to_string())
            .collect::<Vec<_>>();

        assert_eq!(payload_child_names, vec![String::from("dead")]);
    }

    #[test]
    fn build_symbol_tree_nodes_synthesizes_unassigned_module_segments_between_claims() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![
                ProjectSymbolClaim::new_module_offset(String::from("First"), String::from("game.exe"), 0x4, String::from("u32")),
                ProjectSymbolClaim::new_module_offset(String::from("Second"), String::from("game.exe"), 0xC, String::from("u32")),
            ],
        );

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &HashSet::from([String::from("module:game.exe")]),
            &HashMap::new(),
            |data_type_ref| (data_type_ref.get_data_type_id() == "u32").then_some(4),
        );

        assert_eq!(symbol_tree_nodes.len(), 5);
        assert_eq!(symbol_tree_nodes[0].get_display_name(), "game.exe");
        assert_eq!(
            symbol_tree_nodes[1].get_kind(),
            &SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 0,
                length: 4,
            }
        );
        assert_eq!(symbol_tree_nodes[1].get_display_type_id(), "UNASSIGNED[4]");
        assert_eq!(symbol_tree_nodes[1].can_expand(), false);
        assert_eq!(symbol_tree_nodes[2].get_display_name(), "First");
        assert_eq!(
            symbol_tree_nodes[3].get_kind(),
            &SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 8,
                length: 4,
            }
        );
        assert_eq!(symbol_tree_nodes[3].get_display_type_id(), "UNASSIGNED[4]");
        assert_eq!(symbol_tree_nodes[3].can_expand(), false);
        assert_eq!(symbol_tree_nodes[4].get_display_name(), "Second");
    }

    #[test]
    fn build_symbol_tree_nodes_uses_module_root_layout_unassigned_splits() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(String::from("game.exe"), 0x20)],
            vec![StructLayoutDescriptor::new(
                String::from("game.exe"),
                SymbolicStructDefinition::new(
                    String::from("game.exe"),
                    vec![
                        SymbolicFieldDefinition::new_unassigned(8),
                        SymbolicFieldDefinition::new_unassigned(8),
                        SymbolicFieldDefinition::new_named(String::from("Headers"), DataTypeRef::new("u32"), ContainerType::None),
                    ],
                )
                .with_declared_size_in_bytes(Some(0x20)),
            )],
            Vec::new(),
        );

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &HashSet::from([String::from("module:game.exe")]),
            &HashMap::new(),
            |data_type_ref| (data_type_ref.get_data_type_id() == "u32").then_some(4),
        );

        assert_eq!(symbol_tree_nodes.len(), 5);
        assert_eq!(symbol_tree_nodes[0].get_display_name(), "game.exe");
        assert_eq!(
            symbol_tree_nodes[1].get_kind(),
            &SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 0,
                length: 8,
            }
        );
        assert_eq!(
            symbol_tree_nodes[2].get_kind(),
            &SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 8,
                length: 8,
            }
        );
        assert_eq!(symbol_tree_nodes[3].get_display_name(), "Headers");
        assert_eq!(
            symbol_tree_nodes[4].get_kind(),
            &SymbolTreeNodeKind::UnassignedSegment {
                module_name: String::from("game.exe"),
                offset: 20,
                length: 12,
            }
        );
    }

    #[test]
    fn build_symbol_tree_nodes_maps_module_root_layout_field_back_to_existing_claim() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(String::from("game.exe"), 0x10)],
            vec![StructLayoutDescriptor::new(
                String::from("game.exe"),
                SymbolicStructDefinition::new(
                    String::from("game.exe"),
                    vec![
                        SymbolicFieldDefinition::new_unassigned(8),
                        SymbolicFieldDefinition::new_named(String::from("Health"), DataTypeRef::new("u32"), ContainerType::None),
                    ],
                )
                .with_declared_size_in_bytes(Some(0x10)),
            )],
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Health"),
                String::from("game.exe"),
                8,
                String::from("u32"),
            )],
        );

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &HashSet::from([String::from("module:game.exe")]),
            &HashMap::new(),
            |data_type_ref| (data_type_ref.get_data_type_id() == "u32").then_some(4),
        );

        assert_eq!(symbol_tree_nodes.len(), 4);
        assert_eq!(
            symbol_tree_nodes[2].get_kind(),
            &SymbolTreeNodeKind::SymbolClaim {
                symbol_locator_key: String::from("module:game.exe:8"),
            }
        );
    }

    #[test]
    fn build_symbol_tree_nodes_derives_pointer_target_children_from_resolved_targets() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("player"),
                SymbolicStructDefinition::new(
                    String::from("player"),
                    vec![
                        SymbolicFieldDefinition::new_named(String::from("health"), DataTypeRef::new("u32"), ContainerType::None),
                        SymbolicFieldDefinition::new_named(
                            String::from("next"),
                            DataTypeRef::new("player"),
                            ContainerType::Pointer(PointerScanPointerSize::Pointer64),
                        ),
                    ],
                ),
            )],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x100,
                String::from("player"),
            )],
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:Absolute / Unmapped"),
            String::from("claim:absolute:100"),
            String::from("claim:absolute:100::next"),
            String::from("claim:absolute:100::next::target"),
        ]);
        let resolved_pointer_targets_by_node_key = HashMap::from([(
            String::from("claim:absolute:100::next"),
            ResolvedPointerTarget::new(ProjectSymbolLocator::new_absolute_address(0x200), String::from("0x100 -> 0x200")),
        )]);

        let symbol_tree_nodes = build_symbol_tree_nodes(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &resolved_pointer_targets_by_node_key,
            |data_type_ref| (data_type_ref.get_data_type_id() == "u32").then_some(4),
        );

        assert_eq!(symbol_tree_nodes.len(), 7);
        assert_eq!(symbol_tree_nodes[0].get_display_name(), "Absolute / Unmapped");
        assert_eq!(symbol_tree_nodes[3].get_full_path(), "Player.next");
        assert_eq!(
            symbol_tree_nodes[3].get_container_type(),
            ContainerType::Pointer(PointerScanPointerSize::Pointer64)
        );
        assert_eq!(symbol_tree_nodes[4].get_kind(), &SymbolTreeNodeKind::PointerTarget);
        assert_eq!(symbol_tree_nodes[4].get_full_path(), "Player.next.*");
        assert_eq!(symbol_tree_nodes[4].get_locator(), &ProjectSymbolLocator::new_absolute_address(0x200));
        assert_eq!(symbol_tree_nodes[5].get_full_path(), "Player.next.*.health");
        assert_eq!(symbol_tree_nodes[5].get_locator(), &ProjectSymbolLocator::new_absolute_address(0x200));
        assert_eq!(symbol_tree_nodes[6].get_full_path(), "Player.next.*.next");
    }

    #[test]
    fn build_symbol_tree_nodes_expands_displayed_fixed_pointer_array_elements() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            Vec::new(),
            vec![
                StructLayoutDescriptor::new(
                    String::from("entity"),
                    SymbolicStructDefinition::new(
                        String::from("entity"),
                        vec![SymbolicFieldDefinition::new_named(
                            String::from("health"),
                            DataTypeRef::new("u32"),
                            ContainerType::None,
                        )],
                    ),
                ),
                StructLayoutDescriptor::new(
                    String::from("entity_list"),
                    SymbolicStructDefinition::new(
                        String::from("entity_list"),
                        vec![
                            SymbolicFieldDefinition::from_str("count:u32").expect("Expected count field to parse."),
                            SymbolicFieldDefinition::from_str("entities:entity*(u64)[1024] display resolver(entity_list.count) @ +8")
                                .expect("Expected pointer array field to parse."),
                        ],
                    ),
                ),
            ],
            vec![SymbolicResolverDescriptor::new(
                String::from("entity_list.count"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("count"))),
            )],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("EntityList"),
                0x100,
                String::from("entity_list"),
            )],
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("module:Absolute / Unmapped"),
            String::from("claim:absolute:100"),
            String::from("claim:absolute:100::entities"),
            String::from("claim:absolute:100::entities::[0]"),
            String::from("claim:absolute:100::entities::[0]::target"),
        ]);
        let resolved_pointer_targets_by_node_key = HashMap::from([(
            String::from("claim:absolute:100::entities::[0]"),
            ResolvedPointerTarget::new(ProjectSymbolLocator::new_absolute_address(0x500), String::from("0x108 -> 0x500")),
        )]);

        let symbol_tree_nodes = build_symbol_tree_nodes_with_scalar_reader(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &resolved_pointer_targets_by_node_key,
            |data_type_ref| match data_type_ref.get_data_type_id() {
                "u32" => Some(4),
                _ => None,
            },
            |field_locator, field_definition, _| match (field_locator, field_definition.get_field_name()) {
                (ProjectSymbolLocator::AbsoluteAddress { address }, "count") if *address == 0x100 => Ok(Some(2)),
                _ => Ok(None),
            },
        );

        assert!(symbol_tree_nodes.iter().any(|symbol_tree_node| {
            symbol_tree_node.get_full_path() == "EntityList.entities[0]"
                && symbol_tree_node.get_container_type() == ContainerType::Pointer(PointerScanPointerSize::Pointer64)
        }));
        assert!(
            symbol_tree_nodes
                .iter()
                .any(|symbol_tree_node| symbol_tree_node.get_full_path() == "EntityList.entities[1]")
        );
        assert!(
            !symbol_tree_nodes
                .iter()
                .any(|symbol_tree_node| symbol_tree_node.get_full_path() == "EntityList.entities[2]")
        );
        assert!(symbol_tree_nodes.iter().any(|symbol_tree_node| {
            symbol_tree_node.get_full_path() == "EntityList.entities[0].*.health"
                && symbol_tree_node.get_locator() == &ProjectSymbolLocator::new_absolute_address(0x500)
        }));
    }
}
