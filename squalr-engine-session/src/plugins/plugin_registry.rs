use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use squalr_engine_api::{
    plugins::{PluginActivationState, PluginCapability, PluginPackage, PluginState},
    structures::processes::opened_process_info::OpenedProcessInfo,
};
use squalr_plugin_builtins::get_builtin_plugin_packages;

pub struct PluginRegistry {
    plugin_packages: Vec<Arc<dyn PluginPackage>>,
    data_type_plugin_ids_by_data_type_id: HashMap<String, String>,
    enabled_plugin_ids: RwLock<HashSet<String>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::from_plugin_packages(get_builtin_plugin_packages())
    }

    pub(crate) fn from_plugin_packages(plugin_packages: Vec<Arc<dyn PluginPackage>>) -> Self {
        let enabled_plugin_ids = plugin_packages
            .iter()
            .map(|plugin_package| plugin_package.metadata())
            .filter(|plugin_metadata| plugin_metadata.get_is_enabled_by_default())
            .map(|plugin_metadata| plugin_metadata.get_plugin_id().to_string())
            .collect();
        let data_type_plugin_ids_by_data_type_id = Self::build_data_type_plugin_ids_by_data_type_id(&plugin_packages);

        Self {
            plugin_packages,
            data_type_plugin_ids_by_data_type_id,
            enabled_plugin_ids: RwLock::new(enabled_plugin_ids),
        }
    }

    fn build_data_type_plugin_ids_by_data_type_id(plugin_packages: &[Arc<dyn PluginPackage>]) -> HashMap<String, String> {
        let mut data_type_plugin_ids_by_data_type_id = HashMap::new();

        for plugin_package in plugin_packages {
            let Some(data_type_plugin) = plugin_package.as_data_type_plugin() else {
                continue;
            };

            for data_type_id in data_type_plugin.contributed_data_type_ids() {
                if data_type_plugin_ids_by_data_type_id.contains_key(*data_type_id) {
                    log::warn!(
                        "Ignoring duplicate contributed data type id '{}' from plugin package '{}'.",
                        data_type_id,
                        plugin_package.metadata().get_plugin_id()
                    );
                    continue;
                }

                data_type_plugin_ids_by_data_type_id.insert((*data_type_id).to_string(), plugin_package.metadata().get_plugin_id().to_string());
            }
        }

        data_type_plugin_ids_by_data_type_id
    }

    pub fn get_plugin_packages(&self) -> &[Arc<dyn PluginPackage>] {
        &self.plugin_packages
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
                    .plugin_packages
                    .iter()
                    .map(|plugin_package| self.build_plugin_state(plugin_package.as_ref(), false, opened_process_info, active_plugin_id, None))
                    .collect();
            }
        };

        let selected_plugin_id = opened_process_info
            .and_then(|opened_process_info| self.find_memory_view_plugin_package_with_enabled_ids(opened_process_info, &enabled_plugin_ids))
            .map(|plugin_package| plugin_package.metadata().get_plugin_id().to_string());

        self.plugin_packages
            .iter()
            .map(|plugin_package| {
                let is_enabled = enabled_plugin_ids.contains(plugin_package.metadata().get_plugin_id());

                self.build_plugin_state(
                    plugin_package.as_ref(),
                    is_enabled,
                    opened_process_info,
                    active_plugin_id,
                    selected_plugin_id.as_deref(),
                )
            })
            .collect()
    }

    fn build_plugin_state(
        &self,
        plugin_package: &dyn PluginPackage,
        is_enabled: bool,
        opened_process_info: Option<&OpenedProcessInfo>,
        active_plugin_id: Option<&str>,
        selected_plugin_id: Option<&str>,
    ) -> PluginState {
        let activation_state = plugin_package
            .as_memory_view_plugin()
            .map(|memory_view_plugin| {
                let can_activate_for_current_process = opened_process_info
                    .map(|opened_process_info| memory_view_plugin.can_attach(opened_process_info))
                    .unwrap_or(false);

                if active_plugin_id
                    .map(|active_plugin_id| active_plugin_id == plugin_package.metadata().get_plugin_id())
                    .unwrap_or(false)
                {
                    PluginActivationState::Activated
                } else if selected_plugin_id
                    .map(|selected_plugin_id| selected_plugin_id == plugin_package.metadata().get_plugin_id())
                    .unwrap_or(false)
                {
                    PluginActivationState::Activating
                } else if can_activate_for_current_process {
                    PluginActivationState::Available
                } else {
                    PluginActivationState::Idle
                }
            })
            .unwrap_or(PluginActivationState::Idle);

        PluginState::new(plugin_package.metadata().clone(), is_enabled, activation_state)
    }

    pub fn set_plugin_enabled(
        &self,
        plugin_id: &str,
        is_enabled: bool,
    ) -> bool {
        if !self
            .plugin_packages
            .iter()
            .any(|plugin_package| plugin_package.metadata().get_plugin_id() == plugin_id)
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

    pub fn get_enabled_plugin_ids(&self) -> Vec<String> {
        let mut enabled_plugin_ids = self
            .enabled_plugin_ids
            .read()
            .map(|enabled_plugin_ids| enabled_plugin_ids.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        enabled_plugin_ids.sort();
        enabled_plugin_ids
    }

    pub fn get_plugin_capabilities(
        &self,
        plugin_id: &str,
    ) -> Option<Vec<PluginCapability>> {
        self.plugin_packages
            .iter()
            .find(|plugin_package| plugin_package.metadata().get_plugin_id() == plugin_id)
            .map(|plugin_package| plugin_package.metadata().get_plugin_capabilities().to_vec())
    }

    pub fn has_plugin_capability(
        &self,
        plugin_id: &str,
        plugin_capability: PluginCapability,
    ) -> bool {
        self.plugin_packages
            .iter()
            .find(|plugin_package| plugin_package.metadata().get_plugin_id() == plugin_id)
            .map(|plugin_package| {
                plugin_package
                    .metadata()
                    .has_plugin_capability(plugin_capability)
            })
            .unwrap_or(false)
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

    pub fn find_memory_view_plugin_package(
        &self,
        process_info: &OpenedProcessInfo,
    ) -> Option<Arc<dyn PluginPackage>> {
        let enabled_plugin_ids = match self.enabled_plugin_ids.read() {
            Ok(enabled_plugin_ids) => enabled_plugin_ids,
            Err(error) => {
                log::error!("Failed to acquire plugin enablement snapshot while selecting memory-view plugin: {}", error);
                return None;
            }
        };

        self.find_memory_view_plugin_package_with_enabled_ids(process_info, &enabled_plugin_ids)
    }

    fn find_memory_view_plugin_package_with_enabled_ids(
        &self,
        process_info: &OpenedProcessInfo,
        enabled_plugin_ids: &HashSet<String>,
    ) -> Option<Arc<dyn PluginPackage>> {
        self.plugin_packages
            .iter()
            .find(|plugin_package| {
                enabled_plugin_ids.contains(plugin_package.metadata().get_plugin_id())
                    && plugin_package
                        .as_memory_view_plugin()
                        .map(|memory_view_plugin| memory_view_plugin.can_attach(process_info))
                        .unwrap_or(false)
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
        let plugin_package = plugin_registry.find_memory_view_plugin_package(&opened_process_info);

        assert!(plugin_package.is_some());
        assert_eq!(plugin_registry.get_plugin_packages().len(), 2);
        assert_eq!(
            plugin_package
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
        let dolphin_plugin_state = plugin_states
            .iter()
            .find(|plugin_state| plugin_state.get_metadata().get_plugin_id() == "builtin.memory-view.dolphin")
            .expect("Expected the Dolphin plugin state to be present.");

        assert_eq!(plugin_states.len(), 2);
        assert_eq!(dolphin_plugin_state.get_activation_state(), PluginActivationState::Activating);
    }

    #[test]
    fn live_router_plugin_reports_activated() {
        let plugin_registry = PluginRegistry::new();
        let opened_process_info = OpenedProcessInfo::new(1, "Dolphin.exe".to_string(), 0, Bitness::Bit64, None);

        let plugin_states = plugin_registry.get_plugin_states(Some(&opened_process_info), Some("builtin.memory-view.dolphin"));
        let dolphin_plugin_state = plugin_states
            .iter()
            .find(|plugin_state| plugin_state.get_metadata().get_plugin_id() == "builtin.memory-view.dolphin")
            .expect("Expected the Dolphin plugin state to be present.");

        assert_eq!(plugin_states.len(), 2);
        assert_eq!(dolphin_plugin_state.get_activation_state(), PluginActivationState::Activated);
    }

    #[test]
    fn disabling_plugin_prevents_matching_and_updates_state() {
        let plugin_registry = PluginRegistry::new();
        let opened_process_info = OpenedProcessInfo::new(1, "Dolphin.exe".to_string(), 0, Bitness::Bit64, None);

        assert!(plugin_registry.set_plugin_enabled("builtin.memory-view.dolphin", false));
        assert!(!plugin_registry.is_plugin_enabled("builtin.memory-view.dolphin"));
        assert!(
            plugin_registry
                .find_memory_view_plugin_package(&opened_process_info)
                .is_none()
        );

        let plugin_states = plugin_registry.get_plugin_states(Some(&opened_process_info), None);
        let dolphin_plugin_state = plugin_states
            .iter()
            .find(|plugin_state| plugin_state.get_metadata().get_plugin_id() == "builtin.memory-view.dolphin")
            .expect("Expected the Dolphin plugin state to be present.");

        assert_eq!(plugin_states.len(), 2);
        assert!(!dolphin_plugin_state.get_is_enabled());
        assert!(dolphin_plugin_state.get_can_activate_for_current_process());
        assert!(!dolphin_plugin_state.get_is_active_for_current_process());
    }
}
