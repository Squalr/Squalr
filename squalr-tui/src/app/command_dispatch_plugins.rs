use super::app_shell::AppShell;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_api::commands::plugins::list::plugin_list_request::PluginListRequest;
use squalr_engine_api::commands::plugins::set_enabled::plugin_set_enabled_request::PluginSetEnabledRequest;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::{privileged_command_request::PrivilegedCommandRequest, unprivileged_command_request::UnprivilegedCommandRequest};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::{Arc, mpsc};
use std::time::Duration;

impl AppShell {
    pub(super) fn refresh_plugins_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self.app_state.plugins_pane_state.is_refreshing_plugins {
            if should_update_status_message {
                self.app_state.plugins_pane_state.status_message = "Plugin refresh is already in progress.".to_string();
            }
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.plugins_pane_state.status_message = "No unprivileged engine state is available for plugin queries.".to_string();
                return;
            }
        };

        if should_update_status_message {
            self.app_state.plugins_pane_state.status_message = "Refreshing plugins.".to_string();
        }

        self.app_state.plugins_pane_state.is_refreshing_plugins = true;
        let plugin_list_request = PluginListRequest::default();
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        plugin_list_request.send(engine_unprivileged_state, move |plugin_list_response| {
            let _ = response_sender.send(plugin_list_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(plugin_list_response) => {
                self.apply_engine_opened_process_state(plugin_list_response.opened_process_info.clone());
                let plugin_count = plugin_list_response.plugins.len();
                self.app_state
                    .plugins_pane_state
                    .apply_plugin_states(plugin_list_response.plugins);
                self.app_state.plugins_pane_state.status_message = format!("Loaded {} plugins.", plugin_count);
            }
            Err(receive_error) => {
                self.app_state.plugins_pane_state.status_message = format!("Timed out waiting for plugin list response: {}", receive_error);
            }
        }

        self.app_state.plugins_pane_state.is_refreshing_plugins = false;
    }

    pub(super) fn toggle_selected_plugin_enabled(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.plugins_pane_state.is_updating_plugin_enabled {
            self.app_state.plugins_pane_state.status_message = "Plugin enablement update is already in progress.".to_string();
            return;
        }

        let Some(selected_plugin) = self.app_state.plugins_pane_state.selected_plugin().cloned() else {
            self.app_state.plugins_pane_state.status_message = "No plugin is selected.".to_string();
            return;
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.plugins_pane_state.status_message = "No unprivileged engine state is available for plugin updates.".to_string();
                return;
            }
        };

        let metadata = selected_plugin.get_metadata();
        let target_enabled_state = !selected_plugin.get_is_enabled();
        let verb = if target_enabled_state { "Enabling" } else { "Disabling" };
        self.app_state.plugins_pane_state.status_message = format!("{} plugin '{}'.", verb, metadata.get_display_name());
        self.app_state.plugins_pane_state.is_updating_plugin_enabled = true;

        let plugin_set_enabled_request = PluginSetEnabledRequest {
            plugin_id: metadata.get_plugin_id().to_string(),
            is_enabled: target_enabled_state,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        plugin_set_enabled_request.send(engine_unprivileged_state, move |plugin_set_enabled_response| {
            let _ = response_sender.send(plugin_set_enabled_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(plugin_set_enabled_response) => {
                self.apply_engine_opened_process_state(plugin_set_enabled_response.opened_process_info.clone());
                self.app_state
                    .plugins_pane_state
                    .apply_plugin_states(plugin_set_enabled_response.plugins);
                self.app_state.plugins_pane_state.status_message = if plugin_set_enabled_response.did_update {
                    let save_status_message = self
                        .persist_opened_project_plugin_configuration(engine_unprivileged_state)
                        .unwrap_or_default();

                    let base_status_message = format!(
                        "{} plugin '{}'.",
                        if target_enabled_state { "Enabled" } else { "Disabled" },
                        metadata.get_display_name()
                    );

                    if save_status_message.is_empty() {
                        base_status_message
                    } else {
                        format!("{} {}", base_status_message, save_status_message)
                    }
                } else {
                    format!(
                        "Plugin '{}' was already {} or could not be updated.",
                        metadata.get_display_name(),
                        if target_enabled_state { "enabled" } else { "disabled" }
                    )
                };
            }
            Err(receive_error) => {
                self.app_state.plugins_pane_state.status_message = format!("Timed out waiting for plugin enablement response: {}", receive_error);
            }
        }

        self.app_state.plugins_pane_state.is_updating_plugin_enabled = false;
    }

    fn persist_opened_project_plugin_configuration(
        &mut self,
        engine_unprivileged_state: &Arc<squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState>,
    ) -> Option<String> {
        let has_opened_project = match engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .read()
        {
            Ok(opened_project) => opened_project.is_some(),
            Err(_) => false,
        };

        if !has_opened_project {
            return None;
        }

        let project_save_request = ProjectSaveRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_save_request.send(engine_unprivileged_state, move |project_save_response| {
            let _ = response_sender.send(project_save_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_save_response) => {
                if project_save_response.success {
                    Some("Saved plugin configuration to the opened project.".to_string())
                } else {
                    Some("Project save failed while persisting plugin configuration.".to_string())
                }
            }
            Err(receive_error) => Some(format!("Timed out waiting for project save response: {}", receive_error)),
        }
    }
}
