use crate::process_info::ProcessInfo;

use sysinfo::Pid;

pub trait IProcessQueryer {
    fn get_processes(
        &mut self,
        options: ProcessQueryOptions,
    ) -> Vec<Pid>;
    fn is_process_windowed(
        &self,
        pid: &Pid,
    ) -> bool;
    fn get_icon(
        &self,
        pid: &Pid,
    ) -> Option<Vec<u8>>;
    fn get_process_name(
        &self,
        pid: Pid,
    ) -> Option<String>;
    fn open_process(
        &self,
        process_id: &Pid
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
mod linux;

#[cfg(any(target_os = "linux"))]
pub use self::linux::linux_process_query::LinuxProcessQuery as ProcessQueryImpl;

#[cfg(any(target_os = "macos"))]
mod macos;
#[cfg(any(target_os = "macos"))]
pub use self::macos::macos_process_query::MacOsProcessQuery as ProcessQueryImpl;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::windows_process_query::WindowsProcessQuery as ProcessQueryImpl;

pub struct ProcessQuery;

impl ProcessQuery {
    pub fn get_instance() -> Box<dyn IProcessQueryer> {
        return Box::new(ProcessQueryImpl::new());
    }
}
