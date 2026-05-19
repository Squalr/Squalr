use crate::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteModuleRangeMode;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    project_symbol_locator::ProjectSymbolLocator,
    symbol_tree::{
        operations::build_symbol_tree::resolve_symbol_tree_node_size_in_bytes,
        symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind},
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleChildRangeTarget {
    pub module_name: String,
    pub offset: u64,
    pub length: u64,
    pub display_name: String,
    pub delete_mode: ProjectSymbolsDeleteModuleRangeMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeleteConfirmationDescription {
    pub text: String,
    pub is_warning: bool,
}

pub fn build_delete_module_range_confirmation_description(
    module_name: &str,
    length: u64,
    mode: ProjectSymbolsDeleteModuleRangeMode,
) -> DeleteConfirmationDescription {
    match mode {
        ProjectSymbolsDeleteModuleRangeMode::ShiftLeft => DeleteConfirmationDescription {
            text: format!(
                "WARNING: {} will be {} byte(s) smaller. Proceeding fields will be shifted left.",
                module_name, length
            ),
            is_warning: true,
        },
        ProjectSymbolsDeleteModuleRangeMode::ReplaceWithUnassigned => DeleteConfirmationDescription {
            text: String::from("This removes the field definition and leaves the bytes unassigned."),
            is_warning: false,
        },
    }
}

pub fn build_module_child_range_target(
    project_symbol_catalog: &ProjectSymbolCatalog,
    symbol_tree_node: &SymbolTreeNode,
    resolve_primitive_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
) -> Option<ModuleChildRangeTarget> {
    match symbol_tree_node.get_kind() {
        SymbolTreeNodeKind::UnassignedSegment { module_name, offset, length } => Some(ModuleChildRangeTarget {
            module_name: module_name.to_string(),
            offset: *offset,
            length: *length,
            display_name: symbol_tree_node.get_display_name().to_string(),
            delete_mode: ProjectSymbolsDeleteModuleRangeMode::ShiftLeft,
        }),
        SymbolTreeNodeKind::SymbolClaim { .. } if symbol_tree_node.get_depth() == 1 => {
            let ProjectSymbolLocator::ModuleOffset { module_name, offset } = symbol_tree_node.get_locator() else {
                return None;
            };
            let length = resolve_symbol_tree_node_size_in_bytes(project_symbol_catalog, symbol_tree_node, resolve_primitive_size_in_bytes);

            (length > 0).then(|| ModuleChildRangeTarget {
                module_name: module_name.to_string(),
                offset: *offset,
                length,
                display_name: symbol_tree_node.get_display_name().to_string(),
                delete_mode: ProjectSymbolsDeleteModuleRangeMode::ReplaceWithUnassigned,
            })
        }
        _ => None,
    }
}
