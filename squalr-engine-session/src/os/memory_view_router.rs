use crate::plugins::plugin_registry::PluginRegistry;
use squalr_engine_api::{plugins::memory_view::MemoryViewInstance, structures::processes::opened_process_info::OpenedProcessInfo};
use std::sync::{Arc, Mutex, RwLock};

pub(crate) type SharedMemoryViewInstance = Arc<Mutex<Box<dyn MemoryViewInstance>>>;

struct CachedMemoryViewInstance {
    process_id: u32,
    process_handle: u64,
    process_name: String,
    plugin_id: String,
    memory_view_instance: SharedMemoryViewInstance,
}

impl CachedMemoryViewInstance {
    fn new(
        process_info: &OpenedProcessInfo,
        plugin_id: String,
        memory_view_instance: SharedMemoryViewInstance,
    ) -> Self {
        Self {
            process_id: process_info.get_process_id(),
            process_handle: process_info.get_handle(),
            process_name: process_info.get_name().to_string(),
            plugin_id,
            memory_view_instance,
        }
    }

    fn matches(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> bool {
        self.process_id == process_info.get_process_id() && self.process_handle == process_info.get_handle() && self.process_name == process_info.get_name()
    }
}

pub(crate) struct MemoryViewRouter {
    plugin_registry: Arc<PluginRegistry>,
    active_memory_view_instance: RwLock<Option<CachedMemoryViewInstance>>,
}

impl MemoryViewRouter {
    pub(crate) fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self {
            plugin_registry,
            active_memory_view_instance: RwLock::new(None),
        }
    }

    pub(crate) fn get_or_create_instance(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Option<SharedMemoryViewInstance> {
        let matching_memory_view_plugin = self.plugin_registry.find_memory_view_plugin(process_info);

        if let Ok(active_memory_view_instance) = self.active_memory_view_instance.read() {
            if let Some(cached_memory_view_instance) = active_memory_view_instance.as_ref() {
                if cached_memory_view_instance.matches(process_info) {
                    if matching_memory_view_plugin
                        .as_ref()
                        .map(|memory_view_plugin| memory_view_plugin.metadata().get_plugin_id() == cached_memory_view_instance.plugin_id)
                        .unwrap_or(false)
                    {
                        return Some(cached_memory_view_instance.memory_view_instance.clone());
                    }
                }
            }
        }

        if let Ok(mut active_memory_view_instance) = self.active_memory_view_instance.write() {
            let should_clear_cached_instance = active_memory_view_instance
                .as_ref()
                .map(|cached_memory_view_instance| cached_memory_view_instance.matches(process_info))
                .unwrap_or(false);

            if should_clear_cached_instance {
                *active_memory_view_instance = None;
            }
        }

        let memory_view_plugin = matching_memory_view_plugin?;
        let plugin_id = memory_view_plugin.metadata().get_plugin_id().to_string();

        let memory_view_instance = match memory_view_plugin.create_instance(process_info) {
            Ok(memory_view_instance) => memory_view_instance,
            Err(error) => {
                log::warn!(
                    "Failed to create memory-view instance for plugin `{}` on process `{}` (pid {}): {}",
                    plugin_id,
                    process_info.get_name(),
                    process_info.get_process_id(),
                    error
                );
                return None;
            }
        };

        let shared_memory_view_instance = Arc::new(Mutex::new(memory_view_instance));

        if let Ok(mut active_memory_view_instance) = self.active_memory_view_instance.write() {
            *active_memory_view_instance = Some(CachedMemoryViewInstance::new(
                process_info,
                plugin_id.clone(),
                shared_memory_view_instance.clone(),
            ));
        }

        log::info!(
            "Activated memory-view plugin `{}` for process `{}` (pid {}).",
            plugin_id,
            process_info.get_name(),
            process_info.get_process_id()
        );

        Some(shared_memory_view_instance)
    }

    pub(crate) fn get_active_instance(&self) -> Option<SharedMemoryViewInstance> {
        self.active_memory_view_instance
            .read()
            .ok()
            .and_then(|active_memory_view_instance| {
                active_memory_view_instance
                    .as_ref()
                    .map(|cached_memory_view_instance| cached_memory_view_instance.memory_view_instance.clone())
            })
    }

    pub(crate) fn clear_cached_instance(
        &self,
        process_handle: u64,
    ) {
        if let Ok(mut active_memory_view_instance) = self.active_memory_view_instance.write() {
            if active_memory_view_instance
                .as_ref()
                .map(|cached_memory_view_instance| cached_memory_view_instance.process_handle == process_handle)
                .unwrap_or(false)
            {
                *active_memory_view_instance = None;
            }
        }
    }
}
