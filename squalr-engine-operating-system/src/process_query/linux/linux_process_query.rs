use crate::process_query::process_query_error::ProcessQueryError;
use crate::process_query::process_query_options::ProcessQueryOptions;
use crate::process_query::process_queryer::ProcessQueryer;
use squalr_engine_api::structures::memory::bitness::Bitness;
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

    fn get_processes(_options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::LinuxProcessQuery;
    use crate::process_query::process_queryer::ProcessQueryer;
    use squalr_engine_api::structures::memory::bitness::Bitness;
    use squalr_engine_api::structures::processes::process_info::ProcessInfo;

    #[test]
    fn open_process_returns_opened_process_info_with_expected_fields() {
        let process_id = 1337;
        let process_name = "linux-target".to_string();
        let process_info = ProcessInfo::new(process_id, process_name.clone(), false, None);

        let opened_process_info = LinuxProcessQuery::open_process(&process_info).expect("linux open_process should return an opened process info object.");

        assert_eq!(opened_process_info.get_process_id_raw(), process_id);
        assert_eq!(opened_process_info.get_name(), process_name);
        assert_eq!(opened_process_info.get_handle(), 0);
        assert_eq!(opened_process_info.get_bitness(), Bitness::Bit64);
        assert!(opened_process_info.get_icon().is_none());
    }

    #[test]
    fn close_process_returns_ok_for_any_handle() {
        let close_result = LinuxProcessQuery::close_process(42);

        assert!(close_result.is_ok());
    }
}
