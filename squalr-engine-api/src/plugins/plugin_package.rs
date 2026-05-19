use crate::plugins::{
    Plugin, data_type::DataTypePlugin, instruction_set::InstructionSetPlugin, memory_view::MemoryViewPlugin, symbol_tree::symbol_tree_plugin::SymbolTreePlugin,
};

pub trait PluginPackage: Plugin {
    fn as_data_type_plugin(&self) -> Option<&dyn DataTypePlugin> {
        None
    }

    fn as_instruction_set_plugin(&self) -> Option<&dyn InstructionSetPlugin> {
        None
    }

    fn as_memory_view_plugin(&self) -> Option<&dyn MemoryViewPlugin> {
        None
    }

    fn as_symbol_tree_plugin(&self) -> Option<&dyn SymbolTreePlugin> {
        None
    }
}
