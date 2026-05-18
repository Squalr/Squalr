use crate::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use crate::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
use crate::structures::projects::project_symbol_claim::ProjectSymbolClaim;
use crate::structures::projects::project_symbol_locator::ProjectSymbolLocator;
use crate::structures::projects::project_symbol_module::ProjectSymbolModule;
use crate::structures::projects::project_symbol_module_field::ProjectSymbolModuleField;
use crate::structures::projects::symbol_layouts::symbol_layout_descriptor_builder::SymbolLayoutDescriptorBuilder;
use crate::structures::projects::symbol_layouts::symbol_layout_field_materializer::{SymbolLayoutFieldMaterializer, SymbolLayoutPositionedField};
use crate::structures::structs::symbolic_field_definition::SymbolicFieldDefinition;
use crate::structures::structs::symbolic_field_definition::SymbolicFieldOffsetResolution;
use crate::structures::structs::symbolic_struct_definition::SymbolicStructDefinition;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectSymbolCatalog {
    #[serde(default)]
    symbol_modules: Vec<ProjectSymbolModule>,
    #[serde(default)]
    struct_layout_descriptors: Vec<StructLayoutDescriptor>,
    #[serde(default)]
    symbolic_resolver_descriptors: Vec<SymbolicResolverDescriptor>,
    #[serde(default)]
    symbol_claims: Vec<ProjectSymbolClaim>,
}

impl ProjectSymbolCatalog {
    pub fn new(struct_layout_descriptors: Vec<StructLayoutDescriptor>) -> Self {
        Self::new_with_modules_and_symbol_claims(Vec::new(), struct_layout_descriptors, Vec::new())
    }

    pub fn new_with_symbol_claims(
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
        symbol_claims: Vec<ProjectSymbolClaim>,
    ) -> Self {
        Self::new_with_modules_and_symbol_claims(Vec::new(), struct_layout_descriptors, symbol_claims)
    }

    pub fn new_with_modules_and_symbol_claims(
        symbol_modules: Vec<ProjectSymbolModule>,
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
        symbol_claims: Vec<ProjectSymbolClaim>,
    ) -> Self {
        Self::new_with_modules_resolvers_and_symbol_claims(symbol_modules, struct_layout_descriptors, Vec::new(), symbol_claims)
    }

    pub fn new_with_modules_resolvers_and_symbol_claims(
        symbol_modules: Vec<ProjectSymbolModule>,
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
        symbolic_resolver_descriptors: Vec<SymbolicResolverDescriptor>,
        symbol_claims: Vec<ProjectSymbolClaim>,
    ) -> Self {
        Self {
            symbol_modules,
            struct_layout_descriptors,
            symbolic_resolver_descriptors,
            symbol_claims,
        }
    }

    pub fn get_symbol_modules(&self) -> &[ProjectSymbolModule] {
        &self.symbol_modules
    }

    pub fn get_symbol_modules_mut(&mut self) -> &mut Vec<ProjectSymbolModule> {
        &mut self.symbol_modules
    }

    pub fn find_symbol_module(
        &self,
        module_name: &str,
    ) -> Option<&ProjectSymbolModule> {
        self.symbol_modules
            .iter()
            .find(|symbol_module| symbol_module.get_module_name() == module_name)
    }

    pub fn find_symbol_module_mut(
        &mut self,
        module_name: &str,
    ) -> Option<&mut ProjectSymbolModule> {
        self.symbol_modules
            .iter_mut()
            .find(|symbol_module| symbol_module.get_module_name() == module_name)
    }

    pub fn find_module_field(
        &self,
        symbol_locator_key: &str,
    ) -> Option<(&ProjectSymbolModule, &ProjectSymbolModuleField)> {
        let ProjectSymbolLocator::ModuleOffset { module_name, offset } = parse_symbol_locator_key(symbol_locator_key)? else {
            return None;
        };
        let symbol_module = self.find_symbol_module(&module_name)?;
        let module_field = symbol_module.find_field(offset)?;

        Some((symbol_module, module_field))
    }

