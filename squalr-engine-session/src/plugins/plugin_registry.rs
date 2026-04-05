use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use squalr_engine_api::{
    plugins::{PluginActivationState, PluginState, memory_view::MemoryViewPlugin},
    structures::processes::opened_process_info::OpenedProcessInfo,
};
use squalr_plugin_builtins::get_builtin_memory_view_plugins;

pub struct PluginRegistry {
    memory_view_plugins: Vec<Arc<dyn MemoryViewPlugin>>,
    enabled_plugin_ids: RwLock<HashSet<String>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let memory_view_plugins = get_builtin_memory_view_plugins();
        let enabled_plugin_ids = memory_view_plugins
            .iter()
            .map(|memory_view_plugin| memory_view_plugin.metadata().get_plugin_id().to_string())
            .collect();

        Self {
            memory_view_plugins,
            enabled_plugin_ids: RwLock::new(enabled_plugin_ids),
        }
    }

    pub fn get_memory_view_plugins(&self) -> &[Arc<dyn MemoryViewPlugin>] {
        &self.memory_view_plugins
    }

    pub fn get_plugin_states(
        &self,
        opened_process_info: Option<&OpenedProcessInfo>,
        active_plugin_id: Option<&str>,
    ) -> Vec<PluginState> {
        let selected_plugin_id = opened_process_info
            .and_then(|opened_process_info| self.find_memory_view_plugin(opened_process_info))
            .map(|memory_view_plugin| memory_view_plugin.metadata().get_plugin_id().to_string());

        self.memory_view_plugins
            .iter()
            .map(|memory_view_plugin| {
                let is_enabled = self.is_plugin_enabled(memory_view_plugin.metadata().get_plugin_id());
                let can_activate_for_current_process = opened_process_info
                    .map(|opened_process_info| memory_view_plugin.can_attach(opened_process_info))
                    .unwrap_or(false);
                let activation_state = if active_plugin_id
                    .map(|active_plugin_id| active_plugin_id == memory_view_plugin.metadata().get_plugin_id())
                    .unwrap_or(false)
                {
                    PluginActivationState::Activated
                } else if selected_plugin_id
                    .as_deref()
                    .map(|selected_plugin_id| selected_plugin_id == memory_view_plugin.metadata().get_plugin_id())
                    .unwrap_or(false)
                {
                    PluginActivationState::Activating
                } else if can_activate_for_current_process {
                    PluginActivationState::Available
                } else {
                    PluginActivationState::Idle
                };

                PluginState::new(memory_view_plugin.metadata().clone(), is_enabled, activation_state)
            })
            .collect()
    }

    pub fn set_plugin_enabled(
        &self,
        plugin_id: &str,
        is_enabled: bool,
    ) -> bool {
        if !self
            .memory_view_plugins
            .iter()
            .any(|memory_view_plugin| memory_view_plugin.metadata().get_plugin_id() == plugin_id)
        {
            return false;
        }

        match self.enabled_plugin_ids.write() {
            Ok(mut enabled_plugin_ids) => {
                if is_enabled {
                    enabled_plugin_ids.insert(plugin_id.to_string())
                } else {
                    enabled_plugin_ids.remove(plugin_id)
                }
            }
            Err(error) => {
                log::error!("Failed to update plugin enablement for `{}`: {}", plugin_id, error);
                false
            }
        }
    }

    pub fn is_plugin_enabled(
        &self,
        plugin_id: &str,
    ) -> bool {
        self.enabled_plugin_ids
            .read()
            .map(|enabled_plugin_ids| enabled_plugin_ids.contains(plugin_id))
            .unwrap_or(false)
    }

    pub fn find_memory_view_plugin(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Option<Arc<dyn MemoryViewPlugin>> {
        self.memory_view_plugins
            .iter()
            .find(|memory_view_plugin| self.is_plugin_enabled(memory_view_plugin.metadata().get_plugin_id()) && memory_view_plugin.can_attach(process_info))
            .cloned()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::PluginRegistry;
    use squalr_engine_api::{
        plugins::PluginActivationState,
        structures::{memory::bitness::Bitness, processes::opened_process_info::OpenedProcessInfo},
    };

    #[test]
    fn registry_exposes_builtin_dolphin_memory_view_plugin() {
        let plugin_registry = PluginRegistry::new();
        let opened_process_info = OpenedProcessInfo::new(1, "Dolphin.exe".to_string(), 0, Bitness::Bit64, None);
        let plugin = plugin_registry.find_memory_view_plugin(&opened_process_info);

        assert!(plugin.is_some());
        assert_eq!(plugin_registry.get_memory_view_plugins().len(), 1);
        assert_eq!(
            plugin
                .expect("Expected the Dolphin plugin to match the Dolphin process.")
                .metadata()
                .get_plugin_id(),
            "builtin.memory-view.dolphin"
        );
    }

    #[test]
    fn selected_plugin_reports_activating_until_router_has_live_instance() {
        let plugin_registry = PluginRegistry::new();
        let opened_process_info = OpenedProcessInfo::new(1, "Dolphin.exe".to_string(), 0, Bitness::Bit64, None);

        let plugin_states = plugin_registry.get_plugin_states(Some(&opened_process_info), None);

        assert_eq!(plugin_states.len(), 1);
        assert_eq!(plugin_states[0].get_activation_state(), PluginActivationState::Activating);
    }

    #[test]
    fn live_router_plugin_reports_activated() {
        let plugin_registry = PluginRegistry::new();
        let opened_process_info = OpenedProcessInfo::new(1, "Dolphin.exe".to_string(), 0, Bitness::Bit64, None);

        let plugin_states = plugin_registry.get_plugin_states(Some(&opened_process_info), Some("builtin.memory-view.dolphin"));

        assert_eq!(plugin_states.len(), 1);
        assert_eq!(plugin_states[0].get_activation_state(), PluginActivationState::Activated);
    }

    #[test]
    fn disabling_plugin_prevents_matching_and_updates_state() {
        let plugin_registry = PluginRegistry::new();
        let opened_process_info = OpenedProcessInfo::new(1, "Dolphin.exe".to_string(), 0, Bitness::Bit64, None);

        assert!(plugin_registry.set_plugin_enabled("builtin.memory-view.dolphin", false));
        assert!(!plugin_registry.is_plugin_enabled("builtin.memory-view.dolphin"));
        assert!(
            plugin_registry
                .find_memory_view_plugin(&opened_process_info)
                .is_none()
        );

        let plugin_states = plugin_registry.get_plugin_states(Some(&opened_process_info), None);

        assert_eq!(plugin_states.len(), 1);
        assert!(!plugin_states[0].get_is_enabled());
        assert!(plugin_states[0].get_can_activate_for_current_process());
        assert!(!plugin_states[0].get_is_active_for_current_process());
    }
}
