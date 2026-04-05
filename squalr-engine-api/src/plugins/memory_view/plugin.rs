use crate::{
    plugins::{Plugin, memory_view::{MemoryViewInstance, MemoryViewPluginError}},
    structures::processes::opened_process_info::OpenedProcessInfo,
};

pub trait MemoryViewPlugin: Plugin {
    fn can_attach(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> bool;

    fn create_instance(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Result<Box<dyn MemoryViewInstance>, MemoryViewPluginError>;
}