    pub fn find_module_field_offset_by_display_name(
        &self,
        module_name: &str,
        display_name: &str,
    ) -> Option<u64> {
        let symbol_module = self.find_symbol_module(module_name)?;

        symbol_module
            .get_fields()
            .iter()
            .find(|module_field| module_field.get_display_name() == display_name)
            .map(ProjectSymbolModuleField::get_offset)
    }

    pub fn find_module_symbol_offset_by_display_name(
        &self,
        module_name: &str,
        display_name: &str,
    ) -> Option<u64> {
        if let Some(module_field_offset) = self.find_module_field_offset_by_display_name(module_name, display_name) {
            return Some(module_field_offset);
        }

        self.symbol_claims.iter().find_map(|symbol_claim| {
            if symbol_claim.get_display_name() != display_name {
                return None;
            }

            let ProjectSymbolLocator::ModuleOffset {
                module_name: claim_module_name,
                offset,
            } = symbol_claim.get_locator()
            else {
                return None;
            };

            (claim_module_name == module_name).then_some(*offset)
        })
    }

    pub fn find_module_field_mut(
        &mut self,
        symbol_locator_key: &str,
    ) -> Option<&mut ProjectSymbolModuleField> {
        let ProjectSymbolLocator::ModuleOffset { module_name, offset } = parse_symbol_locator_key(symbol_locator_key)? else {
            return None;
        };
        let symbol_module = self.find_symbol_module_mut(&module_name)?;

        symbol_module.find_field_mut(offset)
    }

    pub fn ensure_symbol_module(
        &mut self,
        module_name: &str,
        minimum_size: u64,
    ) {
        if module_name.trim().is_empty() {
            return;
        }

        if let Some(symbol_module) = self.find_symbol_module_mut(module_name) {
            if symbol_module.get_size() < minimum_size {
                symbol_module.set_size(minimum_size);
            }

            return;
        }

        self.symbol_modules
            .push(ProjectSymbolModule::new(module_name.to_string(), minimum_size));
    }

    pub fn get_struct_layout_descriptors(&self) -> &[StructLayoutDescriptor] {
        &self.struct_layout_descriptors
    }

