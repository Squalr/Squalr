use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use squalr_engine_api::{
    plugins::{PluginActivationState, PluginKind, PluginState, data_type::DataTypePlugin, memory_view::MemoryViewPlugin},
    structures::processes::opened_process_info::OpenedProcessInfo,
};
use squalr_plugin_builtins::{get_builtin_data_type_plugins, get_builtin_memory_view_plugins};

pub struct PluginRegistry {
    memory_view_plugins: Vec<Arc<dyn MemoryViewPlugin>>,
    data_type_plugins: Vec<Arc<dyn DataTypePlugin>>,
    data_type_plugin_ids_by_data_type_id: HashMap<String, String>,
    enabled_plugin_ids: RwLock<HashSet<String>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let memory_view_plugins = get_builtin_memory_view_plugins();
        let data_type_plugins = get_builtin_data_type_plugins();
        let enabled_plugin_ids = memory_view_plugins
            .iter()
            .map(|memory_view_plugin| memory_view_plugin.metadata())
            .chain(
                data_type_plugins
                    .iter()
                    .map(|data_type_plugin| data_type_plugin.metadata()),
            )
            .filter(|plugin_metadata| plugin_metadata.get_is_enabled_by_default())
            .map(|plugin_metadata| plugin_metadata.get_plugin_id().to_string())
            .collect();
        let data_type_plugin_ids_by_data_type_id = data_type_plugins
            .iter()
            .flat_map(|data_type_plugin| {
                data_type_plugin
                    .contributed_data_type_ids()
                    .iter()
                    .map(move |data_type_id| ((*data_type_id).to_string(), data_type_plugin.metadata().get_plugin_id().to_string()))
            })
            .collect();

        Self {
            memory_view_plugins,
            data_type_plugins,
            data_type_plugin_ids_by_data_type_id,
            enabled_plugin_ids: RwLock::new(enabled_plugin_ids),
        }
    }

    pub fn get_memory_view_plugins(&self) -> &[Arc<dyn MemoryViewPlugin>] {
        &self.memory_view_plugins
    }

    pub fn get_data_type_plugins(&self) -> &[Arc<dyn DataTypePlugin>] {
        &self.data_type_plugins
    }

    pub fn get_plugin_states(
        &self,
        opened_process_info: Option<&OpenedProcessInfo>,
        active_plugin_id: Option<&str>,
    ) -> Vec<PluginState> {
        let enabled_plugin_ids = match self.enabled_plugin_ids.read() {
            Ok(enabled_plugin_ids) => enabled_plugin_ids,
            Err(error) => {
                log::error!("Failed to acquire plugin enablement snapshot: {}", error);
                return self
                    .memory_view_plugins
                    .iter()
                    .map(|memory_view_plugin| {
                        let can_activate_for_current_process = opened_process_info
                            .map(|opened_process_info| memory_view_plugin.can_attach(opened_process_info))
                            .unwrap_or(false);
                        let activation_state = if active_plugin_id
                            .map(|active_plugin_id| active_plugin_id == memory_view_plugin.metadata().get_plugin_id())
                            .unwrap_or(false)
                        {
                            PluginActivationState::Activated
                        } else if can_activate_for_current_process {
                            PluginActivationState::Available
                        } else {
                            PluginActivationState::Idle
                        };

                        PluginState::new(memory_view_plugin.metadata().clone(), false, activation_state)
                    })
                    .collect();
            }
        };

        let selected_plugin_id = opened_process_info
            .and_then(|opened_process_info| self.find_memory_view_plugin_with_enabled_ids(opened_process_info, &enabled_plugin_ids))
            .map(|memory_view_plugin| memory_view_plugin.metadata().get_plugin_id().to_string());

        self.memory_view_plugins
            .iter()
            .map(|memory_view_plugin| {
                let is_enabled = enabled_plugin_ids.contains(memory_view_plugin.metadata().get_plugin_id());
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
            .chain(self.data_type_plugins.iter().map(|data_type_plugin| {
                let is_enabled = enabled_plugin_ids.contains(data_type_plugin.metadata().get_plugin_id());

                PluginState::new(data_type_plugin.metadata().clone(), is_enabled, PluginActivationState::Idle)
            }))
            .collect()
    }

    pub fn set_plugin_enabled(
        &self,
        plugin_id: &str,
        is_enabled: bool,
    ) -> bool {
        if self.get_plugin_kind(plugin_id).is_none() {
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

    pub fn get_enabled_plugin_ids(&self) -> Vec<String> {
        let mut enabled_plugin_ids = self
            .enabled_plugin_ids
            .read()
            .map(|enabled_plugin_ids| enabled_plugin_ids.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        enabled_plugin_ids.sort();
        enabled_plugin_ids
    }

    pub fn get_plugin_kind(
        &self,
        plugin_id: &str,
    ) -> Option<PluginKind> {
        self.memory_view_plugins
            .iter()
            .map(|memory_view_plugin| memory_view_plugin.metadata())
            .chain(
                self.data_type_plugins
                    .iter()
                    .map(|data_type_plugin| data_type_plugin.metadata()),
            )
            .find(|plugin_metadata| plugin_metadata.get_plugin_id() == plugin_id)
            .map(|plugin_metadata| plugin_metadata.get_plugin_kind())
    }

    pub fn is_data_type_enabled(
        &self,
        data_type_id: &str,
    ) -> bool {
        let Some(plugin_id) = self.data_type_plugin_ids_by_data_type_id.get(data_type_id) else {
            return true;
        };

        self.is_plugin_enabled(plugin_id)
    }

    pub fn find_memory_view_plugin(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Option<Arc<dyn MemoryViewPlugin>> {
        let enabled_plugin_ids = match self.enabled_plugin_ids.read() {
            Ok(enabled_plugin_ids) => enabled_plugin_ids,
            Err(error) => {
                log::error!("Failed to acquire plugin enablement snapshot while selecting memory-view plugin: {}", error);
                return None;
            }
        };

        self.find_memory_view_plugin_with_enabled_ids(process_info, &enabled_plugin_ids)
    }

    fn find_memory_view_plugin_with_enabled_ids(
        &self,
        process_info: &OpenedProcessInfo,
        enabled_plugin_ids: &HashSet<String>,
    ) -> Option<Arc<dyn MemoryViewPlugin>> {
        self.memory_view_plugins
            .iter()
            .find(|memory_view_plugin| {
                enabled_plugin_ids.contains(memory_view_plugin.metadata().get_plugin_id()) && memory_view_plugin.can_attach(process_info)
            })
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
