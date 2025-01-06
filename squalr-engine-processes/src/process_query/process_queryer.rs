use crate::process_info::ProcessInfo;
use image::DynamicImage;
use sysinfo::Pid;

pub trait ProcessQueryer {
    fn get_processes(
        &mut self,
        options: ProcessQueryOptions,
    ) -> Vec<ProcessInfo>;
    fn is_process_windowed(
        &self,
        pid: &Pid,
    ) -> bool;
    fn get_icon(
        &self,
        pid: &Pid,
    ) -> Option<DynamicImage>;
    fn get_process_name(
        &self,
        pid: Pid,
    ) -> Option<String>;
    fn open_process(
        &self,
        process_id: &Pid,
    ) -> Result<ProcessInfo, String>;
    fn close_process(
        &self,
        handle: u64,
    ) -> Result<(), String>;
}

pub struct ProcessQueryOptions {
    pub require_windowed: bool,
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
    pub fn get_instance() -> Box<dyn crate::process_query::process_queryer::ProcessQueryer> {
        return Box::new(ProcessQueryImpl::new());
    }
}
