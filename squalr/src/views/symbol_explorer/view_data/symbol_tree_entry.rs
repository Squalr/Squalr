use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::container_type::ContainerType,
    projects::{project_root_symbol::ProjectRootSymbol, project_root_symbol_locator::ProjectRootSymbolLocator, project_symbol_catalog::ProjectSymbolCatalog},
    structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolTreeEntryKind {
    RootedSymbol { symbol_key: String },
    StructField,
    ArrayElement,
    PointerTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPointerTarget {
    target_locator: ProjectRootSymbolLocator,
    evaluated_pointer_path: String,
}

impl ResolvedPointerTarget {
    pub fn new(
        target_locator: ProjectRootSymbolLocator,
        evaluated_pointer_path: String,
    ) -> Self {
        Self {
            target_locator,
            evaluated_pointer_path,
        }
    }

    pub fn get_target_locator(&self) -> &ProjectRootSymbolLocator {
        &self.target_locator
    }

    pub fn get_evaluated_pointer_path(&self) -> &str {
        &self.evaluated_pointer_path
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolTreeEntry {
    node_key: String,
    kind: SymbolTreeEntryKind,
    depth: usize,
    display_name: String,
    full_path: String,
    promotion_display_name: String,
    root_symbol_key: String,
    locator: ProjectRootSymbolLocator,
    symbol_type_id: String,
    container_type: ContainerType,
    can_expand: bool,
    is_expanded: bool,
}

impl SymbolTreeEntry {
    pub fn new(
        node_key: String,
        kind: SymbolTreeEntryKind,
        depth: usize,
        display_name: String,
        full_path: String,
        promotion_display_name: String,
        root_symbol_key: String,
        locator: ProjectRootSymbolLocator,
        symbol_type_id: String,
        container_type: ContainerType,
        can_expand: bool,
        is_expanded: bool,
    ) -> Self {
        Self {
            node_key,
            kind,
            depth,
            display_name,
            full_path,
            promotion_display_name,
            root_symbol_key,
            locator,
            symbol_type_id,
            container_type,
            can_expand,
            is_expanded,
        }
    }

    pub fn get_node_key(&self) -> &str {
        &self.node_key
    }

    pub fn get_kind(&self) -> &SymbolTreeEntryKind {
        &self.kind
    }

    pub fn get_depth(&self) -> usize {
        self.depth
    }

    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn get_full_path(&self) -> &str {
        &self.full_path
    }

    pub fn get_promotion_display_name(&self) -> &str {
        &self.promotion_display_name
    }

    pub fn get_root_symbol_key(&self) -> &str {
        &self.root_symbol_key
    }

    pub fn get_locator(&self) -> &ProjectRootSymbolLocator {
        &self.locator
    }

    pub fn get_symbol_type_id(&self) -> &str {
        &self.symbol_type_id
    }

    pub fn get_promoted_symbol_type_id(&self) -> String {
        format!("{}{}", self.symbol_type_id, self.container_type)
    }

    pub fn get_container_type(&self) -> ContainerType {
        self.container_type
    }

    pub fn can_expand(&self) -> bool {
        self.can_expand
    }

    pub fn is_expanded(&self) -> bool {
        self.is_expanded
    }
}

pub fn build_symbol_tree_entries<ResolvePrimitiveSize>(
    project_symbol_catalog: &ProjectSymbolCatalog,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
) -> Vec<SymbolTreeEntry>
where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
{
    let mut symbol_tree_entries = Vec::new();

    for rooted_symbol in project_symbol_catalog.get_rooted_symbols() {
        append_rooted_symbol_entry(
            &mut symbol_tree_entries,
            project_symbol_catalog,
            rooted_symbol,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
        );
    }

    symbol_tree_entries
}

fn append_rooted_symbol_entry<ResolvePrimitiveSize>(
    symbol_tree_entries: &mut Vec<SymbolTreeEntry>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    rooted_symbol: &ProjectRootSymbol,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
) where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
{
    let root_node_key = format!("root:{}", rooted_symbol.get_symbol_key());
    let root_symbol_type = resolve_root_symbol_type(project_symbol_catalog, rooted_symbol.get_struct_layout_id());
    let can_expand = root_symbol_type.can_expand(project_symbol_catalog);
    let is_expanded = can_expand && expanded_tree_node_keys.contains(&root_node_key);

    symbol_tree_entries.push(SymbolTreeEntry::new(
        root_node_key.clone(),
        SymbolTreeEntryKind::RootedSymbol {
            symbol_key: rooted_symbol.get_symbol_key().to_string(),
        },
        0,
        rooted_symbol.get_display_name().to_string(),
        rooted_symbol.get_display_name().to_string(),
        rooted_symbol.get_display_name().to_string(),
        rooted_symbol.get_symbol_key().to_string(),
        rooted_symbol.get_root_locator().clone(),
        root_symbol_type.symbol_type_id().to_string(),
        root_symbol_type.container_type(),
        can_expand,
        is_expanded,
    ));

    if !is_expanded {
        return;
    }

    match root_symbol_type {
        ResolvedRootSymbolType::Struct { struct_layout_definition, .. } => append_struct_field_entries(
            symbol_tree_entries,
            project_symbol_catalog,
            rooted_symbol.get_symbol_key(),
            &root_node_key,
            rooted_symbol.get_display_name(),
            rooted_symbol.get_display_name(),
            rooted_symbol.get_root_locator(),
            &struct_layout_definition,
            1,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            &mut HashSet::new(),
        ),
        ResolvedRootSymbolType::Field {
            data_type_ref, container_type, ..
        } => append_field_children(
            symbol_tree_entries,
            project_symbol_catalog,
            rooted_symbol.get_symbol_key(),
            &root_node_key,
            rooted_symbol.get_display_name(),
            rooted_symbol.get_display_name(),
            rooted_symbol.get_root_locator(),
            &data_type_ref,
            container_type,
            1,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            &mut HashSet::new(),
        ),
    }
}

fn append_struct_field_entries<ResolvePrimitiveSize>(
    symbol_tree_entries: &mut Vec<SymbolTreeEntry>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    root_symbol_key: &str,
    parent_node_key: &str,
    parent_full_path: &str,
    parent_promotion_display_name: &str,
    parent_locator: &ProjectRootSymbolLocator,
    struct_layout_definition: &SymbolicStructDefinition,
    depth: usize,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    visited_struct_layout_ids: &mut HashSet<String>,
) where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
{
    let mut cumulative_field_offset = 0_u64;

    for (field_index, field_definition) in struct_layout_definition.get_fields().iter().enumerate() {
        let field_display_name = if field_definition.get_field_name().is_empty() {
            format!("field_{}", field_index)
        } else {
            field_definition.get_field_name().to_string()
        };
        let field_full_path = format!("{}.{}", parent_full_path, field_display_name);
        let field_promotion_display_name = if parent_promotion_display_name.is_empty() {
            field_display_name.clone()
        } else {
            format!("{}.{}", parent_promotion_display_name, field_display_name)
        };
        let field_symbol_type_id = format!("{}{}", field_definition.get_data_type_ref(), field_definition.get_container_type());
        let field_node_key = format!("{}::{}", parent_node_key, field_display_name);
        let field_locator = offset_locator(parent_locator, cumulative_field_offset);
        let can_expand = field_can_expand(project_symbol_catalog, field_definition);
        let is_expanded = can_expand && expanded_tree_node_keys.contains(&field_node_key);

        symbol_tree_entries.push(SymbolTreeEntry::new(
            field_node_key.clone(),
            SymbolTreeEntryKind::StructField,
            depth,
            field_display_name.clone(),
            field_full_path.clone(),
            field_promotion_display_name.clone(),
            root_symbol_key.to_string(),
            field_locator.clone(),
            field_symbol_type_id,
            field_definition.get_container_type(),
            can_expand,
            is_expanded,
        ));

        if is_expanded {
            append_field_children(
                symbol_tree_entries,
                project_symbol_catalog,
                root_symbol_key,
                &field_node_key,
                &field_full_path,
                &field_promotion_display_name,
                &field_locator,
                field_definition.get_data_type_ref(),
                field_definition.get_container_type(),
                depth + 1,
                expanded_tree_node_keys,
                resolved_pointer_targets_by_node_key,
                resolve_primitive_size_in_bytes,
                visited_struct_layout_ids,
            );
        }

        cumulative_field_offset = cumulative_field_offset.saturating_add(resolve_field_size_in_bytes(
            project_symbol_catalog,
            field_definition,
            resolve_primitive_size_in_bytes,
            visited_struct_layout_ids,
        ));
    }
}

fn append_field_children<ResolvePrimitiveSize>(
    symbol_tree_entries: &mut Vec<SymbolTreeEntry>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    root_symbol_key: &str,
    parent_node_key: &str,
    parent_full_path: &str,
    parent_promotion_display_name: &str,
    parent_locator: &ProjectRootSymbolLocator,
    data_type_ref: &DataTypeRef,
    container_type: ContainerType,
    depth: usize,
    expanded_tree_node_keys: &HashSet<String>,
    resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
    resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    visited_struct_layout_ids: &mut HashSet<String>,
) where
    ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
{
    match container_type {
        ContainerType::ArrayFixed(length) => {
            let element_size_in_bytes = resolve_data_type_size_in_bytes(
                project_symbol_catalog,
                data_type_ref,
                resolve_primitive_size_in_bytes,
                visited_struct_layout_ids,
            );

            for array_index in 0..length {
                let array_element_display_name = format!("[{}]", array_index);
                let array_element_full_path = format!("{}{}", parent_full_path, array_element_display_name);
                let array_element_promotion_display_name = format!("{}{}", parent_promotion_display_name, array_element_display_name);
                let array_element_node_key = format!("{}{}", parent_node_key, array_element_display_name);
                let array_element_locator = offset_locator(parent_locator, element_size_in_bytes.saturating_mul(array_index));
                let can_expand = data_type_ref_can_expand(project_symbol_catalog, data_type_ref, ContainerType::None, visited_struct_layout_ids);
                let is_expanded = can_expand && expanded_tree_node_keys.contains(&array_element_node_key);

                symbol_tree_entries.push(SymbolTreeEntry::new(
                    array_element_node_key.clone(),
                    SymbolTreeEntryKind::ArrayElement,
                    depth,
                    array_element_display_name.clone(),
                    array_element_full_path.clone(),
                    array_element_promotion_display_name.clone(),
                    root_symbol_key.to_string(),
                    array_element_locator.clone(),
                    data_type_ref.to_string(),
                    ContainerType::None,
                    can_expand,
                    is_expanded,
                ));

                if is_expanded {
                    if let Some(nested_struct_layout_definition) = resolve_struct_layout_definition(project_symbol_catalog, data_type_ref.get_data_type_id()) {
                        append_struct_field_entries(
                            symbol_tree_entries,
                            project_symbol_catalog,
                            root_symbol_key,
                            &array_element_node_key,
                            &array_element_full_path,
                            &array_element_promotion_display_name,
                            &array_element_locator,
                            &nested_struct_layout_definition,
                            depth + 1,
                            expanded_tree_node_keys,
                            resolved_pointer_targets_by_node_key,
                            resolve_primitive_size_in_bytes,
                            visited_struct_layout_ids,
                        );
                    }
                }
            }
        }
        ContainerType::None => {
            if let Some(nested_struct_layout_definition) = resolve_struct_layout_definition(project_symbol_catalog, data_type_ref.get_data_type_id()) {
                let type_identifier = data_type_ref.get_data_type_id().to_string();

                if !visited_struct_layout_ids.insert(type_identifier.clone()) {
                    return;
                }

                append_struct_field_entries(
                    symbol_tree_entries,
                    project_symbol_catalog,
                    root_symbol_key,
                    parent_node_key,
                    parent_full_path,
                    parent_promotion_display_name,
                    parent_locator,
                    &nested_struct_layout_definition,
                    depth,
                    expanded_tree_node_keys,
                    resolved_pointer_targets_by_node_key,
                    resolve_primitive_size_in_bytes,
                    visited_struct_layout_ids,
                );

                visited_struct_layout_ids.remove(&type_identifier);
            }
        }
        ContainerType::Pointer(_) | ContainerType::Pointer32 | ContainerType::Pointer64 => {
            let Some(resolved_pointer_target) = resolved_pointer_targets_by_node_key.get(parent_node_key) else {
                return;
            };
            let pointer_target_node_key = format!("{}::target", parent_node_key);
            let pointer_target_full_path = format!("{}.*", parent_full_path);
            let pointer_target_locator = resolved_pointer_target.get_target_locator().clone();
            let can_expand = data_type_ref_can_expand(project_symbol_catalog, data_type_ref, ContainerType::None, visited_struct_layout_ids);
            let is_expanded = can_expand && expanded_tree_node_keys.contains(&pointer_target_node_key);

            symbol_tree_entries.push(SymbolTreeEntry::new(
                pointer_target_node_key.clone(),
                SymbolTreeEntryKind::PointerTarget,
                depth,
                String::from("*"),
                pointer_target_full_path.clone(),
                parent_promotion_display_name.to_string(),
                root_symbol_key.to_string(),
                pointer_target_locator.clone(),
                data_type_ref.to_string(),
                ContainerType::None,
                can_expand,
                is_expanded,
            ));

            if is_expanded {
                append_field_children(
                    symbol_tree_entries,
                    project_symbol_catalog,
                    root_symbol_key,
                    &pointer_target_node_key,
                    &pointer_target_full_path,
                    parent_promotion_display_name,
                    &pointer_target_locator,
                    data_type_ref,
                    ContainerType::None,
                    depth + 1,
                    expanded_tree_node_keys,
                    resolved_pointer_targets_by_node_key,
                    resolve_primitive_size_in_bytes,
                    visited_struct_layout_ids,
                );
            }
        }
        ContainerType::Array => {}
    }
}

fn field_can_expand(
    project_symbol_catalog: &ProjectSymbolCatalog,
    field_definition: &SymbolicFieldDefinition,
) -> bool {
    data_type_ref_can_expand(
        project_symbol_catalog,
        field_definition.get_data_type_ref(),
        field_definition.get_container_type(),
        &mut HashSet::new(),
    )
}

fn data_type_ref_can_expand(
    project_symbol_catalog: &ProjectSymbolCatalog,
    data_type_ref: &DataTypeRef,
    container_type: ContainerType,
    visited_struct_layout_ids: &mut HashSet<String>,
) -> bool {
    match container_type {
        ContainerType::ArrayFixed(length) => length > 0,
        ContainerType::Pointer(_) | ContainerType::Pointer32 | ContainerType::Pointer64 => true,
        ContainerType::None => {
            let data_type_id = data_type_ref.get_data_type_id();

            if !visited_struct_layout_ids.insert(data_type_id.to_string()) {
                return false;
            }

            let can_expand = resolve_struct_layout_definition(project_symbol_catalog, data_type_id)
                .map(|struct_layout_definition| !struct_layout_definition.get_fields().is_empty())
                .unwrap_or(false);

            visited_struct_layout_ids.remove(data_type_id);

            can_expand
        }
        ContainerType::Array => false,
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
            struct_layout_definition
                .get_fields()
                .iter()
                .map(|field_definition| {
                    resolve_field_size_in_bytes(
                        project_symbol_catalog,
                        field_definition,
                        resolve_primitive_size_in_bytes,
                        visited_struct_layout_ids,
                    )
                })
                .sum::<u64>()
        })
        .or_else(|| resolve_primitive_size_in_bytes(data_type_ref))
        .unwrap_or(1);

    visited_struct_layout_ids.remove(data_type_id);

    size_in_bytes
}

fn offset_locator(
    project_root_symbol_locator: &ProjectRootSymbolLocator,
    offset: u64,
) -> ProjectRootSymbolLocator {
    match project_root_symbol_locator {
        ProjectRootSymbolLocator::AbsoluteAddress { address } => ProjectRootSymbolLocator::new_absolute_address(address.saturating_add(offset)),
        ProjectRootSymbolLocator::ModuleOffset {
            module_name,
            offset: base_offset,
        } => ProjectRootSymbolLocator::new_module_offset(module_name.clone(), base_offset.saturating_add(offset)),
    }
}

enum ResolvedRootSymbolType {
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

impl ResolvedRootSymbolType {
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
}

fn resolve_root_symbol_type(
    project_symbol_catalog: &ProjectSymbolCatalog,
    root_symbol_type_id: &str,
) -> ResolvedRootSymbolType {
    if let Some(struct_layout_descriptor) = resolve_exact_struct_layout_descriptor(project_symbol_catalog, root_symbol_type_id) {
        return ResolvedRootSymbolType::Struct {
            symbol_type_id: root_symbol_type_id.to_string(),
            struct_layout_definition: struct_layout_descriptor.get_struct_layout_definition().clone(),
        };
    }

    if let Ok(symbolic_field_definition) = SymbolicFieldDefinition::from_str(root_symbol_type_id) {
        return ResolvedRootSymbolType::Field {
            symbol_type_id: root_symbol_type_id.to_string(),
            data_type_ref: symbolic_field_definition.get_data_type_ref().clone(),
            container_type: symbolic_field_definition.get_container_type(),
        };
    }

    ResolvedRootSymbolType::Struct {
        symbol_type_id: root_symbol_type_id.to_string(),
        struct_layout_definition: SymbolicStructDefinition::from_str(root_symbol_type_id).unwrap_or_else(|_| SymbolicStructDefinition::new_anonymous(vec![])),
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolvedPointerTarget, SymbolTreeEntryKind, build_symbol_tree_entries};
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::{
            project_root_symbol::ProjectRootSymbol, project_root_symbol_locator::ProjectRootSymbolLocator, project_symbol_catalog::ProjectSymbolCatalog,
        },
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };
    use std::collections::{HashMap, HashSet};

    #[test]
    fn build_symbol_tree_entries_derives_nested_struct_and_array_children() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_rooted_symbols(
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
            vec![ProjectRootSymbol::new_absolute_address(
                String::from("sym.player"),
                String::from("Player"),
                0x100,
                String::from("player"),
            )],
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("root:sym.player"),
            String::from("root:sym.player::position"),
            String::from("root:sym.player::items"),
        ]);

        let symbol_tree_entries =
            build_symbol_tree_entries(
                &project_symbol_catalog,
                &expanded_tree_node_keys,
                &HashMap::new(),
                |data_type_ref| match data_type_ref.get_data_type_id() {
                    "u16" => Some(2),
                    "u32" => Some(4),
                    _ => None,
                },
            );

        assert_eq!(symbol_tree_entries.len(), 9);
        assert_eq!(symbol_tree_entries[0].get_display_name(), "Player");
        assert_eq!(
            symbol_tree_entries[0].get_kind(),
            &SymbolTreeEntryKind::RootedSymbol {
                symbol_key: String::from("sym.player"),
            }
        );
        assert_eq!(symbol_tree_entries[1].get_full_path(), "Player.health");
        assert_eq!(symbol_tree_entries[1].get_locator(), &ProjectRootSymbolLocator::new_absolute_address(0x100));
        assert_eq!(symbol_tree_entries[2].get_full_path(), "Player.position");
        assert_eq!(symbol_tree_entries[2].get_locator(), &ProjectRootSymbolLocator::new_absolute_address(0x104));
        assert_eq!(symbol_tree_entries[3].get_full_path(), "Player.position.x");
        assert_eq!(symbol_tree_entries[3].get_locator(), &ProjectRootSymbolLocator::new_absolute_address(0x104));
        assert_eq!(symbol_tree_entries[4].get_full_path(), "Player.position.y");
        assert_eq!(symbol_tree_entries[4].get_locator(), &ProjectRootSymbolLocator::new_absolute_address(0x108));
        assert_eq!(symbol_tree_entries[5].get_full_path(), "Player.items");
        assert_eq!(symbol_tree_entries[5].get_locator(), &ProjectRootSymbolLocator::new_absolute_address(0x10C));
        assert_eq!(symbol_tree_entries[6].get_full_path(), "Player.items[0]");
        assert_eq!(symbol_tree_entries[6].get_locator(), &ProjectRootSymbolLocator::new_absolute_address(0x10C));
        assert_eq!(symbol_tree_entries[7].get_full_path(), "Player.items[1]");
        assert_eq!(symbol_tree_entries[7].get_locator(), &ProjectRootSymbolLocator::new_absolute_address(0x10E));
        assert_eq!(symbol_tree_entries[8].get_full_path(), "Player.next");
        assert_eq!(symbol_tree_entries[8].can_expand(), true);
    }

    #[test]
    fn build_symbol_tree_entries_treats_primitive_root_type_as_leaf_node() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_rooted_symbols(
            Vec::new(),
            vec![ProjectRootSymbol::new_module_offset(
                String::from("sym.health"),
                String::from("Health"),
                String::from("game.exe"),
                0x1234,
                String::from("u32"),
            )],
        );
        let expanded_tree_node_keys = HashSet::from([String::from("root:sym.health")]);

        let symbol_tree_entries = build_symbol_tree_entries(&project_symbol_catalog, &expanded_tree_node_keys, &HashMap::new(), |data_type_ref| {
            (data_type_ref.get_data_type_id() == "u32").then_some(4)
        });

        assert_eq!(symbol_tree_entries.len(), 1);
        assert_eq!(symbol_tree_entries[0].get_symbol_type_id(), "u32");
        assert_eq!(symbol_tree_entries[0].get_container_type(), ContainerType::None);
        assert_eq!(symbol_tree_entries[0].can_expand(), false);
    }

    #[test]
    fn build_symbol_tree_entries_derives_pointer_target_children_from_resolved_targets() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_rooted_symbols(
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
            vec![ProjectRootSymbol::new_absolute_address(
                String::from("sym.player"),
                String::from("Player"),
                0x100,
                String::from("player"),
            )],
        );
        let expanded_tree_node_keys = HashSet::from([
            String::from("root:sym.player"),
            String::from("root:sym.player::next"),
            String::from("root:sym.player::next::target"),
        ]);
        let resolved_pointer_targets_by_node_key = HashMap::from([(
            String::from("root:sym.player::next"),
            ResolvedPointerTarget::new(ProjectRootSymbolLocator::new_absolute_address(0x200), String::from("0x100 -> 0x200")),
        )]);

        let symbol_tree_entries = build_symbol_tree_entries(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &resolved_pointer_targets_by_node_key,
            |data_type_ref| (data_type_ref.get_data_type_id() == "u32").then_some(4),
        );

        assert_eq!(symbol_tree_entries.len(), 6);
        assert_eq!(symbol_tree_entries[2].get_full_path(), "Player.next");
        assert_eq!(
            symbol_tree_entries[2].get_container_type(),
            ContainerType::Pointer(PointerScanPointerSize::Pointer64)
        );
        assert_eq!(symbol_tree_entries[3].get_kind(), &SymbolTreeEntryKind::PointerTarget);
        assert_eq!(symbol_tree_entries[3].get_full_path(), "Player.next.*");
        assert_eq!(symbol_tree_entries[3].get_locator(), &ProjectRootSymbolLocator::new_absolute_address(0x200));
        assert_eq!(symbol_tree_entries[4].get_full_path(), "Player.next.*.health");
        assert_eq!(symbol_tree_entries[4].get_locator(), &ProjectRootSymbolLocator::new_absolute_address(0x200));
        assert_eq!(symbol_tree_entries[5].get_full_path(), "Player.next.*.next");
    }
}
