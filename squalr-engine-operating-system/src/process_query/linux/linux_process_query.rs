use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::processes::process_info::ProcessInfo;

pub struct LinuxProcessQuery;

impl ProcessQueryer for LinuxProcessQuery {
    fn start_monitoring() -> Result<(), ProcessQueryError> {
        // Linux process query currently exposes immediate operations only.
        Ok(())
    }

    fn stop_monitoring() -> Result<(), ProcessQueryError> {
        // Linux process query currently exposes immediate operations only.
        Ok(())
    }

    fn open_process(_process_info: &ProcessInfo) -> Result<OpenedProcessInfo, ProcessQueryError> {
        Err(ProcessQueryError::not_implemented("open_process", "linux"))
    }

    fn close_process(_handle: u64) -> Result<(), ProcessQueryError> {
        Err(ProcessQueryError::not_implemented("close_process", "linux"))
    }

    fn get_processes(_options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        vec![]
    }
}