    pub fn contains_struct_layout_id(
        &self,
        struct_layout_id: &str,
    ) -> bool {
        self.struct_layout_descriptors
            .iter()
            .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == struct_layout_id)
    }

    pub fn ensure_module_root_struct_layout(
        &mut self,
        module_name: &str,
        module_size_in_bytes: u64,
    ) {
        let module_name = module_name.trim();
        if module_name.is_empty() {
            return;
        }

        if let Some(module_struct_layout_position) = self
            .struct_layout_descriptors
            .iter()
            .position(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == module_name)
        {
            let updated_struct_layout_definition = self.struct_layout_descriptors[module_struct_layout_position]
                .get_struct_layout_definition()
                .clone()
                .with_declared_size_in_bytes(Some(module_size_in_bytes));
            let updated_struct_layout_definition = self.materialize_struct_layout_definition(updated_struct_layout_definition);
            self.struct_layout_descriptors[module_struct_layout_position] =
                StructLayoutDescriptor::new(module_name.to_string(), updated_struct_layout_definition);
        } else {
            let struct_layout_definition = self.materialize_struct_layout_definition(
                SymbolicStructDefinition::new(module_name.to_string(), Vec::new()).with_declared_size_in_bytes(Some(module_size_in_bytes)),
            );
            self.struct_layout_descriptors
                .push(StructLayoutDescriptor::new(module_name.to_string(), struct_layout_definition));
        }

        self.sort_struct_layout_descriptors_by_id();
    }

    fn materialize_struct_layout_definition(
        &self,
        symbolic_struct_definition: SymbolicStructDefinition,
    ) -> SymbolicStructDefinition {
        let mut positioned_fields = Vec::new();
        let mut next_sequential_offset = 0_u64;

        for symbolic_field_definition in symbolic_struct_definition.get_fields() {
            if symbolic_field_definition.is_unassigned() {
                next_sequential_offset = next_sequential_offset.saturating_add(
                    symbolic_field_definition
                        .get_unassigned_size_in_bytes()
                        .unwrap_or(0),
                );
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
            let field_size_in_bytes = SymbolLayoutDescriptorBuilder::resolve_symbolic_field_size_in_bytes(self, symbolic_field_definition, &mut HashSet::new());

            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
            positioned_fields.push(SymbolLayoutPositionedField::new(
                field_offset,
                field_size_in_bytes,
                symbolic_field_definition.clone(),
            ));
        }

        let Ok(materialized_fields) = SymbolLayoutFieldMaterializer::materialize_positioned_fields(
            symbolic_struct_definition.get_layout_kind(),
            symbolic_struct_definition.get_declared_size_in_bytes(),
            positioned_fields,
        ) else {
            return symbolic_struct_definition;
        };

        SymbolicStructDefinition::new_with_layout_kind(
            symbolic_struct_definition.get_symbol_namespace().to_string(),
            symbolic_struct_definition.get_layout_kind(),
            materialized_fields,
        )
        .with_declared_size_in_bytes(symbolic_struct_definition.get_declared_size_in_bytes())
    }

    pub fn rename_module_root_struct_layout(
        &mut self,
        old_module_name: &str,
        new_module_name: &str,
    ) -> Result<(), String> {
        let old_module_name = old_module_name.trim();
        let new_module_name = new_module_name.trim();
        if old_module_name.is_empty() || new_module_name.is_empty() || old_module_name == new_module_name {
            return Ok(());
        }

        let Some(old_struct_layout_position) = self
            .struct_layout_descriptors
            .iter()
            .position(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == old_module_name)
        else {
            return Ok(());
        };

        if self
            .struct_layout_descriptors
            .iter()
            .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == new_module_name)
        {
            return Err(format!(
                "Cannot rename module root layout '{}' to '{}': the destination layout already exists.",
                old_module_name, new_module_name
            ));
        }

        let old_struct_layout_descriptor = self
            .struct_layout_descriptors
            .remove(old_struct_layout_position);
        let old_struct_layout_definition = old_struct_layout_descriptor.get_struct_layout_definition();
        let renamed_struct_layout_definition = SymbolicStructDefinition::new_with_layout_kind(
            new_module_name.to_string(),
            old_struct_layout_definition.get_layout_kind(),
            old_struct_layout_definition.get_fields().to_vec(),
        )
        .with_declared_size_in_bytes(old_struct_layout_definition.get_declared_size_in_bytes());

        self.struct_layout_descriptors
            .push(StructLayoutDescriptor::new(new_module_name.to_string(), renamed_struct_layout_definition));
        self.sort_struct_layout_descriptors_by_id();

        Ok(())
    }

    pub fn delete_module_root_struct_layouts(
        &mut self,
        module_names: &HashSet<String>,
    ) -> u64 {
        let struct_layout_count_before_delete = self.struct_layout_descriptors.len();
        self.struct_layout_descriptors
            .retain(|struct_layout_descriptor| !module_names.contains(struct_layout_descriptor.get_struct_layout_id()));

        struct_layout_count_before_delete.saturating_sub(self.struct_layout_descriptors.len()) as u64
    }

    pub fn set_struct_layout_descriptors(
        &mut self,
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
    ) {
        self.struct_layout_descriptors = struct_layout_descriptors;
    }

    pub fn get_symbolic_resolver_descriptors(&self) -> &[SymbolicResolverDescriptor] {
        &self.symbolic_resolver_descriptors
    }

    pub fn set_symbolic_resolver_descriptors(
        &mut self,
        symbolic_resolver_descriptors: Vec<SymbolicResolverDescriptor>,
    ) {
        self.symbolic_resolver_descriptors = symbolic_resolver_descriptors;
    }

    pub fn find_symbolic_resolver_descriptor(
        &self,
        resolver_id: &str,
    ) -> Option<&SymbolicResolverDescriptor> {
        self.symbolic_resolver_descriptors
            .iter()
            .find(|resolver_descriptor| resolver_descriptor.get_resolver_id() == resolver_id)
    }

    pub fn get_symbol_claims(&self) -> &[ProjectSymbolClaim] {
        &self.symbol_claims
    }

    pub fn find_symbol_claim(
        &self,
        symbol_locator_key: &str,
    ) -> Option<&ProjectSymbolClaim> {
        self.symbol_claims
            .iter()
            .find(|symbol_claim| symbol_claim.get_symbol_locator_key() == symbol_locator_key)
    }

    pub fn resolve_symbol_claim(
        &self,
        symbol_locator_key: &str,
    ) -> Option<ProjectSymbolClaim> {
        if let Some(symbol_claim) = self.find_symbol_claim(symbol_locator_key) {
            return Some(symbol_claim.clone());
        }

        let (symbol_module, module_field) = self.find_module_field(symbol_locator_key)?;

        Some(ProjectSymbolClaim::new_module_offset(
            module_field.get_display_name().to_string(),
            symbol_module.get_module_name().to_string(),
            module_field.get_offset(),
            module_field.get_struct_layout_id().to_string(),
        ))
    }

    pub fn find_symbol_claim_mut(
        &mut self,
        symbol_locator_key: &str,
    ) -> Option<&mut ProjectSymbolClaim> {
        self.symbol_claims
            .iter_mut()
            .find(|symbol_claim| symbol_claim.get_symbol_locator_key() == symbol_locator_key)
    }

    pub fn get_symbol_claims_mut(&mut self) -> &mut Vec<ProjectSymbolClaim> {
        &mut self.symbol_claims
    }

    pub fn set_symbol_claims(
        &mut self,
        symbol_claims: Vec<ProjectSymbolClaim>,
    ) {
        self.symbol_claims = symbol_claims;
    }

    pub fn is_empty(&self) -> bool {
        self.symbol_modules.is_empty()
            && self.struct_layout_descriptors.is_empty()
            && self.symbolic_resolver_descriptors.is_empty()
            && self.symbol_claims.is_empty()
    }

    pub fn validate_local_resolver_dependencies(&self) -> Result<(), String> {
        for struct_layout_descriptor in &self.struct_layout_descriptors {
            self.validate_local_resolver_dependencies_for_struct_layout(struct_layout_descriptor)?;
        }

        Ok(())
    }

    pub fn validate_local_resolver_dependencies_for_struct_layout(
        &self,
        struct_layout_descriptor: &StructLayoutDescriptor,
    ) -> Result<(), String> {
        let fields = struct_layout_descriptor
            .get_struct_layout_definition()
            .get_fields();
        let field_names = fields
            .iter()
            .filter_map(|field_definition| {
                let field_name = field_definition.get_field_name();

                (!field_name.is_empty()).then_some(field_name.to_string())
            })
            .collect::<HashSet<_>>();
        let mut dependencies_by_field_name = HashMap::new();

        for field_definition in fields {
            let field_name = field_definition.get_field_name();
            if field_name.is_empty() {
                continue;
            }

            let dependencies = self.collect_local_field_layout_dependencies(field_definition, &field_names);
            dependencies_by_field_name.insert(field_name.to_string(), dependencies);
        }

        let mut visiting_field_names = HashSet::new();
        let mut visited_field_names = HashSet::new();
        let mut dependency_stack = Vec::new();

        for field_name in dependencies_by_field_name.keys() {
            if let Some(cycle_path) = Self::find_local_dependency_cycle(
                field_name,
                &dependencies_by_field_name,
                &mut visiting_field_names,
                &mut visited_field_names,
                &mut dependency_stack,
            ) {
                return Err(format!(
                    "Struct `{}` field layout resolvers contain a dependency cycle: {}.",
                    struct_layout_descriptor.get_struct_layout_id(),
                    cycle_path.join(" -> ")
                ));
            }
        }

        Ok(())
    }

    fn collect_local_field_layout_dependencies(
        &self,
        field_definition: &SymbolicFieldDefinition,
        field_names: &HashSet<String>,
    ) -> Vec<String> {
        let mut dependencies = Vec::new();

        dependencies.extend(self.collect_local_field_dependencies_from_resolver(field_definition.get_count_resolution().as_resolver_id()));
        dependencies.extend(self.collect_local_field_dependencies_from_resolver(field_definition.get_display_count_resolution().as_resolver_id()));
        dependencies.extend(self.collect_local_field_dependencies_from_resolver(field_definition.get_offset_resolution().as_resolver_id()));
        dependencies.extend(
            self.collect_local_field_dependencies_from_resolver(
                field_definition
                    .get_active_when_resolver()
                    .map(|resolver_ref| resolver_ref.get_resolver_id()),
            ),
        );
        dependencies.retain(|dependency| field_names.contains(dependency));
        dependencies.sort();
        dependencies.dedup();

        dependencies
    }

    fn collect_local_field_dependencies_from_resolver(
        &self,
        resolver_id: Option<&str>,
    ) -> Vec<String> {
        resolver_id
            .and_then(|resolver_id| self.find_symbolic_resolver_descriptor(resolver_id))
            .map(|resolver_descriptor| {
                resolver_descriptor
                    .get_resolver_definition()
                    .referenced_local_fields()
            })
            .unwrap_or_default()
    }

    fn find_local_dependency_cycle(
        field_name: &str,
        dependencies_by_field_name: &HashMap<String, Vec<String>>,
        visiting_field_names: &mut HashSet<String>,
        visited_field_names: &mut HashSet<String>,
        dependency_stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if visited_field_names.contains(field_name) {
            return None;
        }

        if visiting_field_names.contains(field_name) {
            let cycle_start_dependency_index = dependency_stack
                .iter()
                .position(|dependency_field_name| dependency_field_name == field_name)
                .unwrap_or(0);
            let mut cycle_path = dependency_stack[cycle_start_dependency_index..].to_vec();
            cycle_path.push(field_name.to_string());

            return Some(cycle_path);
        }

        visiting_field_names.insert(field_name.to_string());
        dependency_stack.push(field_name.to_string());

        if let Some(dependencies) = dependencies_by_field_name.get(field_name) {
            for dependency in dependencies {
                if let Some(cycle_path) = Self::find_local_dependency_cycle(
                    dependency,
                    dependencies_by_field_name,
                    visiting_field_names,
                    visited_field_names,
                    dependency_stack,
                ) {
                    return Some(cycle_path);
                }
            }
        }

        dependency_stack.pop();
        visiting_field_names.remove(field_name);
        visited_field_names.insert(field_name.to_string());

        None
    }

    fn sort_struct_layout_descriptors_by_id(&mut self) {
        self.struct_layout_descriptors
            .sort_by(|left_layout, right_layout| {
                left_layout
                    .get_struct_layout_id()
                    .to_ascii_lowercase()
                    .cmp(&right_layout.get_struct_layout_id().to_ascii_lowercase())
            });
    }
}

