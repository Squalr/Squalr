use crate::process_info::OpenedProcessInfo;
use crate::process_info::ProcessInfo;
use crate::process_query::process_query_options::ProcessQueryOptions;

pub(crate) trait ProcessQueryer {
    fn start_monitoring() -> Result<(), String>;
    fn stop_monitoring() -> Result<(), String>;
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String>;
    fn close_process(handle: u64) -> Result<(), String>;
    fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo>;
}

#[cfg(any(target_os = "android"))]
use crate::process_query::android::android_process_query::AndroidProcessQuery as ProcessQueryImpl;

#[cfg(any(target_os = "linux"))]
use crate::process_query::linux::linux_process_query::LinuxProcessQuery as ProcessQueryImpl;

#[cfg(any(target_os = "macos"))]
use crate::process_query::macos::macos_process_query::MacOsProcessQuery as ProcessQueryImpl;

#[cfg(target_os = "windows")]
use crate::process_query::windows::windows_process_query::WindowsProcessQuery as ProcessQueryImpl;

pub struct ProcessQuery;

impl ProcessQuery {
    pub fn start_monitoring() -> Result<(), String> {
        ProcessQueryImpl::start_monitoring()
    }

    pub fn stop_monitoring() -> Result<(), String> {
        ProcessQueryImpl::stop_monitoring()
    }

    pub fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        ProcessQueryImpl::open_process(process_info)
    }

    pub fn close_process(handle: u64) -> Result<(), String> {
        ProcessQueryImpl::close_process(handle)
    }

    pub fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        ProcessQueryImpl::get_processes(options)
    }
}
