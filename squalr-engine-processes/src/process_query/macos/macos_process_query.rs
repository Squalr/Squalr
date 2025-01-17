use crate::process_info::ProcessIcon;
use crate::process_info::{OpenedProcessInfo, ProcessInfo};
use crate::process_query::process_queryer::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use std::sync::{Arc, RwLock};
use sysinfo::{Pid, System};

pub struct MacosProcessQuery {
    system: System,
}

impl MacosProcessQuery {
    pub fn new() -> Self {
        MacosProcessQuery { system: System::new_all() }
    }
}

impl ProcessQueryer for MacosProcessQuery {
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        Err("Not implemented".into())
    }

    fn close_process(handle: u64) -> Result<(), String> {
        Err("Not implemented".into())
    }

    fn get_processes(
        options: ProcessQueryOptions,
        system: Arc<RwLock<System>>,
    ) -> Vec<ProcessInfo> {
        vec![]
    }

    fn is_process_windowed(process_id: &Pid) -> bool {
        false
    }

    fn get_icon(process_id: &Pid) -> Option<ProcessIcon> {
        None
    }
}
