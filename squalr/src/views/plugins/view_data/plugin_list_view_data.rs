use crate::app_context::AppContext;
use squalr_engine_api::{
    commands::{
        plugins::{list::plugin_list_request::PluginListRequest, set_enabled::plugin_set_enabled_request::PluginSetEnabledRequest},
        privileged_command_request::PrivilegedCommandRequest,
        project::save::project_save_request::ProjectSaveRequest,
        unprivileged_command_request::UnprivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    engine::engine_execution_context::EngineExecutionContext,
    plugins::PluginState,
    structures::processes::opened_process_info::OpenedProcessInfo,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct PluginListViewData {
    plugin_states: Vec<PluginState>,
    selected_plugin_id: Option<String>,
    opened_process_info: Option<OpenedProcessInfo>,
    is_loading: bool,
}

impl PluginListViewData {
    pub fn new() -> Self {
        Self {
            plugin_states: Vec::new(),
            selected_plugin_id: None,
            opened_process_info: None,
            is_loading: false,
        }
    }

    pub fn get_plugin_states(&self) -> &[PluginState] {
        &self.plugin_states
    }

    pub fn get_selected_plugin_id(&self) -> Option<&str> {
        self.selected_plugin_id.as_deref()
    }

    pub fn get_opened_process_info(&self) -> Option<&OpenedProcessInfo> {
        self.opened_process_info.as_ref()
    }

    pub fn get_is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn select_plugin(
        plugin_list_view_data: Dependency<PluginListViewData>,
        plugin_id: Option<String>,
    ) {
        if let Some(mut plugin_list_view_data) = plugin_list_view_data.write("Plugin list view data select plugin") {
            plugin_list_view_data.selected_plugin_id = plugin_id;
        }
    }

    pub fn refresh(
        plugin_list_view_data: Dependency<PluginListViewData>,
        app_context: Arc<AppContext>,
    ) {
        Self::set_loading(plugin_list_view_data.clone(), true);
        let plugin_list_view_data_for_response = plugin_list_view_data.clone();
        let plugin_list_request = PluginListRequest::default();
        let did_dispatch = plugin_list_request.send(&app_context.engine_unprivileged_state, move |plugin_list_response| {
            Self::apply_snapshot(
                plugin_list_view_data_for_response,
                plugin_list_response.plugins,
                plugin_list_response.opened_process_info,
            );
        });

        if !did_dispatch {
            Self::set_loading(plugin_list_view_data, false);
        }
    }

    pub fn set_plugin_enabled(
        plugin_list_view_data: Dependency<PluginListViewData>,
        app_context: Arc<AppContext>,
        plugin_id: String,
        is_enabled: bool,
    ) {
        Self::set_loading(plugin_list_view_data.clone(), true);
        let plugin_list_view_data_for_response = plugin_list_view_data.clone();
        let app_context_for_response = app_context.clone();
        let plugin_set_enabled_request = PluginSetEnabledRequest { plugin_id, is_enabled };
        let did_dispatch = plugin_set_enabled_request.send(&app_context.engine_unprivileged_state, move |plugin_set_enabled_response| {
            let persisted_plugin_states = plugin_set_enabled_response.plugins.clone();

            Self::apply_snapshot(
                plugin_list_view_data_for_response,
                plugin_set_enabled_response.plugins,
                plugin_set_enabled_response.opened_process_info,
            );

            if plugin_set_enabled_response.did_update {
                Self::persist_opened_project_plugin_configuration(app_context_for_response, persisted_plugin_states);
            }
        });

        if !did_dispatch {
            Self::set_loading(plugin_list_view_data, false);
        }
    }

    fn set_loading(
        plugin_list_view_data: Dependency<PluginListViewData>,
        is_loading: bool,
    ) {
        if let Some(mut plugin_list_view_data) = plugin_list_view_data.write("Plugin list view data set loading") {
            plugin_list_view_data.is_loading = is_loading;
        }
    }

    fn apply_snapshot(
        plugin_list_view_data: Dependency<PluginListViewData>,
        plugin_states: Vec<PluginState>,
        opened_process_info: Option<OpenedProcessInfo>,
    ) {
        if let Some(mut plugin_list_view_data) = plugin_list_view_data.write("Plugin list view data apply snapshot") {
            let selected_plugin_id = plugin_list_view_data.selected_plugin_id.clone();
            let resolved_selected_plugin_id = selected_plugin_id
                .filter(|selected_plugin_id| {
                    plugin_states
                        .iter()
                        .any(|plugin_state| plugin_state.get_metadata().get_plugin_id() == selected_plugin_id)
                })
                .or_else(|| {
                    plugin_states
                        .first()
                        .map(|plugin_state| plugin_state.get_metadata().get_plugin_id().to_string())
                });

            plugin_list_view_data.plugin_states = plugin_states;
            plugin_list_view_data.selected_plugin_id = resolved_selected_plugin_id;
            plugin_list_view_data.opened_process_info = opened_process_info;
            plugin_list_view_data.is_loading = false;
        }
    }

    fn persist_opened_project_plugin_configuration(
        app_context: Arc<AppContext>,
        plugin_states: Vec<PluginState>,
    ) {
        let enabled_plugin_ids = {
            let mut enabled_plugin_ids = plugin_states
                .into_iter()
                .filter(|plugin_state| plugin_state.get_is_enabled())
                .map(|plugin_state| plugin_state.get_metadata().get_plugin_id().to_string())
                .collect::<Vec<_>>();

            enabled_plugin_ids.sort();
            enabled_plugin_ids.dedup();
            enabled_plugin_ids
        };

        let has_opened_project = match app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
        {
            Ok(mut opened_project) => {
                if let Some(opened_project) = opened_project.as_mut() {
                    let project_info = opened_project.get_project_info_mut();
                    project_info.set_enabled_plugin_ids(Some(enabled_plugin_ids));
                    project_info.set_has_unsaved_changes(true);
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        };

        if !has_opened_project {
            return;
        }

        let project_save_request = ProjectSaveRequest {};
        project_save_request.send(&app_context.engine_unprivileged_state, move |project_save_response| {
            if !project_save_response.success {
                log::error!("Failed to persist project plugin configuration after plugin enablement changed.");
            }
        });
    }
}

impl Default for PluginListViewData {
    fn default() -> Self {
        Self::new()
    }
}