fn parse_symbol_locator_key(symbol_locator_key: &str) -> Option<ProjectSymbolLocator> {
    if let Some(address_text) = symbol_locator_key.strip_prefix("absolute:") {
        let address = u64::from_str_radix(address_text, 16).ok()?;

        return Some(ProjectSymbolLocator::new_absolute_address(address));
    }

    let module_locator_text = symbol_locator_key.strip_prefix("module:")?;
    let (module_name, offset_text) = module_locator_text.rsplit_once(':')?;
    let offset = u64::from_str_radix(offset_text, 16).ok()?;

    Some(ProjectSymbolLocator::new_module_offset(module_name.to_string(), offset))
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolCatalog;
    use crate::registries::symbols::{struct_layout_descriptor::StructLayoutDescriptor, symbolic_resolver_descriptor::SymbolicResolverDescriptor};
    use crate::structures::structs::{
        symbolic_field_definition::SymbolicFieldDefinition,
        symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverNode},
        symbolic_struct_definition::SymbolicStructDefinition,
    };
    use std::str::FromStr;

    #[test]
    fn catalog_stores_symbolic_resolver_descriptors() {
        let resolver_descriptor = SymbolicResolverDescriptor::new(
            String::from("inventory.count"),
            SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("count"))),
        );
        let project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(Vec::new(), Vec::new(), vec![resolver_descriptor], Vec::new());

        let found_resolver_descriptor = project_symbol_catalog
            .find_symbolic_resolver_descriptor("inventory.count")
            .expect("Expected resolver descriptor.");

        assert_eq!(found_resolver_descriptor.get_resolver_id(), "inventory.count");
    }

    #[test]
    fn deserializes_legacy_catalog_with_missing_additive_fields() {
        let project_symbol_catalog: ProjectSymbolCatalog =
            serde_json::from_str(r#"{"struct_layout_descriptors":[]}"#).expect("Expected legacy project symbol catalog to deserialize.");

        assert!(project_symbol_catalog.get_symbol_modules().is_empty());
        assert!(
            project_symbol_catalog
                .get_struct_layout_descriptors()
                .is_empty()
        );
        assert!(
            project_symbol_catalog
                .get_symbolic_resolver_descriptors()
                .is_empty()
        );
        assert!(project_symbol_catalog.get_symbol_claims().is_empty());
    }

    #[test]
    fn empty_catalog_considers_resolvers() {
        let mut project_symbol_catalog = ProjectSymbolCatalog::default();

        assert!(project_symbol_catalog.is_empty());

        project_symbol_catalog.set_symbolic_resolver_descriptors(vec![SymbolicResolverDescriptor::new(
            String::from("inventory.count"),
            SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(1)),
        )]);

        assert!(!project_symbol_catalog.is_empty());
    }

    #[test]
    fn validate_local_resolver_dependencies_accepts_acyclic_resolver_field_dependencies() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            Vec::new(),
            vec![StructLayoutDescriptor::new(
                String::from("inventory"),
                SymbolicStructDefinition::new(
                    String::from("inventory"),
                    vec![
                        SymbolicFieldDefinition::from_str("count:u32").expect("Expected count field."),
                        SymbolicFieldDefinition::from_str("items:u16[resolver(inventory.item_count)]").expect("Expected items field."),
                    ],
                ),
            )],
            vec![SymbolicResolverDescriptor::new(
                String::from("inventory.item_count"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("count"))),
            )],
            Vec::new(),
        );

        assert!(
            project_symbol_catalog
                .validate_local_resolver_dependencies()
                .is_ok()
        );
    }

    #[test]
    fn validate_local_resolver_dependencies_rejects_resolver_field_cycles() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            Vec::new(),
            vec![StructLayoutDescriptor::new(
                String::from("cycle"),
                SymbolicStructDefinition::new(
                    String::from("cycle"),
                    vec![
                        SymbolicFieldDefinition::from_str("left:u8[resolver(read_right)]").expect("Expected left field."),
                        SymbolicFieldDefinition::from_str("right:u8[resolver(read_left)]").expect("Expected right field."),
                    ],
                ),
            )],
            vec![
                SymbolicResolverDescriptor::new(
                    String::from("read_right"),
                    SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("right"))),
                ),
                SymbolicResolverDescriptor::new(
                    String::from("read_left"),
                    SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("left"))),
                ),
            ],
            Vec::new(),
        );

        let validation_result = project_symbol_catalog.validate_local_resolver_dependencies();

        assert!(validation_result.is_err_and(|error| error.contains("left") && error.contains("right")));
    }
}
