use crate::plugins::{Plugin, data_type::DataTypePlugin, memory_view::MemoryViewPlugin};

pub trait PluginPackage: Plugin {
    fn as_data_type_plugin(&self) -> Option<&dyn DataTypePlugin> {
        None
    }

    fn as_memory_view_plugin(&self) -> Option<&dyn MemoryViewPlugin> {
        None
    }
}
