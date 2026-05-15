use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    memory::symbolic_pointer_chain::SymbolicPointerChain,
    projects::{
        project_symbol_catalog::ProjectSymbolCatalog,
        project_symbol_locator::ProjectSymbolLocator,
        symbol_tree::symbol_tree_node::{
            ResolvedPointerTarget, SymbolTreeNode, build_symbol_tree_nodes, build_symbol_tree_nodes_with_scalar_reader,
            build_symbol_tree_nodes_with_scalar_reader_and_pointer_chains,
        },
    },
    structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_resolver_definition::SymbolicResolverEvaluationError},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolTree {
    nodes: Vec<SymbolTreeNode>,
}

impl SymbolTree {
    pub fn new(nodes: Vec<SymbolTreeNode>) -> Self {
        Self { nodes }
    }

    pub fn build<ResolvePrimitiveSize>(
        project_symbol_catalog: &ProjectSymbolCatalog,
        expanded_tree_node_keys: &HashSet<String>,
        resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
        resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
    ) -> Self
    where
        ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
    {
        Self::new(build_symbol_tree_nodes(
            project_symbol_catalog,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
        ))
    }

    pub fn build_with_scalar_reader<ResolvePrimitiveSize, ReadScalarField>(
        project_symbol_catalog: &ProjectSymbolCatalog,
        expanded_tree_node_keys: &HashSet<String>,
        resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
        resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
        read_scalar_field: ReadScalarField,
    ) -> Self
    where
        ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
        ReadScalarField: Fn(&ProjectSymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String> + Copy,
    {
        Self::new(build_symbol_tree_nodes_with_scalar_reader(
            project_symbol_catalog,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
        ))
    }

    pub fn build_with_scalar_reader_and_pointer_chains<ResolvePrimitiveSize, ReadScalarField, ResolveRelativePointerChain, ResolveGlobalPointerChain>(
        project_symbol_catalog: &ProjectSymbolCatalog,
        expanded_tree_node_keys: &HashSet<String>,
        resolved_pointer_targets_by_node_key: &HashMap<String, ResolvedPointerTarget>,
        resolve_primitive_size_in_bytes: ResolvePrimitiveSize,
        read_scalar_field: ReadScalarField,
        resolve_relative_pointer_chain: ResolveRelativePointerChain,
        resolve_global_pointer_chain: ResolveGlobalPointerChain,
    ) -> Self
    where
        ResolvePrimitiveSize: Fn(&DataTypeRef) -> Option<u64> + Copy,
        ReadScalarField: Fn(&ProjectSymbolLocator, &SymbolicFieldDefinition, u64) -> Result<Option<i128>, String> + Copy,
        ResolveRelativePointerChain: Fn(&ProjectSymbolLocator, &SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
        ResolveGlobalPointerChain: Fn(&SymbolicPointerChain) -> Result<i128, SymbolicResolverEvaluationError> + Copy,
    {
        Self::new(build_symbol_tree_nodes_with_scalar_reader_and_pointer_chains(
            project_symbol_catalog,
            expanded_tree_node_keys,
            resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
        ))
    }

    pub fn get_nodes(&self) -> &[SymbolTreeNode] {
        &self.nodes
    }

    pub fn into_nodes(self) -> Vec<SymbolTreeNode> {
        self.nodes
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolTree;
    use crate::structures::projects::{project_symbol_catalog::ProjectSymbolCatalog, project_symbol_module::ProjectSymbolModule};
    use std::collections::{HashMap, HashSet};

    #[test]
    fn build_wraps_project_symbol_catalog_nodes() {
        let project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![ProjectSymbolModule::new(String::from("game.exe"), 0x1000)], Vec::new(), Vec::new());
        let symbol_tree = SymbolTree::build(&project_symbol_catalog, &HashSet::new(), &HashMap::new(), |_| None);

        assert_eq!(symbol_tree.get_nodes().len(), 1);
        assert_eq!(symbol_tree.get_nodes()[0].get_display_name(), "game.exe");
    }
}
