use squalr_engine_api::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;

pub struct ProjectSymbolResolverMutation;

impl ProjectSymbolResolverMutation {
    pub fn upsert_resolver_descriptor(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        original_resolver_id: Option<&str>,
        resolver_descriptor: SymbolicResolverDescriptor,
    ) -> Result<(), String> {
        let resolver_id = resolver_descriptor.get_resolver_id();
        let conflicts_with_existing_resolver = original_resolver_id.is_some_and(|original_resolver_id| {
            original_resolver_id != resolver_id
                && project_symbol_catalog
                    .get_symbolic_resolver_descriptors()
                    .iter()
                    .any(|existing_resolver_descriptor| existing_resolver_descriptor.get_resolver_id() == resolver_id)
        });

        if conflicts_with_existing_resolver {
            return Err(format!("Resolver id `{}` is already used.", resolver_id));
        }

        let replacement_resolver_id = resolver_id.to_string();
        let mut resolver_descriptors = project_symbol_catalog
            .get_symbolic_resolver_descriptors()
            .iter()
            .filter(|existing_resolver_descriptor| {
                Some(existing_resolver_descriptor.get_resolver_id()) != original_resolver_id
                    && existing_resolver_descriptor.get_resolver_id() != replacement_resolver_id
            })
            .cloned()
            .collect::<Vec<_>>();

        resolver_descriptors.push(resolver_descriptor);
        Self::sort_resolver_descriptors(&mut resolver_descriptors);
        project_symbol_catalog.set_symbolic_resolver_descriptors(resolver_descriptors);
        project_symbol_catalog.validate_local_resolver_dependencies()
    }

    pub fn delete_resolver(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        resolver_id: &str,
    ) -> Result<(), String> {
        let resolver_count_before_delete = project_symbol_catalog.get_symbolic_resolver_descriptors().len();
        let resolver_descriptors = project_symbol_catalog
            .get_symbolic_resolver_descriptors()
            .iter()
            .filter(|resolver_descriptor| resolver_descriptor.get_resolver_id() != resolver_id)
            .cloned()
            .collect::<Vec<_>>();

        if resolver_descriptors.len() == resolver_count_before_delete {
            return Err(format!("Resolver `{}` does not exist.", resolver_id));
        }

        project_symbol_catalog.set_symbolic_resolver_descriptors(resolver_descriptors);
        project_symbol_catalog.validate_local_resolver_dependencies()
    }

    fn sort_resolver_descriptors(resolver_descriptors: &mut [SymbolicResolverDescriptor]) {
        resolver_descriptors.sort_by(|left_resolver, right_resolver| {
            left_resolver
                .get_resolver_id()
                .to_ascii_lowercase()
                .cmp(&right_resolver.get_resolver_id().to_ascii_lowercase())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolResolverMutation;
    use squalr_engine_api::registries::symbols::{struct_layout_descriptor::StructLayoutDescriptor, symbolic_resolver_descriptor::SymbolicResolverDescriptor};
    use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
    use squalr_engine_api::structures::structs::{
        symbolic_field_definition::SymbolicFieldDefinition,
        symbolic_resolver_definition::{SymbolicResolverDefinition, SymbolicResolverNode},
        symbolic_struct_definition::SymbolicStructDefinition,
    };
    use std::str::FromStr;

    #[test]
    fn upsert_resolver_descriptor_sorts_resolvers() {
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            Vec::new(),
            Vec::new(),
            vec![SymbolicResolverDescriptor::new(
                String::from("z.count"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(2)),
            )],
            Vec::new(),
        );

        ProjectSymbolResolverMutation::upsert_resolver_descriptor(
            &mut project_symbol_catalog,
            None,
            SymbolicResolverDescriptor::new(String::from("a.count"), SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(1))),
        )
        .expect("Expected resolver upsert to succeed.");

        assert_eq!(project_symbol_catalog.get_symbolic_resolver_descriptors()[0].get_resolver_id(), "a.count");
        assert_eq!(project_symbol_catalog.get_symbolic_resolver_descriptors()[1].get_resolver_id(), "z.count");
    }

    #[test]
    fn delete_resolver_removes_descriptor() {
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            Vec::new(),
            Vec::new(),
            vec![SymbolicResolverDescriptor::new(
                String::from("inventory.count"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_literal(2)),
            )],
            Vec::new(),
        );

        ProjectSymbolResolverMutation::delete_resolver(&mut project_symbol_catalog, "inventory.count").expect("Expected resolver delete to succeed.");

        assert!(
            project_symbol_catalog
                .get_symbolic_resolver_descriptors()
                .is_empty()
        );
    }

    #[test]
    fn upsert_resolver_descriptor_rejects_local_resolver_field_cycles() {
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_resolvers_and_symbol_claims(
            Vec::new(),
            vec![StructLayoutDescriptor::new(
                String::from("cycle"),
                SymbolicStructDefinition::new(
                    String::from("cycle"),
                    vec![
                        SymbolicFieldDefinition::from_str("left:u8[resolver(read_right)]").expect("Expected left field to parse."),
                        SymbolicFieldDefinition::from_str("right:u8[resolver(read_left)]").expect("Expected right field to parse."),
                    ],
                ),
            )],
            vec![SymbolicResolverDescriptor::new(
                String::from("read_right"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("right"))),
            )],
            Vec::new(),
        );

        let result = ProjectSymbolResolverMutation::upsert_resolver_descriptor(
            &mut project_symbol_catalog,
            None,
            SymbolicResolverDescriptor::new(
                String::from("read_left"),
                SymbolicResolverDefinition::new(SymbolicResolverNode::new_local_field(String::from("left"))),
            ),
        );

        assert!(result.is_err_and(|error| error.contains("dependency cycle")));
    }
}
