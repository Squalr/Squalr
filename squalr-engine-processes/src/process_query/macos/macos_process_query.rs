use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use once_cell::sync::Lazy;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use std::collections::HashMap;
use std::sync::RwLock;
use sysinfo::{Pid, ProcessesToUpdate, System};

static PROCESS_CACHE: Lazy<RwLock<HashMap<Pid, ProcessInfo>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub struct MacOsProcessQuery {}

impl MacOsProcessQuery {
    fn is_process_windowed(_process_id: &Pid) -> bool {
        // Proper implementation requires CoreGraphics / Cocoa.
        false
    }

    fn get_icon(_process_id: &Pid) -> Option<ProcessIcon> {
        // Requires NSWorkspace / NSImage.
        None
    }

    fn update_cache(
        process_id: Pid,
        info: &ProcessInfo,
    ) {
        if let Ok(mut cache) = PROCESS_CACHE.write() {
            cache.insert(process_id, info.clone());
        }
    }

    fn get_from_cache(process_id: &Pid) -> Option<ProcessInfo> {
        PROCESS_CACHE
            .read()
            .ok()
            .and_then(|cache| cache.get(process_id).cloned())
    }
}

impl ProcessQueryer for MacOsProcessQuery {
    fn start_monitoring() -> Result<(), ProcessQueryError> {
        Ok(())
    }

    fn stop_monitoring() -> Result<(), ProcessQueryError> {
        Ok(())
    }

    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, ProcessQueryError> {
        Ok(OpenedProcessInfo::new(
            process_info.get_process_id_raw(),
            process_info.get_name().to_string(),
            0,
            Bitness::Bit64,
            process_info.get_icon().clone(),
        ))
    }

    fn close_process(_handle: u64) -> Result<(), ProcessQueryError> {
        Ok(())
    }

    fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        let mut system = System::new_all();

        system.refresh_processes(ProcessesToUpdate::All, true);

        system
            .processes()
            .iter()
            .filter_map(|(process_id, process)| {
                let process_info = if let Some(cached) = Self::get_from_cache(process_id) {
                    if options.fetch_icons && cached.get_icon().is_none() {
                        let mut updated = cached.clone();
                        updated.set_icon(Self::get_icon(process_id));
                        Self::update_cache(*process_id, &updated);
                        updated
                    } else {
                        cached
                    }
                } else {
                    let icon = if options.fetch_icons { Self::get_icon(process_id) } else { None };

                    let info = ProcessInfo::new(
                        process_id.as_u32(),
                        process.name().to_string_lossy().to_string(),
                        Self::is_process_windowed(process_id),
                        icon,
                    );

                    Self::update_cache(*process_id, &info);
                    info
                };

                let mut matches = true;

                if options.require_windowed {
                    matches &= process_info.get_is_windowed();
                }

                if let Some(ref term) = options.search_name {
                    if options.match_case {
                        matches &= process_info.get_name().contains(term);
                    } else {
                        matches &= process_info
                            .get_name()
                            .to_lowercase()
                            .contains(&term.to_lowercase());
                    }
                }

                if let Some(required_pid) = options.required_process_id {
                    matches &= process_info.get_process_id_raw() == required_pid.as_u32();
                }

                matches.then_some(process_info)
            })
            .take(options.limit.unwrap_or(u64::MAX) as usize)
            .collect()
    }
}
