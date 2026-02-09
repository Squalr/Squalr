use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use sysinfo::{Pid, ProcessesToUpdate, System};

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
                let process_name = process.name().to_string_lossy().to_string();
                let process_is_windowed = Self::is_process_windowed(process_id);
                let process_icon = if options.fetch_icons { Self::get_icon(process_id) } else { None };

                let process_info = ProcessInfo::new(process_id.as_u32(), process_name, process_is_windowed, process_icon);

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
