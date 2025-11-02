use eframe::egui::{Context, TextureOptions};
use epaint::{ColorImage, TextureHandle};
use squalr_engine_api::{
    commands::{
        engine_command_request::EngineCommandRequest,
        process::{list::process_list_request::ProcessListRequest, open::process_open_request::ProcessOpenRequest},
    },
    dependency_injection::dependency::Dependency,
    engine::engine_execution_context::EngineExecutionContext,
    structures::processes::{opened_process_info::OpenedProcessInfo, process_icon::ProcessIcon, process_info::ProcessInfo},
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use crate::app_context::AppContext;

pub struct ProcessSelectorViewData {
    pub opened_process: Option<OpenedProcessInfo>,
    pub cached_icon: Option<TextureHandle>,
    pub windowed_process_list: Vec<ProcessInfo>,
    pub full_process_list: Vec<ProcessInfo>,
    pub icon_cache: RwLock<HashMap<u32, TextureHandle>>,
}

impl ProcessSelectorViewData {
    pub fn new() -> Self {
        Self {
            opened_process: None,
            cached_icon: None,
            windowed_process_list: Vec::new(),
            full_process_list: Vec::new(),
            icon_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn refresh_windowed_process_list(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let list_windowed_processes_request = ProcessListRequest {
            require_windowed: true,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };
        let engine_execution_context = engine_execution_context.clone();

        list_windowed_processes_request.send(&engine_execution_context, move |process_list_response| {
            let mut process_selector_view_data = match process_selector_view_data.write() {
                Ok(process_selector_view_data) => process_selector_view_data,
                Err(error) => {
                    log::error!("Failed to access process selector view data for updating windowed process list: {}", error);
                    return;
                }
            };

            process_selector_view_data.set_windowed_process_list(process_list_response.processes);
        });
    }

    pub fn refresh_full_process_list(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let list_windowed_processes_request = ProcessListRequest {
            require_windowed: false,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };
        let engine_execution_context = engine_execution_context.clone();

        list_windowed_processes_request.send(&engine_execution_context, move |process_list_response| {
            let mut process_selector_view_data = match process_selector_view_data.write() {
                Ok(process_selector_view_data) => process_selector_view_data,
                Err(error) => {
                    log::error!("Failed to access process selector view data for updating windowed process list: {}", error);
                    return;
                }
            };

            process_selector_view_data.set_full_process_list(process_list_response.processes);
        });
    }

    pub fn select_process(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
        process_id: u32,
    ) {
        let engine_execution_context = app_context.engine_execution_context.clone();
        let process_open_request = ProcessOpenRequest {
            process_id: Some(process_id),
            search_name: None,
            match_case: false,
        };

        process_open_request.send(&engine_execution_context, move |process_open_response| {
            Self::update_cached_opened_process(process_selector_view_data, app_context, process_open_response.opened_process_info)
        });
    }

    pub fn update_cached_opened_process(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
        process_info: Option<OpenedProcessInfo>,
    ) {
        let mut process_selector_view_data = match process_selector_view_data.write() {
            Ok(process_selector_view_data) => process_selector_view_data,
            Err(_error) => return,
        };

        process_selector_view_data.set_opened_process(&app_context.context, process_info);
    }

    pub fn set_opened_process(
        &mut self,
        context: &Context,
        opened_process: Option<OpenedProcessInfo>,
    ) {
        self.opened_process = opened_process;

        match &self.opened_process {
            Some(opened_proces) => match opened_proces.get_icon() {
                Some(icon) => {
                    let process_id = opened_proces.get_process_id_raw();
                    let texture_handle = self.get_or_create_icon(context, process_id, icon);

                    self.cached_icon = texture_handle;
                }
                None => self.cached_icon = None,
            },
            None => self.cached_icon = None,
        }
    }

    pub fn get_or_create_icon(
        &self,
        context: &Context,
        process_id: u32,
        icon: &ProcessIcon,
    ) -> Option<TextureHandle> {
        let mut icon_cache = match self.icon_cache.write() {
            Ok(icon_cache) => icon_cache,
            Err(error) => {
                log::error!("Failed to acquire icon cache lock: {}", error);
                return None;
            }
        };

        if icon_cache.contains_key(&process_id) {
            return icon_cache.get(&process_id).cloned();
        }

        let size = [icon.get_width() as usize, icon.get_height() as usize];
        let texture = context.load_texture(
            &format!("process_icon_{process_id}"),
            ColorImage::from_rgba_unmultiplied(size, icon.get_bytes_rgba()),
            TextureOptions::default(),
        );

        icon_cache.insert(process_id, texture.clone());

        Some(texture)
    }

    pub fn set_windowed_process_list(
        &mut self,
        new_list: Vec<ProcessInfo>,
    ) {
        let removed = self.diff_pids(&self.windowed_process_list, &new_list);

        self.windowed_process_list = new_list;

        // Remove icons for processes no longer present.
        self.remove_from_cache(&removed);

        // If current opened process was removed, clear it.
        if let Some(opened) = &self.opened_process {
            if removed.contains(&opened.get_process_id_raw()) {
                self.cached_icon = None;
            }
        }
    }

    pub fn set_full_process_list(
        &mut self,
        new_list: Vec<ProcessInfo>,
    ) {
        let removed = self.diff_pids(&self.full_process_list, &new_list);

        self.full_process_list = new_list;

        // Remove icons for processes no longer present.
        self.remove_from_cache(&removed);
    }

    /// Computes process ID deltas between old/new PID sets.
    fn diff_pids(
        &self,
        old: &[ProcessInfo],
        new: &[ProcessInfo],
    ) -> HashSet<u32> {
        let old_set: HashSet<u32> = old
            .iter()
            .map(|process_info| process_info.get_process_id_raw())
            .collect();
        let new_set: HashSet<u32> = new
            .iter()
            .map(|process_info| process_info.get_process_id_raw())
            .collect();
        let removed = &old_set - &new_set;

        removed
    }

    /// Removes cached icons for removed processes.
    fn remove_from_cache(
        &self,
        removed: &HashSet<u32>,
    ) {
        let mut icon_cache = match self.icon_cache.write() {
            Ok(icon_cache) => icon_cache,
            Err(error) => {
                log::error!("Failed to lock icon cache: {}", error);
                return;
            }
        };

        icon_cache.retain(|process_id, _| !removed.contains(process_id));
    }
}
