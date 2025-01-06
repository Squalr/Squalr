use crate::process_info::OpenedProcessInfo;
use crate::process_info::ProcessIcon;
use crate::process_info::ProcessInfo;
use sysinfo::Pid;

pub trait ProcessQueryer {
    fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String>;
    fn close_process(handle: u64) -> Result<(), String>;
    fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo>;
    fn is_process_windowed(pid: &Pid) -> bool;
    fn get_icon(pid: &Pid) -> Option<ProcessIcon>;
}

pub struct ProcessQueryOptions {
    pub require_windowed: bool,
    pub required_pid: Option<Pid>,
    pub search_name: Option<String>,
    pub match_case: bool,
    pub limit: Option<u64>,
}

#[cfg(any(target_os = "linux"))]
pub use crate::process_query::linux::linux_process_query::LinuxProcessQuery as ProcessQueryImpl;

#[cfg(any(target_os = "macos"))]
pub use crate::process_query::macos::macos_process_query::MacOsProcessQuery as ProcessQueryImpl;

#[cfg(target_os = "windows")]
pub use crate::process_query::windows::windows_process_query::WindowsProcessQuery as ProcessQueryImpl;

pub struct ProcessQuery;

impl ProcessQuery {
    pub fn open_process(process_info: &ProcessInfo) -> Result<OpenedProcessInfo, String> {
        ProcessQueryImpl::open_process(process_info)
    }

    pub fn close_process(handle: u64) -> Result<(), String> {
        ProcessQueryImpl::close_process(handle)
    }

    pub fn get_processes(options: ProcessQueryOptions) -> Vec<ProcessInfo> {
        ProcessQueryImpl::get_processes(options)
    }

    pub fn is_process_windowed(pid: &Pid) -> bool {
        ProcessQueryImpl::is_process_windowed(pid)
    }

    pub fn get_icon(pid: &Pid) -> Option<ProcessIcon> {
        ProcessQueryImpl::get_icon(pid)
    }
}
