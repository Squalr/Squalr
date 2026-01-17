use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;
use std::sync::{Arc, RwLock};
use sysinfo::{Pid, System};

pub struct MacOsProcessQuery {
    system: System,
}

impl MacOsProcessQuery {
    pub fn new() -> Self {
        MacOsProcessQuery { system: System::new_all() }
    }

    fn is_process_windowed(process_id: &Pid) -> bool {
        false
    }

    fn get_icon(process_id: &Pid) -> Option<ProcessIcon> {
        None
    }
}

impl ProcessQueryer for MacOsProcessQuery {
    fn start_monitoring() -> Result<(), String> {
        Err("Not implemented".into())
    }

    fn stop_monitoring() -> Result<(), String> {
        Err("Not implemented".into())
    }

    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        Err("Not implemented".into())
    }

    fn close_process(handle: u64) -> Result<(), String> {
        Err("Not implemented".into())
    }

    fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        vec![]
    }
}
