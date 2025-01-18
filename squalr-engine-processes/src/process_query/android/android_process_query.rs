use crate::process_info::{Bitness, OpenedProcessInfo, ProcessIcon, ProcessInfo};
use crate::process_query::process_queryer::{ProcessQueryOptions, ProcessQueryer};
use once_cell::sync::Lazy;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use sysinfo::{Pid, System};

static PROCESS_CACHE: Lazy<RwLock<HashMap<Pid, ProcessInfo>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub struct AndroidProcessQuery {}

impl AndroidProcessQuery {
    pub fn new() -> Self {
        AndroidProcessQuery {}
    }

    fn update_cache(
        pid: Pid,
        name: String,
        is_windowed: bool,
        icon: Option<ProcessIcon>,
    ) {
        if let Ok(mut cache) = PROCESS_CACHE.write() {
            cache.insert(pid, ProcessInfo { pid, name, is_windowed, icon });
        }
    }

    fn get_from_cache(pid: &Pid) -> Option<ProcessInfo> {
        PROCESS_CACHE
            .read()
            .ok()
            .and_then(|cache| cache.get(pid).cloned())
    }
}

impl ProcessQueryer for AndroidProcessQuery {
    /// "Open" a process on Android. Since there's no traditional "handle" concept,
    /// we simply copy fields and return 0 for the handle.
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        Ok(OpenedProcessInfo {
            pid: process_info.pid,
            name: process_info.name.clone(),
            handle: 0,
            bitness: Bitness::Bit64,
            icon: None,
        })
    }

    /// "Close" a process on Android. Again, no handle needed, so do nothing.
    fn close_process(_handle: u64) -> Result<(), String> {
        Ok(())
    }

    /// Get process list from sysinfo on Android.
    /// Fills in the basic fields; ignores icons and windowed state.
    fn get_processes(
        options: ProcessQueryOptions,
        system: Arc<RwLock<System>>,
    ) -> Vec<ProcessInfo> {
        let system_guard = match system.read() {
            Ok(guard) => guard,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to acquire system read lock: {}", e), None);
                return Vec::new();
            }
        };

        // Process and filter in a single pass, using cache when possible
        let filtered_processes: Vec<ProcessInfo> = system_guard
            .processes()
            .iter()
            .filter_map(|(pid, process)| {
                // Try to get from cache first
                let process_info = if let Some(cached_info) = Self::get_from_cache(pid) {
                    // If icons are required but not in cache, update the icon
                    if options.fetch_icons && cached_info.icon.is_none() {
                        let mut updated_info = cached_info.clone();
                        updated_info.icon = Self::get_icon(pid);
                        // Update cache with new icon
                        Self::update_cache(*pid, updated_info.name.clone(), updated_info.is_windowed, updated_info.icon.clone());
                        updated_info
                    } else {
                        cached_info
                    }
                } else {
                    // Create new ProcessInfo and cache it
                    let new_info = ProcessInfo {
                        pid: *pid,
                        name: process.name().to_string_lossy().into_owned(),
                        is_windowed: Self::is_process_windowed(pid),
                        icon: if options.fetch_icons { Self::get_icon(pid) } else { None },
                    };
                    Self::update_cache(*pid, new_info.name.clone(), new_info.is_windowed, new_info.icon.clone());
                    new_info
                };

                let mut matches = true;

                // Apply filters
                if options.require_windowed {
                    matches &= process_info.is_windowed;
                }

                if let Some(ref term) = options.search_name {
                    if options.match_case {
                        matches &= process_info.name.contains(term);
                    } else {
                        matches &= process_info.name.to_lowercase().contains(&term.to_lowercase());
                    }
                }

                if let Some(required_pid) = options.required_pid {
                    matches &= process_info.pid == required_pid;
                }

                matches.then_some(process_info)
            })
            .take(options.limit.unwrap_or(usize::MAX as u64) as usize)
            .collect();

        filtered_processes
    }

    fn is_process_windowed(_process_id: &Pid) -> bool {
        true
    }

    fn get_icon(_process_id: &Pid) -> Option<ProcessIcon> {
        None
    }
}
