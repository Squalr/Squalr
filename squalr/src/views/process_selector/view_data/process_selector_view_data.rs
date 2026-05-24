use crate::app_context::AppContext;
use eframe::egui::TextureOptions;
use epaint::{ColorImage, TextureHandle};
use squalr_engine_api::{
    commands::{
        command_invocation::{CommandInvocationOutcome, EngineCommand, EngineCommandResponse},
        privileged_command::PrivilegedCommand,
        privileged_command_request::PrivilegedCommandRequest,
        privileged_command_response::PrivilegedCommandResponse,
        process::{
            icon::process_icon_request::ProcessIconRequest, icon::process_icon_response::ProcessIconEntry, list::process_list_request::ProcessListRequest,
            open::process_open_request::ProcessOpenRequest, process_command::ProcessCommand, process_response::ProcessResponse,
        },
    },
    dependency_injection::{dependency::Dependency, write_guard::WriteGuard},
    structures::processes::{opened_process_info::OpenedProcessInfo, process_icon::ProcessIcon, process_info::ProcessInfo},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    thread,
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
    pub loading_icon_process_ids: HashSet<u32>,
    pub missing_icon_process_ids: HashMap<u32, Instant>,
    pub is_awaiting_windowed_process_list: bool,
    pub is_awaiting_full_process_list: bool,
    pub is_opening_process: bool,
    pub windowed_process_list_refresh_nonce: u64,
    pub shortcut_dropdown_refresh_nonce: u64,
    pub shortcut_dropdown_process_list: Vec<ProcessInfo>,
    windowed_process_list_request_started_at: Option<Instant>,
    full_process_list_request_started_at: Option<Instant>,
    open_process_request_started_at: Option<Instant>,
}

impl ProcessSelectorViewData {
    const REQUEST_STALE_TIMEOUT: Duration = Duration::from_secs(3);
    const PROCESS_ICON_RETRY_COOLDOWN: Duration = Duration::from_secs(5);
    const IS_ANDROID_TARGET: bool = cfg!(target_os = "android");
    const ENABLE_LAZY_PROCESS_ICONS: bool = true;
    const PROCESS_ICON_REQUEST_BATCH_SIZE: usize = 16;

