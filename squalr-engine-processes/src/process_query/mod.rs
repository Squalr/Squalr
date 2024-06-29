use sysinfo::Pid;

pub trait IProcessQueryer {
    fn get_processes(&mut self) -> Vec<Pid>;
    fn is_process_system_process(&self, pid: &Pid) -> bool;
    fn is_process_windowed(&self, pid: &Pid) -> bool;
    fn get_icon(&self, pid: &Pid) -> Option<Vec<u8>>;
    fn get_process_name(&self, pid: Pid) -> Option<String>;
}

#[cfg(any(target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "macos"))]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(any(target_os = "linux"))]
pub use self::linux::linux_process_query::LinuxProcessQuery as ProcessQueryImpl;

#[cfg(any(target_os = "macos"))]
pub use self::macos::macos_process_query::MacOsProcessQuery as ProcessQueryImpl;

#[cfg(target_os = "windows")]
pub use self::windows::windows_process_query::WindowsProcessQuery as ProcessQueryImpl;

pub struct ProcessQuery;

impl ProcessQuery {
    pub fn instance() -> Box<dyn IProcessQueryer> {
        Box::new(ProcessQueryImpl::new())
    }
}
