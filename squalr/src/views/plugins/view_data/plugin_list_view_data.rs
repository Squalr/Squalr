use crate::app_context::AppContext;
use eframe::egui::Pos2;
use squalr_engine_api::{
    commands::{
        command_invocation::{CommandInvocationOutcome, EngineCommandResponse},
        plugins::plugins_response::PluginsResponse,
        plugins::{
            list::plugin_list_request::PluginListRequest, set_enabled::plugin_set_enabled_request::PluginSetEnabledRequest,
            set_order::plugin_set_order_request::PluginSetOrderRequest,
        },
        privileged_command_request::PrivilegedCommandRequest,
        privileged_command_response::PrivilegedCommandResponse,
        project::save::project_save_request::ProjectSaveRequest,
        unprivileged_command_request::UnprivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    engine::engine_execution_context::EngineExecutionContext,
    plugins::{PluginConfiguration, PluginState},
    structures::processes::opened_process_info::OpenedProcessInfo,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct PluginListViewData {
    plugin_states: Vec<PluginState>,
    default_plugin_ids: Vec<String>,
    selected_plugin_id: Option<String>,
    context_menu_plugin_id: Option<String>,
    context_menu_position: Option<Pos2>,
    opened_process_info: Option<OpenedProcessInfo>,
    is_loading: bool,
}

#[derive(Clone, Copy)]
pub enum PluginPriorityShiftDirection {
    Increase,
    Decrease,
}

impl PluginListViewData {
    pub fn new() -> Self {
        Self {
            plugin_states: Vec::new(),
            default_plugin_ids: Vec::new(),
            selected_plugin_id: None,
            context_menu_plugin_id: None,
            context_menu_position: None,
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

    pub fn get_context_menu_state(&self) -> Option<(&str, Pos2)> {
        self.context_menu_plugin_id
            .as_deref()
            .zip(self.context_menu_position)
    }

    pub fn get_opened_process_info(&self) -> Option<&OpenedProcessInfo> {
        self.opened_process_info.as_ref()
    }

    pub fn show_context_menu(
        plugin_list_view_data: Dependency<PluginListViewData>,
        plugin_id: String,
        position: Pos2,
    ) {
        if let Some(mut plugin_list_view_data) = plugin_list_view_data.write("Plugin list view data show context menu") {
            plugin_list_view_data.context_menu_plugin_id = Some(plugin_id);
            plugin_list_view_data.context_menu_position = Some(position);
        }
    }

    pub fn hide_context_menu(plugin_list_view_data: Dependency<PluginListViewData>) {
        if let Some(mut plugin_list_view_data) = plugin_list_view_data.write("Plugin list view data hide context menu") {
            plugin_list_view_data.context_menu_plugin_id = None;
            plugin_list_view_data.context_menu_position = None;
        }
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
                plugin_list_response.default_plugin_ids,
            );
        });

        if !did_dispatch {
            Self::set_loading(plugin_list_view_data, false);
        }
    }

    pub fn observe_command_responses(
        plugin_list_view_data: Dependency<PluginListViewData>,
        app_context: Arc<AppContext>,
    ) {
        let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();

        engine_unprivileged_state.listen_for_command_response(move |command_invocation_outcome| {
            Self::apply_observed_command_response(plugin_list_view_data.clone(), command_invocation_outcome);
        });
    }

    fn apply_observed_command_response(
        plugin_list_view_data: Dependency<PluginListViewData>,
        command_invocation_outcome: &CommandInvocationOutcome,
    ) {
        let EngineCommandResponse::Privileged(PrivilegedCommandResponse::Plugins(plugins_response)) = command_invocation_outcome.get_response() else {
            return;
        };

        match plugins_response {
            PluginsResponse::List { plugin_list_response } => {
                Self::apply_snapshot(
                    plugin_list_view_data,
                    plugin_list_response.plugins.clone(),
                    plugin_list_response.opened_process_info.clone(),
                    plugin_list_response.default_plugin_ids.clone(),
                );
            }
            PluginsResponse::SetEnabled { plugin_set_enabled_response } => {
                Self::apply_snapshot(
                    plugin_list_view_data,
                    plugin_set_enabled_response.plugins.clone(),
                    plugin_set_enabled_response.opened_process_info.clone(),
                    plugin_set_enabled_response.default_plugin_ids.clone(),
                );
            }
            PluginsResponse::SetOrder { plugin_set_order_response } => {
                Self::apply_snapshot(
                    plugin_list_view_data,
                    plugin_set_order_response.plugins.clone(),
                    plugin_set_order_response.opened_process_info.clone(),
                    plugin_set_order_response.default_plugin_ids.clone(),
                );
            }
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
                plugin_set_enabled_response.default_plugin_ids.clone(),
            );

            if plugin_set_enabled_response.did_update {
                Self::persist_opened_project_plugin_configuration(
                    app_context_for_response,
                    persisted_plugin_states,
                    plugin_set_enabled_response.default_plugin_ids,
                );
            }
        });

        if !did_dispatch {
            Self::set_loading(plugin_list_view_data, false);
        }
    }

    pub fn shift_plugin_priority(
        plugin_list_view_data: Dependency<PluginListViewData>,
        app_context: Arc<AppContext>,
        plugin_id: String,
        shift_direction: PluginPriorityShiftDirection,
    ) {
        let plugin_ids = match plugin_list_view_data.read("Plugin list view data shift plugin priority") {
            Some(plugin_list_view_data) => {
                let mut plugin_ids = plugin_list_view_data
                    .plugin_states
                    .iter()
                    .map(|plugin_state| plugin_state.get_metadata().get_plugin_id().to_string())
                    .collect::<Vec<_>>();
                let Some(source_plugin_position) = plugin_ids
                    .iter()
                    .position(|candidate_plugin_id| candidate_plugin_id == &plugin_id)
                else {
                    return;
                };
                let target_plugin_position = match shift_direction {
                    PluginPriorityShiftDirection::Increase => source_plugin_position.checked_sub(1),
                    PluginPriorityShiftDirection::Decrease => {
                        let next_plugin_position = source_plugin_position + 1;

                        if next_plugin_position < plugin_ids.len() {
                            Some(next_plugin_position)
                        } else {
                            None
                        }
                    }
                };
                let Some(target_plugin_position) = target_plugin_position else {
                    return;
                };

                plugin_ids.swap(source_plugin_position, target_plugin_position);
                plugin_ids
            }
            None => return,
        };

        Self::hide_context_menu(plugin_list_view_data.clone());
        Self::set_loading(plugin_list_view_data.clone(), true);
        let plugin_list_view_data_for_response = plugin_list_view_data.clone();
        let app_context_for_response = app_context.clone();
        let plugin_set_order_request = PluginSetOrderRequest { plugin_ids };
        let did_dispatch = plugin_set_order_request.send(&app_context.engine_unprivileged_state, move |plugin_set_order_response| {
            let persisted_plugin_states = plugin_set_order_response.plugins.clone();
            let persisted_default_plugin_ids = plugin_set_order_response.default_plugin_ids.clone();

            Self::apply_snapshot(
                plugin_list_view_data_for_response,
                plugin_set_order_response.plugins,
                plugin_set_order_response.opened_process_info,
                plugin_set_order_response.default_plugin_ids,
            );

            if plugin_set_order_response.did_update {
                Self::persist_opened_project_plugin_configuration(app_context_for_response, persisted_plugin_states, persisted_default_plugin_ids);
            }
        });

        if !did_dispatch {
            Self::set_loading(plugin_list_view_data, false);
        }
    }

    pub fn has_opened_project(app_context: Arc<AppContext>) -> bool {
        match app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
        {
            Ok(opened_project) => opened_project.is_some(),
            Err(_) => false,
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
        default_plugin_ids: Vec<String>,
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
            plugin_list_view_data.default_plugin_ids = default_plugin_ids;
            plugin_list_view_data.selected_plugin_id = resolved_selected_plugin_id;
            plugin_list_view_data.opened_process_info = opened_process_info;
            plugin_list_view_data.is_loading = false;
        }
    }

    fn persist_opened_project_plugin_configuration(
        app_context: Arc<AppContext>,
        plugin_states: Vec<PluginState>,
        default_plugin_ids: Vec<String>,
    ) {
        let plugin_configuration = PluginConfiguration::from_plugin_states(&plugin_states, &default_plugin_ids);

        let has_opened_project = match app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
        {
            Ok(mut opened_project) => {
                if let Some(opened_project) = opened_project.as_mut() {
                    let project_info = opened_project.get_project_info_mut();
                    project_info.set_plugin_configuration(plugin_configuration);
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
