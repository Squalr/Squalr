use crate::process_info::{Bitness, OpenedProcessInfo, ProcessIcon, ProcessInfo};
use crate::process_query::process_queryer::{ProcessQueryOptions, ProcessQueryer};
use once_cell::sync::Lazy;
use regex::Regex;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use sysinfo::{Pid, System};

/// Caches the overall process info so we don’t reconstruct everything every time
static PROCESS_CACHE: Lazy<RwLock<HashMap<Pid, ProcessInfo>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// A single global cache for “windowed PIDs,” plus the time we last fetched them.
static WINDOW_INFO_CACHE: Lazy<RwLock<WindowInfoCache>> = Lazy::new(|| {
    RwLock::new(WindowInfoCache {
        last_fetch: Instant::now()
            .checked_sub(Duration::from_secs(20)) // Force an immediate refresh on first use
            .unwrap_or_else(Instant::now),
        windowed_pids: HashSet::new(),
    })
});

/// We only refresh the “window info” once per this TTL.
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
            cache.insert(pid, ProcessInfo {
                pid: pid.as_u32(),
                name,
                is_windowed,
                icon,
            });
        }
    }

    fn get_from_cache(pid: &Pid) -> Option<ProcessInfo> {
        PROCESS_CACHE
            .read()
            .ok()
            .and_then(|cache| cache.get(pid).cloned())
    }

    /// Refresh the global window info cache if older than `WINDOW_INFO_TTL`.
    ///
    /// In this version, we have removed `dumpsys` calls. If you need to truly detect
    /// "windowed" processes on Android, you'll need an alternative approach that
    /// doesn't rely on shelling out. For now, this is effectively a no-op.
    fn refresh_windowed_pids_if_needed() {
        let mut cache = WINDOW_INFO_CACHE.write().unwrap();

        // Skip if it's still fresh
        if cache.last_fetch.elapsed() < WINDOW_INFO_TTL {
            return;
        }

        // Because we don't use `dumpsys` now, we have no direct way to detect
        // windowed processes. We'll clear or leave empty:
        cache.windowed_pids.clear();

        // If needed, you could parse other `/proc` data or system stats to infer
        // which processes have UI presence.

        cache.last_fetch = Instant::now();
    }

    /// Check if a given PID is in our globally cached “windowed” set.
    /// Right now, this will always return `false` unless you add logic to
    /// populate `windowed_pids`.
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

    /// Get a list of processes by scanning `/proc` via the `sysinfo` crate.
    /// We also attempt to update the windowed-pid cache, though for now it
    /// is effectively a no-op without `dumpsys`.
    fn get_processes(
        options: ProcessQueryOptions,
        system: Arc<RwLock<System>>,
    ) -> Vec<ProcessInfo> {
        // Refresh cached “windowed” set once (at most) if needed
        Self::refresh_windowed_pids_if_needed();

        // Refresh process list from /proc
        let mut sys = system.write().unwrap();

        let mut results = Vec::new();

        for (pid, proc_) in sys.processes() {
            let pid_u32 = pid.as_u32();
            let name = proc_.name().to_owned();
            let is_windowed = Self::is_process_windowed_impl(pid);

            // Construct a candidate ProcessInfo
            let process_info = ProcessInfo {
                pid: pid_u32,
                name: name.to_string_lossy().to_string(),
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
                matches &= process_info.pid == required_pid.as_u32();
            }

            if options.require_windowed {
                matches &= process_info.is_windowed;
            }

            if matches {
                results.push(process_info);
            }

            // Respect the limit if specified
            if let Some(limit) = options.limit {
                if results.len() >= limit as usize {
                    break;
                }
            }
        }

        results
    }

    /// In this implementation, we don't have a good way to detect "windowed" from /proc,
    /// so we simply consult our `WINDOW_INFO_CACHE`—which is effectively empty
    /// unless you implement your own logic for populating it.
    fn is_process_windowed(process_id: &Pid) -> bool {
        Self::refresh_windowed_pids_if_needed();
        Self::is_process_windowed_impl(process_id)
    }

    /// Icons not (yet) supported on Android via /proc, so return `None`.
    fn get_icon(_process_id: &Pid) -> Option<ProcessIcon> {
        None
    }
}
