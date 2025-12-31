use crate::app_context::AppContext;
use eframe::egui::TextureOptions;
use epaint::{ColorImage, TextureHandle};
use squalr_engine_api::{
    commands::{
        engine_command_request::EngineCommandRequest,
        process::{list::process_list_request::ProcessListRequest, open::process_open_request::ProcessOpenRequest},
    },
    dependency_injection::{dependency::Dependency, write_guard::WriteGuard},
    structures::processes::{opened_process_info::OpenedProcessInfo, process_icon::ProcessIcon, process_info::ProcessInfo},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone)]
pub struct ProcessSelectorViewData {
    pub opened_process: Option<OpenedProcessInfo>,
    pub cached_icon: Option<TextureHandle>,
    pub windowed_process_list: Vec<ProcessInfo>,
    pub full_process_list: Vec<ProcessInfo>,
    pub icon_cache: HashMap<u32, TextureHandle>,
    pub is_awaiting_windowed_process_list: bool,
    pub is_awaiting_full_process_list: bool,
    pub is_opening_process: bool,
}

impl ProcessSelectorViewData {
    pub fn new() -> Self {
        Self {
            opened_process: None,
            cached_icon: None,
            windowed_process_list: Vec::new(),
            full_process_list: Vec::new(),
            icon_cache: HashMap::new(),
            is_awaiting_windowed_process_list: false,
            is_awaiting_full_process_list: false,
            is_opening_process: false,
        }
    }

    pub fn refresh_windowed_process_list(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
    ) {
        let list_windowed_processes_request = ProcessListRequest {
            require_windowed: true,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };

        let engine_execution_context = app_context.engine_execution_context.clone();

        // Early exit if already awaiting response. Clear windowed list if querying up to date info.
        match process_selector_view_data.write("Process selector view data refresh windowed process list") {
            Some(mut process_selector_view_data) => {
                if process_selector_view_data.is_awaiting_windowed_process_list {
                    return;
                }

                process_selector_view_data.is_awaiting_windowed_process_list = true;
            }
            None => return,
        };

        list_windowed_processes_request.send(&engine_execution_context, move |process_list_response| {
            let mut process_selector_view_data = match process_selector_view_data.write("Process selector view data refresh windowed process list response") {
                Some(process_selector_view_data) => process_selector_view_data,
                None => return,
            };

            process_selector_view_data.is_awaiting_windowed_process_list = false;
            ProcessSelectorViewData::set_windowed_process_list(&mut process_selector_view_data, &app_context, process_list_response.processes);
        });
    }

    pub fn refresh_full_process_list(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
    ) {
        let list_windowed_processes_request = ProcessListRequest {
            require_windowed: false,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: true,
        };

        let engine_execution_context = app_context.engine_execution_context.clone();

        // Early exit if already awaiting response. Clear full list if querying up to date info.
        match process_selector_view_data.write("Process selector view data refresh full process list") {
            Some(mut process_selector_view_data) => {
                if process_selector_view_data.is_awaiting_full_process_list {
                    return;
                }

                process_selector_view_data.is_awaiting_full_process_list = true;
            }
            None => return,
        };

        list_windowed_processes_request.send(&engine_execution_context, move |process_list_response| {
            let mut process_selector_view_data = match process_selector_view_data.write("Process selector view data refresh full process list response") {
                Some(process_selector_view_data) => process_selector_view_data,
                None => return,
            };

            process_selector_view_data.is_awaiting_full_process_list = false;

            Self::set_full_process_list(&mut process_selector_view_data, &app_context, process_list_response.processes);
        });
    }

    pub fn select_process(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
        process_id: Option<u32>,
    ) {
        if process_id.is_some() {
            let engine_execution_context = app_context.engine_execution_context.clone();
            let process_open_request = ProcessOpenRequest {
                process_id,
                search_name: None,
                match_case: false,
            };

            match process_selector_view_data.write("Process selector view data select process") {
                Some(mut process_selector_view_data) => {
                    if process_selector_view_data.is_opening_process {
                        return;
                    }

                    process_selector_view_data.is_opening_process = true;
                }
                None => return,
            };

            process_open_request.send(&engine_execution_context, move |process_open_response| {
                Self::set_opened_process_info(process_selector_view_data, &app_context, process_open_response.opened_process_info)
            });
        } else {
            Self::set_opened_process_info(process_selector_view_data, &app_context, None)
        }
    }

