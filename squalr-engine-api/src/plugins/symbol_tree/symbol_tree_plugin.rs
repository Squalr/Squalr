use crate::plugins::{Plugin, symbol_tree::symbol_tree_action::SymbolTreeAction};
use std::sync::Arc;

pub trait SymbolTreePlugin: Plugin {
    fn symbol_tree_actions(&self) -> &[Arc<dyn SymbolTreeAction>];
}
