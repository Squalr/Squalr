pub mod details;
pub mod operations;
pub mod symbol_tree;
pub mod symbol_tree_node;

pub use details::SymbolTreeDetailsProjection;
pub use operations::build_symbol_tree::ResolvedPointerTarget;
pub use symbol_tree::SymbolTree;
pub use symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind};