    pub fn set_opened_process_info(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: &Arc<AppContext>,
        opened_process: Option<OpenedProcessInfo>,
    ) {
        let mut process_selector_view_data = match process_selector_view_data.write("Process selector view data set opened process info") {
            Some(process_selector_view_data) => process_selector_view_data,
            None => return,
        };

        process_selector_view_data.is_opening_process = false;
        process_selector_view_data.opened_process = opened_process;

        let icon_data = match &process_selector_view_data.opened_process {
            Some(opened_proces) => match opened_proces.get_icon() {
                Some(icon) => {
                    let process_id = opened_proces.get_process_id_raw();
                    Some((process_id, icon.clone()))
                }
                None => None,
            },
            None => None,
        };

        if let Some((process_id, icon)) = icon_data {
            let texture_handle = process_selector_view_data.get_icon(app_context, process_id, &icon);

            process_selector_view_data.cached_icon = texture_handle;
        } else {
            process_selector_view_data.cached_icon = None;
        }
    }

    pub fn create_and_cache_icon(
        process_selector_view_data: &mut WriteGuard<'_, ProcessSelectorViewData>,
        app_context: &Arc<AppContext>,
        process_id: u32,
        icon: &ProcessIcon,
    ) {
        if process_selector_view_data.icon_cache.contains_key(&process_id) {
            return;
        }

        let size = [icon.get_width() as usize, icon.get_height() as usize];
        let texture = app_context.context.load_texture(
            &format!("process_icon_{process_id}"),
            ColorImage::from_rgba_unmultiplied(size, icon.get_bytes_rgba()),
            TextureOptions::default(),
        );

        process_selector_view_data
            .icon_cache
            .insert(process_id, texture);
    }

    pub fn get_icon(
        &self,
        app_context: &Arc<AppContext>,
        process_id: u32,
        icon: &ProcessIcon,
    ) -> Option<TextureHandle> {
        if self.icon_cache.contains_key(&process_id) {
            return self.icon_cache.get(&process_id).cloned();
        }

        let size = [icon.get_width() as usize, icon.get_height() as usize];
        let texture = app_context.context.load_texture(
            &format!("process_icon_{process_id}"),
            ColorImage::from_rgba_unmultiplied(size, icon.get_bytes_rgba()),
            TextureOptions::default(),
        );

        Some(texture)
    }

    pub fn set_windowed_process_list(
        process_selector_view_data: &mut WriteGuard<'_, ProcessSelectorViewData>,
        app_context: &Arc<AppContext>,
        new_list: Vec<ProcessInfo>,
    ) {
        let removed = Self::diff_pids(&process_selector_view_data.windowed_process_list, &new_list);

        // Cache icons for the new list up front.
        for process in &new_list {
            let pid = process.get_process_id_raw();
            if let Some(icon) = process.get_icon() {
                Self::create_and_cache_icon(process_selector_view_data, app_context, pid, &icon);
            }
        }

        process_selector_view_data.windowed_process_list = new_list;

        // Remove icons for processes no longer present.
        Self::remove_from_cache(process_selector_view_data, &removed);

        // If current opened process was removed, clear it.
        if let Some(opened) = &process_selector_view_data.opened_process {
            if removed.contains(&opened.get_process_id_raw()) {
                process_selector_view_data.cached_icon = None;
            }
        }
    }

    pub fn set_full_process_list(
        process_selector_view_data: &mut WriteGuard<'_, ProcessSelectorViewData>,
        app_context: &Arc<AppContext>,
        new_list: Vec<ProcessInfo>,
    ) {
        let removed = Self::diff_pids(&process_selector_view_data.full_process_list, &new_list);

        // Cache icons for the new list up front.
        for process in &new_list {
            let pid = process.get_process_id_raw();
            if let Some(icon) = process.get_icon() {
                Self::create_and_cache_icon(process_selector_view_data, app_context, pid, &icon);
            }
        }

        process_selector_view_data.full_process_list = new_list;

        // Remove icons for processes no longer present.
        Self::remove_from_cache(process_selector_view_data, &removed);
    }

    /// Computes process ID deltas between old/new PID sets.
    fn diff_pids(
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
        process_selector_view_data: &mut WriteGuard<'_, ProcessSelectorViewData>,
        removed: &HashSet<u32>,
    ) {
        process_selector_view_data
            .icon_cache
            .retain(|process_id, _| !removed.contains(process_id));
    }
}
