use crate::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    project_symbol_claim::ProjectSymbolClaim,
    symbol_tree::symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind},
};

pub fn build_symbol_layout_edit_target(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_tree_nodes: &[SymbolTreeNode],
    symbol_tree_node: &SymbolTreeNode,
) -> Option<String> {
    let layout_exists = |struct_layout_id: &str| {
        project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == struct_layout_id)
    };

    let symbol_type_id = symbol_tree_node.get_symbol_type_id();
    if layout_exists(symbol_type_id) {
        return Some(symbol_type_id.to_string());
    }

    if let SymbolTreeNodeKind::ModuleSpace { module_name, .. } = symbol_tree_node.get_kind() {
        return layout_exists(module_name).then(|| module_name.to_string());
    }

    let mut ancestor_node_key = symbol_tree_node.get_node_key();
    while let Some((next_ancestor_node_key, _field_node_key)) = ancestor_node_key.rsplit_once("::") {
        if let Some(ancestor_symbol_tree_node) = symbol_tree_nodes
            .iter()
            .find(|ancestor_symbol_tree_node| ancestor_symbol_tree_node.get_node_key() == next_ancestor_node_key)
        {
            let ancestor_symbol_type_id = ancestor_symbol_tree_node.get_symbol_type_id();
            if layout_exists(ancestor_symbol_type_id) {
                return Some(ancestor_symbol_type_id.to_string());
            }
        }

        ancestor_node_key = next_ancestor_node_key;
    }

    let module_name = symbol_tree_node.get_locator().get_focus_module_name();
    if !module_name.trim().is_empty() && layout_exists(module_name) {
        return Some(module_name.to_string());
    }

    let root_symbol_claim_type_id = project_symbol_catalog
        .get_symbol_claims()
        .iter()
        .find(|symbol_claim| symbol_claim.get_symbol_locator_key() == symbol_tree_node.get_symbol_claim_locator_key())
        .map(ProjectSymbolClaim::get_struct_layout_id);

    root_symbol_claim_type_id
        .filter(|struct_layout_id| layout_exists(struct_layout_id))
        .map(str::to_string)
}
