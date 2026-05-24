use crate::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use crate::structures::memory::pointer_chain_segment::PointerChainSegment;
use crate::structures::projects::symbol_tree::symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddSymbolToProjectTarget {
    pub project_item_name: String,
    pub address: u64,
    pub module_name: String,
    pub data_type_id: String,
    pub pointer_offsets: Option<Vec<PointerChainSegment>>,
}

pub fn build_add_symbol_to_project_target(symbol_tree_node: &SymbolTreeNode) -> Option<AddSymbolToProjectTarget> {
    if matches!(
        symbol_tree_node.get_kind(),
        SymbolTreeNodeKind::ModuleSpace { .. } | SymbolTreeNodeKind::UnassignedSegment { .. }
    ) {
        return None;
    }

    let project_item_name = build_add_symbol_project_item_name(symbol_tree_node);
    let address = symbol_tree_node.get_locator().get_focus_address();
    let module_name = symbol_tree_node
        .get_locator()
        .get_focus_module_name()
        .to_string();
    let pointer_offsets = build_add_symbol_pointer_offsets(symbol_tree_node, address, &module_name);

    Some(AddSymbolToProjectTarget {
        project_item_name: if project_item_name.is_empty() {
            String::from("Symbol")
        } else {
            project_item_name
        },
        address,
        module_name,
        data_type_id: symbol_tree_node.get_display_type_id(),
        pointer_offsets,
    })
}

pub fn build_add_symbol_project_item_create_request(add_symbol_to_project_target: &AddSymbolToProjectTarget) -> ProjectItemsCreateRequest {
    ProjectItemsCreateRequest {
        parent_directory_path: PathBuf::new(),
        project_item_name: add_symbol_to_project_target.project_item_name.clone(),
        is_directory: false,
        address: Some(add_symbol_to_project_target.address),
        module_name: Some(add_symbol_to_project_target.module_name.clone()),
        data_type_id: Some(add_symbol_to_project_target.data_type_id.clone()),
        pointer_offsets: add_symbol_to_project_target.pointer_offsets.clone(),
    }
}

fn build_add_symbol_project_item_name(symbol_tree_node: &SymbolTreeNode) -> String {
    match symbol_tree_node.get_kind() {
        SymbolTreeNodeKind::SymbolClaim { .. } => symbol_tree_node.get_display_name().trim().to_string(),
        _ => symbol_tree_node.get_full_path().trim().to_string(),
    }
}

fn build_add_symbol_pointer_offsets(
    symbol_tree_node: &SymbolTreeNode,
    address: u64,
    module_name: &str,
) -> Option<Vec<PointerChainSegment>> {
    if !matches!(symbol_tree_node.get_kind(), SymbolTreeNodeKind::SymbolClaim { .. })
        || symbol_tree_node.get_depth() != 1
        || module_name.trim().is_empty()
        || !PointerChainSegment::is_valid_symbol_name(symbol_tree_node.get_display_name())
    {
        return None;
    }

    Some(vec![
        PointerChainSegment::new_symbol(symbol_tree_node.get_display_name().to_string()).unwrap_or_else(|| PointerChainSegment::new_offset(address as i64)),
    ])
}
