use crate::{
    constants::{DOLPHIN_PLUGIN_DESCRIPTION, DOLPHIN_PLUGIN_DISPLAY_NAME, DOLPHIN_PLUGIN_ID},
    instance::DolphinMemoryViewInstance,
    process_detection::matches_dolphin_process_name,
};
use squalr_engine_api::{
    plugins::memory_view::{MemoryViewInstance, MemoryViewPlugin, MemoryViewPluginError},
    plugins::{Plugin, PluginCapability, PluginMetadata, PluginPackage},
    structures::processes::opened_process_info::OpenedProcessInfo,
};

pub struct DolphinMemoryViewPlugin {
    metadata: PluginMetadata,
}

impl DolphinMemoryViewPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                DOLPHIN_PLUGIN_ID,
                DOLPHIN_PLUGIN_DISPLAY_NAME,
                DOLPHIN_PLUGIN_DESCRIPTION,
                vec![PluginCapability::MemoryView],
                true,
                true,
            ),
        }
    }
}

impl Default for DolphinMemoryViewPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for DolphinMemoryViewPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

impl PluginPackage for DolphinMemoryViewPlugin {
    fn as_memory_view_plugin(&self) -> Option<&dyn MemoryViewPlugin> {
        Some(self)
    }
}

impl MemoryViewPlugin for DolphinMemoryViewPlugin {
    fn can_attach(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> bool {
        matches_dolphin_process_name(process_info.get_name())
    }

    fn create_instance(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Result<Box<dyn MemoryViewInstance>, MemoryViewPluginError> {
        Ok(Box::new(DolphinMemoryViewInstance::new(process_info.clone())))
    }
}
