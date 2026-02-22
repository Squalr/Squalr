use crate::app_context::AppContext;
use eframe::egui::TextureOptions;
use epaint::{ColorImage, TextureHandle};
use squalr_engine_api::{
    commands::{
        privileged_command_request::PrivilegedCommandRequest,
        process::{list::process_list_request::ProcessListRequest, open::process_open_request::ProcessOpenRequest},
    },
    dependency_injection::{dependency::Dependency, write_guard::WriteGuard},
    structures::processes::{opened_process_info::OpenedProcessInfo, process_icon::ProcessIcon, process_info::ProcessInfo},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct ProcessSelectorViewData {
    pub opened_process: Option<OpenedProcessInfo>,
    pub cached_icon: Option<TextureHandle>,
    pub show_windowed_processes_only: bool,
    pub windowed_process_list: Vec<ProcessInfo>,
    pub full_process_list: Vec<ProcessInfo>,
    pub icon_cache: HashMap<u32, TextureHandle>,
    pub is_awaiting_windowed_process_list: bool,
    pub is_awaiting_full_process_list: bool,
    pub is_opening_process: bool,
    windowed_process_list_request_started_at: Option<Instant>,
    full_process_list_request_started_at: Option<Instant>,
    open_process_request_started_at: Option<Instant>,
}

impl ProcessSelectorViewData {
    const REQUEST_STALE_TIMEOUT: Duration = Duration::from_secs(3);

    pub fn new() -> Self {
        Self {
            opened_process: None,
            cached_icon: None,
            show_windowed_processes_only: cfg!(target_os = "android"),
            windowed_process_list: Vec::new(),
            full_process_list: Vec::new(),
            icon_cache: HashMap::new(),
            is_awaiting_windowed_process_list: false,
            is_awaiting_full_process_list: false,
            is_opening_process: false,
            windowed_process_list_request_started_at: None,
            full_process_list_request_started_at: None,
            open_process_request_started_at: None,
        }
    }

    pub fn clear_stale_request_state(process_selector_view_data_dependency: Dependency<ProcessSelectorViewData>) {
        if let Some(mut process_selector_view_data) = process_selector_view_data_dependency.write("Process selector view data clear stale request state") {
            process_selector_view_data.clear_stale_request_state_for_now(Instant::now());
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

        let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();

        // Early exit if already awaiting response. Clear windowed list if querying up to date info.
        match process_selector_view_data.write("Process selector view data refresh windowed process list") {
            Some(mut process_selector_view_data) => {
                if process_selector_view_data.is_awaiting_windowed_process_list {
                    log::debug!("Skipping windowed process-list refresh because a request is already pending.");
                    return;
                }

                process_selector_view_data.is_awaiting_windowed_process_list = true;
                process_selector_view_data.windowed_process_list_request_started_at = Some(Instant::now());
                log::info!("Dispatching windowed process-list refresh request.");
            }
            None => return,
        };

        let process_selector_view_data_for_response = process_selector_view_data.clone();
        let did_dispatch = list_windowed_processes_request.send(&engine_unprivileged_state, move |process_list_response| {
            let mut process_selector_view_data = match process_selector_view_data.write("Process selector view data refresh windowed process list response") {
                Some(process_selector_view_data) => process_selector_view_data,
                None => return,
            };

            process_selector_view_data.is_awaiting_windowed_process_list = false;
            process_selector_view_data.windowed_process_list_request_started_at = None;
            log::info!(
                "Received windowed process-list response with {} entries.",
                process_list_response.processes.len()
            );
            ProcessSelectorViewData::set_windowed_process_list(&mut process_selector_view_data, &app_context, process_list_response.processes);
        });

        if !did_dispatch {
            log::warn!("Windowed process-list refresh request failed to dispatch.");
            if let Some(mut process_selector_view_data) =
                process_selector_view_data_for_response.write("Process selector view data refresh windowed process list dispatch failure")
            {
                process_selector_view_data.is_awaiting_windowed_process_list = false;
                process_selector_view_data.windowed_process_list_request_started_at = None;
            }
        }
    }

    pub fn refresh_active_process_list(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
    ) {
        let show_windowed_processes_only = process_selector_view_data
            .read("Process selector view data refresh active process list")
            .map(|process_selector_view_data| process_selector_view_data.show_windowed_processes_only)
            .unwrap_or(false);

        if show_windowed_processes_only {
            Self::refresh_windowed_process_list(process_selector_view_data, app_context);
        } else {
            Self::refresh_full_process_list(process_selector_view_data, app_context);
        }
    }

    pub fn set_show_windowed_processes_only(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
        show_windowed_processes_only: bool,
    ) {
        if let Some(mut process_selector_view_data_guard) = process_selector_view_data.write("Process selector view data set windowed mode") {
            process_selector_view_data_guard.show_windowed_processes_only = show_windowed_processes_only;
        }

        Self::refresh_active_process_list(process_selector_view_data, app_context);
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

        let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();

        // Early exit if already awaiting response. Clear full list if querying up to date info.
        match process_selector_view_data.write("Process selector view data refresh full process list") {
            Some(mut process_selector_view_data) => {
                if process_selector_view_data.is_awaiting_full_process_list {
                    log::debug!("Skipping full process-list refresh because a request is already pending.");
                    return;
                }

                process_selector_view_data.is_awaiting_full_process_list = true;
                process_selector_view_data.full_process_list_request_started_at = Some(Instant::now());
                log::info!("Dispatching full process-list refresh request.");
            }
            None => return,
        };

        let process_selector_view_data_for_response = process_selector_view_data.clone();
        let did_dispatch = list_windowed_processes_request.send(&engine_unprivileged_state, move |process_list_response| {
            let mut process_selector_view_data = match process_selector_view_data.write("Process selector view data refresh full process list response") {
                Some(process_selector_view_data) => process_selector_view_data,
                None => return,
            };

            process_selector_view_data.is_awaiting_full_process_list = false;
            process_selector_view_data.full_process_list_request_started_at = None;
            log::info!("Received full process-list response with {} entries.", process_list_response.processes.len());

            Self::set_full_process_list(&mut process_selector_view_data, &app_context, process_list_response.processes);
        });

        if !did_dispatch {
            log::warn!("Full process-list refresh request failed to dispatch.");
            if let Some(mut process_selector_view_data) =
                process_selector_view_data_for_response.write("Process selector view data refresh full process list dispatch failure")
            {
                process_selector_view_data.is_awaiting_full_process_list = false;
                process_selector_view_data.full_process_list_request_started_at = None;
            }
        }
    }

    pub fn select_process(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
        process_id: Option<u32>,
    ) {
        if process_id.is_some() {
            let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();
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
                    process_selector_view_data.open_process_request_started_at = Some(Instant::now());
                }
                None => return,
            };

            let process_selector_view_data_for_response = process_selector_view_data.clone();
            let did_dispatch = process_open_request.send(&engine_unprivileged_state, move |process_open_response| {
                Self::set_opened_process_info(process_selector_view_data, &app_context, process_open_response.opened_process_info)
            });

            if !did_dispatch {
                if let Some(mut process_selector_view_data) =
                    process_selector_view_data_for_response.write("Process selector view data select process dispatch failure")
                {
                    process_selector_view_data.is_opening_process = false;
                    process_selector_view_data.open_process_request_started_at = None;
                }
            }
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
        process_selector_view_data.open_process_request_started_at = None;
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

    fn clear_stale_request_state_for_now(
        &mut self,
        current_instant: Instant,
    ) {
        if Self::is_request_stale(
            current_instant,
            self.windowed_process_list_request_started_at,
            self.is_awaiting_windowed_process_list,
        ) {
            self.is_awaiting_windowed_process_list = false;
            self.windowed_process_list_request_started_at = None;
            log::warn!("Cleared stale windowed process-list loading state after timeout.");
        }

        if Self::is_request_stale(current_instant, self.full_process_list_request_started_at, self.is_awaiting_full_process_list) {
            self.is_awaiting_full_process_list = false;
            self.full_process_list_request_started_at = None;
            log::warn!("Cleared stale full process-list loading state after timeout.");
        }

        if Self::is_request_stale(current_instant, self.open_process_request_started_at, self.is_opening_process) {
            self.is_opening_process = false;
            self.open_process_request_started_at = None;
            log::warn!("Cleared stale process-open loading state after timeout.");
        }
    }

    fn is_request_stale(
        current_instant: Instant,
        request_started_at: Option<Instant>,
        is_request_pending: bool,
    ) -> bool {
        if !is_request_pending {
            return false;
        }

        match request_started_at {
            Some(request_start_instant) => current_instant
                .checked_duration_since(request_start_instant)
                .map(|elapsed_duration| elapsed_duration >= Self::REQUEST_STALE_TIMEOUT)
                .unwrap_or(false),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProcessSelectorViewData;
    use std::time::{Duration, Instant};

    #[test]
    fn request_is_stale_when_pending_and_timeout_elapsed() {
        let current_instant = Instant::now();
        let request_started_at = current_instant - (ProcessSelectorViewData::REQUEST_STALE_TIMEOUT + Duration::from_millis(1));

        let is_stale = ProcessSelectorViewData::is_request_stale(current_instant, Some(request_started_at), true);

        assert!(is_stale);
    }

    #[test]
    fn request_is_not_stale_when_not_pending() {
        let current_instant = Instant::now();

        let is_stale = ProcessSelectorViewData::is_request_stale(current_instant, Some(current_instant), false);

        assert!(!is_stale);
    }

    #[test]
    fn request_is_stale_when_pending_without_start_timestamp() {
        let current_instant = Instant::now();

        let is_stale = ProcessSelectorViewData::is_request_stale(current_instant, None, true);

        assert!(is_stale);
    }
}