    pub fn new() -> Self {
        Self {
            opened_process: None,
            cached_icon: None,
            show_windowed_processes_only: cfg!(target_os = "android"),
            windowed_process_list: Vec::new(),
            full_process_list: Vec::new(),
            icon_cache: HashMap::new(),
            loading_icon_process_ids: HashSet::new(),
            missing_icon_process_ids: HashMap::new(),
            is_awaiting_windowed_process_list: false,
            is_awaiting_full_process_list: false,
            is_opening_process: false,
            windowed_process_list_refresh_nonce: 0,
            shortcut_dropdown_refresh_nonce: 0,
            shortcut_dropdown_process_list: Vec::new(),
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

    pub fn observe_command_responses(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
    ) {
        let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();

        engine_unprivileged_state.listen_for_command_response(move |command_invocation_outcome| {
            Self::apply_observed_command_response(process_selector_view_data.clone(), app_context.clone(), command_invocation_outcome);
        });
    }

    fn apply_observed_command_response(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
        command_invocation_outcome: &CommandInvocationOutcome,
    ) {
        let EngineCommandResponse::Privileged(PrivilegedCommandResponse::Process(process_response)) = command_invocation_outcome.get_response() else {
            return;
        };

        match process_response {
            ProcessResponse::List { process_list_response } => {
                let require_windowed = Self::observed_process_list_requires_windowed(command_invocation_outcome);
                let Some(mut process_selector_view_data_guard) = process_selector_view_data.write("Observed process list response") else {
                    return;
                };
                Self::apply_process_list_response(&mut process_selector_view_data_guard, require_windowed, process_list_response.processes.clone());

                drop(process_selector_view_data_guard);
                Self::request_repaint(&app_context);
            }
            ProcessResponse::Open { process_open_response } => {
                Self::set_opened_process_info(process_selector_view_data, &app_context, process_open_response.opened_process_info.clone());
            }
            ProcessResponse::Close { process_close_response } => {
                Self::set_opened_process_info(process_selector_view_data, &app_context, process_close_response.process_info.clone());
            }
            ProcessResponse::Icon { process_icon_response } => {
                Self::apply_process_icon_response(process_selector_view_data, &app_context, process_icon_response.process_icons.clone());
            }
        }
    }

    fn observed_process_list_requires_windowed(command_invocation_outcome: &CommandInvocationOutcome) -> bool {
        match command_invocation_outcome.get_invocation().get_command() {
            EngineCommand::Privileged(PrivilegedCommand::Process(ProcessCommand::List { process_list_request })) => process_list_request.require_windowed,
            _ => false,
        }
    }

    fn request_repaint(app_context: &Arc<AppContext>) {
        app_context.context.request_repaint();
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
            fetch_icons: false,
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
                process_selector_view_data.windowed_process_list_refresh_nonce = process_selector_view_data
                    .windowed_process_list_refresh_nonce
                    .saturating_add(1);
            }
            None => return,
        };

        let process_selector_view_data_for_response = process_selector_view_data.clone();
        let list_windowed_processes_request_for_dispatch = list_windowed_processes_request.clone();
        let app_context_for_response = app_context.clone();
        thread::spawn(move || {
            let process_selector_view_data_for_callback = process_selector_view_data_for_response.clone();
            let did_dispatch = list_windowed_processes_request_for_dispatch.send(&engine_unprivileged_state, move |process_list_response| {
                let process_ids_for_icon_request = process_list_response
                    .processes
                    .iter()
                    .map(|process_info| process_info.get_process_id_raw())
                    .collect::<Vec<_>>();
                let mut process_selector_view_data = match process_selector_view_data.write("Process selector view data refresh windowed process list response")
                {
                    Some(process_selector_view_data) => process_selector_view_data,
                    None => return,
                };

                Self::apply_process_list_response(&mut process_selector_view_data, true, process_list_response.processes);
                drop(process_selector_view_data);
                Self::request_process_icons_if_needed(
                    process_selector_view_data_for_callback.clone(),
                    app_context_for_response.clone(),
                    process_ids_for_icon_request,
                );
                Self::request_repaint(&app_context_for_response);
            });

            if !did_dispatch {
                log::warn!("Windowed process-list refresh request failed to dispatch.");

                if let Some(mut process_selector_view_data) =
                    process_selector_view_data_for_response.write("Process selector view data refresh windowed process list dispatch failure")
                {
                    process_selector_view_data.is_awaiting_windowed_process_list = false;
                    process_selector_view_data.windowed_process_list_request_started_at = None;
                }

                Self::request_repaint(&app_context);
            }
        });
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
            process_selector_view_data_guard.refresh_shortcut_dropdown_process_list();
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
            fetch_icons: false,
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
            }
            None => return,
        };

        let process_selector_view_data_for_response = process_selector_view_data.clone();
        let list_windowed_processes_request_for_dispatch = list_windowed_processes_request.clone();
        let app_context_for_response = app_context.clone();
        thread::spawn(move || {
            let did_dispatch = list_windowed_processes_request_for_dispatch.send(&engine_unprivileged_state, move |process_list_response| {
                let mut process_selector_view_data = match process_selector_view_data.write("Process selector view data refresh full process list response") {
                    Some(process_selector_view_data) => process_selector_view_data,
                    None => return,
                };

                Self::apply_process_list_response(&mut process_selector_view_data, false, process_list_response.processes);
                drop(process_selector_view_data);
                Self::request_repaint(&app_context_for_response);
            });

            if !did_dispatch {
                log::warn!("Full process-list refresh request failed to dispatch.");
                if let Some(mut process_selector_view_data) =
                    process_selector_view_data_for_response.write("Process selector view data refresh full process list dispatch failure")
                {
                    process_selector_view_data.is_awaiting_full_process_list = false;
                    process_selector_view_data.full_process_list_request_started_at = None;
                }

                Self::request_repaint(&app_context);
            }
        });
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
            let app_context_for_response = app_context.clone();
            let did_dispatch = process_open_request.send(&engine_unprivileged_state, move |process_open_response| {
                Self::set_opened_process_info(process_selector_view_data, &app_context_for_response, process_open_response.opened_process_info)
            });

            if !did_dispatch {
                if let Some(mut process_selector_view_data) =
                    process_selector_view_data_for_response.write("Process selector view data select process dispatch failure")
                {
                    process_selector_view_data.is_opening_process = false;
                    process_selector_view_data.open_process_request_started_at = None;
                }

                Self::request_repaint(&app_context);
            }
        } else {
            Self::set_opened_process_info(process_selector_view_data, &app_context, None)
        }
    }

    pub fn set_opened_process_info(
        process_selector_view_data_dependency: Dependency<ProcessSelectorViewData>,
        app_context: &Arc<AppContext>,
        opened_process: Option<OpenedProcessInfo>,
    ) {
        let mut process_selector_view_data = match process_selector_view_data_dependency.write("Process selector view data set opened process info") {
            Some(process_selector_view_data) => process_selector_view_data,
            None => return,
        };

        process_selector_view_data.is_opening_process = false;
        process_selector_view_data.open_process_request_started_at = None;
        process_selector_view_data.opened_process = opened_process;
        process_selector_view_data.cached_icon = process_selector_view_data
            .opened_process
            .as_ref()
            .and_then(|opened_process| {
                process_selector_view_data
                    .icon_cache
                    .get(&opened_process.get_process_id_raw())
                    .cloned()
            });

        drop(process_selector_view_data);

        if let Some(opened_process_id) = process_selector_view_data_dependency
            .read("Process selector view data opened process icon lookup")
            .and_then(|process_selector_view_data| {
                process_selector_view_data
                    .opened_process
                    .as_ref()
                    .map(|opened_process| opened_process.get_process_id_raw())
            })
        {
            Self::request_process_icons_if_needed(process_selector_view_data_dependency, app_context.clone(), vec![opened_process_id]);
        }

        Self::request_repaint(app_context);
    }

    pub fn create_and_cache_icon(
        process_selector_view_data: &mut WriteGuard<'_, ProcessSelectorViewData>,
        app_context: &Arc<AppContext>,
        process_id: u32,
        icon: &ProcessIcon,
    ) -> TextureHandle {
        if let Some(texture_handle) = process_selector_view_data.icon_cache.get(&process_id).cloned() {
            return texture_handle;
        }

        let size = [icon.get_width() as usize, icon.get_height() as usize];
        let texture = app_context.context.load_texture(
            &format!("process_icon_{process_id}"),
            ColorImage::from_rgba_unmultiplied(size, icon.get_bytes_rgba()),
            TextureOptions::default(),
        );

        process_selector_view_data
            .icon_cache
            .insert(process_id, texture.clone());

        texture
    }

    pub fn get_cached_icon(
        &self,
        process_id: u32,
    ) -> Option<TextureHandle> {
        self.icon_cache.get(&process_id).cloned()
    }

    pub fn request_process_icons_if_needed(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: Arc<AppContext>,
        process_ids: Vec<u32>,
    ) {
        if !Self::ENABLE_LAZY_PROCESS_ICONS {
            return;
        }

        let process_ids_to_request = match process_selector_view_data.write("Process selector view data request process icons") {
            Some(mut process_selector_view_data) => {
                let process_ids_to_request = process_ids
                    .into_iter()
                    .filter(|process_id| {
                        let can_retry_missing_icon = process_selector_view_data
                            .missing_icon_process_ids
                            .get(process_id)
                            .map(|last_failed_icon_request_at| last_failed_icon_request_at.elapsed() >= Self::PROCESS_ICON_RETRY_COOLDOWN)
                            .unwrap_or(true);

                        !process_selector_view_data.icon_cache.contains_key(process_id)
                            && !process_selector_view_data
                                .loading_icon_process_ids
                                .contains(process_id)
                            && can_retry_missing_icon
                    })
                    .collect::<Vec<_>>();

                for process_id in &process_ids_to_request {
                    process_selector_view_data
                        .loading_icon_process_ids
                        .insert(*process_id);
                    process_selector_view_data
                        .missing_icon_process_ids
                        .remove(process_id);
                }

                process_ids_to_request
            }
            None => Vec::new(),
        };

        if process_ids_to_request.is_empty() {
            return;
        }

        let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();
        let process_selector_view_data_for_response = process_selector_view_data.clone();
        thread::spawn(move || {
            for process_id_chunk in process_ids_to_request.chunks(Self::PROCESS_ICON_REQUEST_BATCH_SIZE) {
                let process_id_chunk = process_id_chunk.to_vec();
                let process_icon_request = ProcessIconRequest {
                    process_ids: process_id_chunk.clone(),
                };
                let app_context_for_response = app_context.clone();
                let process_selector_view_data_for_chunk = process_selector_view_data.clone();
                let did_dispatch = process_icon_request.send(&engine_unprivileged_state, move |process_icon_response| {
                    Self::apply_process_icon_response(
                        process_selector_view_data_for_chunk,
                        &app_context_for_response,
                        process_icon_response.process_icons,
                    );
                });

                if !did_dispatch {
                    if let Some(mut process_selector_view_data) =
                        process_selector_view_data_for_response.write("Process selector view data request process icons dispatch failure")
                    {
                        for process_id in &process_id_chunk {
                            process_selector_view_data
                                .loading_icon_process_ids
                                .remove(process_id);
                        }
                    }

                    Self::request_repaint(&app_context);
                }
            }
        });
    }

    fn apply_process_icon_response(
        process_selector_view_data: Dependency<ProcessSelectorViewData>,
        app_context: &Arc<AppContext>,
        process_icons: Vec<ProcessIconEntry>,
    ) {
        let mut process_selector_view_data = match process_selector_view_data.write("Process selector view data apply process icon response") {
            Some(process_selector_view_data) => process_selector_view_data,
            None => return,
        };

        for process_icon_entry in process_icons {
            let process_id = process_icon_entry.process_id;
            process_selector_view_data
                .loading_icon_process_ids
                .remove(&process_id);

            if let Some(process_icon) = process_icon_entry.process_icon {
                let texture_handle = Self::create_and_cache_icon(&mut process_selector_view_data, app_context, process_id, &process_icon);
                process_selector_view_data
                    .missing_icon_process_ids
                    .remove(&process_id);

                if process_selector_view_data
                    .opened_process
                    .as_ref()
                    .is_some_and(|opened_process| opened_process.get_process_id_raw() == process_id)
                {
                    process_selector_view_data.cached_icon = Some(texture_handle);
                }
            } else {
                process_selector_view_data
                    .missing_icon_process_ids
                    .insert(process_id, Instant::now());

                if process_selector_view_data
                    .opened_process
                    .as_ref()
                    .is_some_and(|opened_process| opened_process.get_process_id_raw() == process_id)
                {
                    process_selector_view_data.cached_icon = None;
                }
            }
        }

        drop(process_selector_view_data);
        Self::request_repaint(app_context);
    }

    pub fn set_windowed_process_list(
        process_selector_view_data: &mut WriteGuard<'_, ProcessSelectorViewData>,
        new_list: Vec<ProcessInfo>,
    ) {
        let next_windowed_process_list = Self::sort_processes_case_insensitive_then_process_id(new_list);
        Self::clear_missing_icon_retry_state_for_processes(process_selector_view_data, &next_windowed_process_list);

        let removed = Self::diff_pids(&process_selector_view_data.windowed_process_list, &next_windowed_process_list);

        process_selector_view_data.windowed_process_list = next_windowed_process_list;
        process_selector_view_data.refresh_shortcut_dropdown_process_list();

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
        new_list: Vec<ProcessInfo>,
    ) {
        Self::clear_missing_icon_retry_state_for_processes(process_selector_view_data, &new_list);
        let removed_full = Self::diff_pids(&process_selector_view_data.full_process_list, &new_list);
        let next_windowed_process_list = Self::collect_windowed_processes_from_full_list(&new_list);
        let removed_windowed = Self::diff_pids(&process_selector_view_data.windowed_process_list, &next_windowed_process_list);

        process_selector_view_data.full_process_list = new_list;
        process_selector_view_data.windowed_process_list = next_windowed_process_list;
        process_selector_view_data.refresh_shortcut_dropdown_process_list();

        // Remove icons for processes no longer present.
        let removed = &removed_full | &removed_windowed;
        Self::remove_from_cache(process_selector_view_data, &removed);
    }

    fn apply_process_list_response(
        process_selector_view_data: &mut WriteGuard<'_, ProcessSelectorViewData>,
        require_windowed: bool,
        processes: Vec<ProcessInfo>,
    ) {
        if require_windowed {
            process_selector_view_data.is_awaiting_windowed_process_list = false;
            process_selector_view_data.windowed_process_list_request_started_at = None;
            Self::set_windowed_process_list(process_selector_view_data, processes);
        } else {
            process_selector_view_data.is_awaiting_full_process_list = false;
            process_selector_view_data.full_process_list_request_started_at = None;
            Self::set_full_process_list(process_selector_view_data, processes);
        }
    }

    fn collect_windowed_processes_from_full_list(full_processes: &[ProcessInfo]) -> Vec<ProcessInfo> {
        let mut windowed_processes = full_processes
            .iter()
            .filter(|process_info| process_info.get_is_windowed())
            .cloned()
            .collect::<Vec<_>>();

        windowed_processes.sort_by(|left_process_info, right_process_info| {
            let name_ordering = left_process_info
                .get_name()
                .to_ascii_lowercase()
                .cmp(&right_process_info.get_name().to_ascii_lowercase());
            if name_ordering.is_eq() {
                left_process_info
                    .get_process_id_raw()
                    .cmp(&right_process_info.get_process_id_raw())
            } else {
                name_ordering
            }
        });

        windowed_processes
    }

    /// Applies deterministic ordering by process name then process ID.
    fn sort_processes_case_insensitive_then_process_id(mut processes: Vec<ProcessInfo>) -> Vec<ProcessInfo> {
        processes.sort_by(|left_process_info, right_process_info| {
            let name_ordering = left_process_info
                .get_name()
                .to_ascii_lowercase()
                .cmp(&right_process_info.get_name().to_ascii_lowercase());
            if name_ordering.is_eq() {
                left_process_info
                    .get_process_id_raw()
                    .cmp(&right_process_info.get_process_id_raw())
            } else {
                name_ordering
            }
        });

        processes
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
        process_selector_view_data
            .loading_icon_process_ids
            .retain(|process_id| !removed.contains(process_id));
        process_selector_view_data
            .missing_icon_process_ids
            .retain(|process_id, _| !removed.contains(process_id));
    }

    fn clear_missing_icon_retry_state_for_processes(
        process_selector_view_data: &mut WriteGuard<'_, ProcessSelectorViewData>,
        processes: &[ProcessInfo],
    ) {
        for process_info in processes {
            process_selector_view_data
                .missing_icon_process_ids
                .remove(&process_info.get_process_id_raw());
        }
    }

    fn refresh_shortcut_dropdown_process_list(&mut self) {
        let next_shortcut_dropdown_process_list = if !Self::IS_ANDROID_TARGET {
            // Desktop shortcut dropdowns stay windowed-only.
            Self::sort_processes_case_insensitive_then_process_id(self.windowed_process_list.clone())
        } else if self.show_windowed_processes_only {
            Self::choose_shortcut_dropdown_windowed_candidates(&self.windowed_process_list, &self.full_process_list)
        } else {
            Self::sort_processes_case_insensitive_then_process_id(self.full_process_list.clone())
        };

        self.shortcut_dropdown_process_list = next_shortcut_dropdown_process_list;
        self.shortcut_dropdown_refresh_nonce = self.shortcut_dropdown_refresh_nonce.saturating_add(1);
    }

    fn choose_shortcut_dropdown_windowed_candidates(
        windowed_processes: &[ProcessInfo],
        full_processes: &[ProcessInfo],
    ) -> Vec<ProcessInfo> {
        if !windowed_processes.is_empty() {
            return windowed_processes.to_vec();
        }

        if !Self::IS_ANDROID_TARGET {
            return Vec::new();
        }

        if full_processes.is_empty() {
            return windowed_processes.to_vec();
        }

        let primary_package_processes = Self::extract_primary_package_processes(full_processes);
        if !primary_package_processes.is_empty() {
            log::warn!(
                "Shortcut dropdown fallback activated: using {} primary package processes because windowed results are empty.",
                primary_package_processes.len(),
            );

            return primary_package_processes;
        }

        log::warn!("Shortcut dropdown fallback skipped: no primary package candidates and windowed list is empty.");
        Vec::new()
    }

    fn extract_primary_package_processes(full_processes: &[ProcessInfo]) -> Vec<ProcessInfo> {
        let mut sorted_full_processes = Self::sort_processes_case_insensitive_then_process_id(full_processes.to_vec());
        let mut seen_process_names = HashSet::new();

        sorted_full_processes.retain(|process_info| {
            let process_name = process_info.get_name();
            let is_primary_package_name = process_name.contains('.') && !process_name.contains(':');
            if !is_primary_package_name {
                return false;
            }

            seen_process_names.insert(process_name.to_ascii_lowercase())
        });

        sorted_full_processes
    }

    fn clear_stale_request_state_for_now(
        &mut self,
        current_instant: Instant,
    ) {
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
