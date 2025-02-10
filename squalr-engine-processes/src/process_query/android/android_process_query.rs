use crate::process_info::{Bitness, OpenedProcessInfo, ProcessIcon, ProcessInfo};
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use once_cell::sync::Lazy;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::collections::HashSet;
use std::fs;
use std::sync::{Arc, RwLock};
use sysinfo::{Pid, System};

/// Minimum UID for user-installed apps.
const MIN_USER_UID: u32 = 10000;

pub struct AndroidProcessQuery {}

impl AndroidProcessQuery {
    /// Checks if a process belongs to a user app (UID ≥ 10000).
    fn is_user_app(process_id: u32) -> bool {
        let status_path = format!("/proc/{}/status", process_id);
        if let Ok(status) = fs::read_to_string(status_path) {
            for line in status.lines() {
                if line.starts_with("Uid:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        if let Ok(uid) = parts[1].parse::<u32>() {
                            return uid >= MIN_USER_UID;
                        }
                    }
                }
            }
        }
        false
    }

    /// Finds the PIDs of `zygote` and `zygote64` for parent checking.
    fn find_zygote_process_ids() -> HashSet<u32> {
        let mut zygote_process_ids = HashSet::new();
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(process_id_str) = entry.file_name().into_string() {
                    if process_id_str.chars().all(|c| c.is_digit(10)) {
                        let cmd_path = format!("/proc/{}/cmdline", process_id_str);
                        if let Ok(cmd) = fs::read_to_string(cmd_path) {
                            let cmd_trimmed = cmd.trim_end_matches('\0');
                            if cmd_trimmed == "zygote" || cmd_trimmed == "zygote64" {
                                if let Ok(pid) = process_id_str.parse::<u32>() {
                                    zygote_process_ids.insert(pid);
                                }
                            }
                        }
                    }
                }
            }
        }

        zygote_process_ids
    }
}

static WINDOWED_PROCESSES: Lazy<RwLock<HashSet<u32>>> = Lazy::new(|| RwLock::new(HashSet::new()));

impl ProcessQueryer for AndroidProcessQuery {
    // Android has no concept of opening a process -- do nothing, return 0 for handle.
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        Ok(OpenedProcessInfo {
            process_id: process_info.process_id,
            name: process_info.name.clone(),
            handle: 0,
            bitness: Bitness::Bit64,
            icon: process_info.icon.clone(),
        })
    }

    // Android has no concept of closing a process -- do nothing.
    fn close_process(_handle: u64) -> Result<(), String> {
        Ok(())
    }

    fn get_processes(
        options: ProcessQueryOptions,
        system: Arc<RwLock<System>>,
    ) -> Vec<ProcessInfo> {
        Logger::get_instance().log(LogLevel::Info, "Fetching processes...", None);
        let system_guard = match system.read() {
            Ok(guard) => guard,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("Failed to acquire system read lock: {}", e), None);
                return Vec::new();
            }
        };

        let mut windowed_process_ids = HashSet::new();
        let zygote_process_ids = Self::find_zygote_process_ids();

        for (process_id, process) in system_guard.processes() {
            let parent_process_id = process
                .parent()
                .map(|parent_process_id| parent_process_id.as_u32())
                .unwrap_or(0);
            let is_zygote_spawned_process = zygote_process_ids.contains(&parent_process_id);
            let is_user_app = Self::is_user_app(process_id.as_u32());

            if is_zygote_spawned_process && is_user_app {
                windowed_process_ids.insert(process_id.as_u32());
            }
        }

        // Persist the windowed process list so that it can be used in `is_process_windowed()`.
        {
            let mut windowed_process_guard = match WINDOWED_PROCESSES.write() {
                Ok(guard) => guard,
                Err(err) => {
                    Logger::get_instance().log(LogLevel::Error, &format!("Failed to acquire windowed processes write lock: {}", err), None);
                    return Vec::new();
                }
            };
            *windowed_process_guard = windowed_process_ids;
        }

        let mut results = Vec::new();

        for (process_id, process) in system_guard.processes() {
            let process_id_u32 = process_id.as_u32();
            let name = process.name().to_string_lossy().to_string();
            let is_windowed = Self::is_process_windowed(process_id);
            let process_info = ProcessInfo {
                process_id: process_id_u32,
                name,
                is_windowed,
                icon: None,
            };
            let mut matches = true;

            if let Some(ref term) = options.search_name {
                if options.match_case {
                    matches &= process_info.name.contains(term);
                } else {
                    matches &= process_info.name.to_lowercase().contains(&term.to_lowercase());
                }
            }

            if let Some(required_process_id) = options.required_process_id {
                matches &= process_info.process_id == required_process_id.as_u32();
            }

            if options.require_windowed {
                matches &= process_info.is_windowed;
            }

            if matches {
                results.push(process_info);
            }

            if let Some(limit) = options.limit {
                if results.len() >= limit as usize {
                    break;
                }
            }
        }

        results
    }

    fn is_process_windowed(process_id: &Pid) -> bool {
        WINDOWED_PROCESSES
            .read()
            .map_or(false, |set| set.contains(&process_id.as_u32()))
    }

    fn get_icon(_process_id: &Pid) -> Option<ProcessIcon> {
        None
    }
}
