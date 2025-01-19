use crate::process_info::{Bitness, OpenedProcessInfo, ProcessIcon, ProcessInfo};
use crate::process_query::process_queryer::{ProcessQueryOptions, ProcessQueryer};
use once_cell::sync::Lazy;
use regex::Regex;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::privileges::android::android_super_user::AndroidSuperUser;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use sysinfo::{Pid, System};

static SESSION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"mSession=Session\{\w+\s+(\d+):u0a\d+}").unwrap());
/// Caches the overall process info so we don’t reconstruct everything every time
static PROCESS_CACHE: Lazy<RwLock<HashMap<Pid, ProcessInfo>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// A single global cache for windowed PIDs, plus the time we last fetched them.
/// This replaces per-PID TTL caching.  
static WINDOW_INFO_CACHE: Lazy<RwLock<WindowInfoCache>> = Lazy::new(|| {
    RwLock::new(WindowInfoCache {
        last_fetch: Instant::now()
            .checked_sub(Duration::from_secs(20)) // Force an immediate refresh on first use
            .unwrap_or_else(Instant::now),
        windowed_pids: HashSet::new(),
    })
});

/// We only refresh the window info once per this TTL.
const WINDOW_INFO_TTL: Duration = Duration::from_secs(10);

/// Holds the “which PIDs are windowed?” snapshot + when we last fetched it
struct WindowInfoCache {
    last_fetch: Instant,
    windowed_pids: HashSet<Pid>,
}

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

    /// Refresh the global window info cache if older than `WINDOW_INFO_TTL`.
    fn refresh_windowed_pids_if_needed() {
        let mut cache = WINDOW_INFO_CACHE.write().unwrap();

        // Skip if it's still fresh
        if cache.last_fetch.elapsed() < WINDOW_INFO_TTL {
            return;
        }

        // Acquire su and execute dumpsys
        let android_su = AndroidSuperUser::get_instance();
        let mut su = match android_su.write() {
            Ok(su_guard) => su_guard,
            Err(e) => {
                // handle lock error
                return;
            }
        };

        let output = match su.execute_command("dumpsys window windows") {
            Ok(out) => out,
            Err(e) => {
                // handle dumpsys error
                return;
            }
        };

        // We'll parse the entire output
        let mut new_windowed_pids = HashSet::new();
        let mut current_pid: Option<Pid> = None;

        for line in output {
            if line.starts_with("Window #") {
                // Start of a new window block => reset current_pid
                current_pid = None;
            }

            // Grab PID from `mSession=Session{someHex somePID:u0aSomething}`
            if let Some(cap) = SESSION_REGEX.captures(&line) {
                if let Ok(parsed_pid) = cap[1].parse::<u32>() {
                    new_windowed_pids.insert(Pid::from_u32(parsed_pid));
                }
            }
        }

        cache.windowed_pids = new_windowed_pids;
        cache.last_fetch = Instant::now();
    }

    /// Check if a given PID is in our globally cached “windowed” set.
    fn is_process_windowed_impl(process_id: &Pid) -> bool {
        let cache = WINDOW_INFO_CACHE.read().unwrap();
        cache.windowed_pids.contains(process_id)
    }
}

impl ProcessQueryer for AndroidProcessQuery {
    /// "Open" a process on Android. No real handle concept, so just return a dummy handle (0).
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        Ok(OpenedProcessInfo {
            pid: process_info.pid,
            name: process_info.name.clone(),
            handle: 0,
            bitness: Bitness::Bit64,
            icon: None,
        })
    }

    /// "Close" a process on Android. Again, no real handle, so do nothing.
    fn close_process(_handle: u64) -> Result<(), String> {
        Ok(())
    }

    /// Get a list of processes by running `ps -A`. We ignore icons for Android here.
    fn get_processes(
        options: ProcessQueryOptions,
        system: Arc<RwLock<System>>,
    ) -> Vec<ProcessInfo> {
        // Before we parse lines, refresh the “windowed” set once (at most) if needed.
        Self::refresh_windowed_pids_if_needed();

        let android_su = AndroidSuperUser::get_instance();

        let mut su = match android_su.write() {
            Ok(su_guard) => su_guard,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, "Failed to acquire write lock on AndroidSuperUser.", Some(&e.to_string()));
                return Vec::new();
            }
        };

        // Attempt to run `ps -A`
        let lines = match su.execute_command("ps -A") {
            Ok(output) => output,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, "Failed to execute `ps -A` via `su`.", Some(&e.to_string()));
                return Vec::new();
            }
        };

        // The first line in many shells is a header. Skip it if we have at least 2 lines.
        let without_header = if lines.len() > 1 {
            &lines[1..]
        } else {
            // If only 1 line, it’s not standard. But let's avoid panic.
            &lines[..]
        };

        let processes: Vec<ProcessInfo> = without_header
            .iter()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                // For typical Android `ps -A` output: parts[1] = PID, parts[8] = process name
                if parts.len() >= 9 {
                    let pid_num: i32 = match parts[1].parse() {
                        Ok(val) => val,
                        Err(e) => {
                            Logger::get_instance().log(LogLevel::Error, "Failed to parse PID in `ps -A` output line.", Some(&e.to_string()));
                            return None;
                        }
                    };

                    let name = parts[8].to_string();
                    let pid = Pid::from_u32(pid_num as u32);
                    let is_windowed = Self::is_process_windowed_impl(&pid);

                    // Construct the ProcessInfo
                    let process_info = ProcessInfo {
                        pid,
                        name,
                        is_windowed,
                        icon: None,
                    };

                    // Apply filters from `ProcessQueryOptions`
                    let mut matches = true;
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

                    if options.require_windowed {
                        matches &= process_info.is_windowed;
                    }

                    matches.then_some(process_info)
                } else {
                    None
                }
            })
            .take(options.limit.unwrap_or(usize::MAX as u64) as usize)
            .collect();

        processes
    }

    fn is_process_windowed(process_id: &Pid) -> bool {
        Self::refresh_windowed_pids_if_needed();
        Self::is_process_windowed_impl(process_id)
    }

    fn get_icon(_process_id: &Pid) -> Option<ProcessIcon> {
        // Icons not (yet) supported on Android
        None
    }
}
