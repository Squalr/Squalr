use sysinfo::Pid;

pub trait IProcessQueryer {
    fn get_processes(&mut self) -> Vec<Pid>;
    fn is_process_system_process(&self, pid: &Pid) -> bool;
    fn is_process_windowed(&self, pid: &Pid) -> bool;
    fn get_icon(&self, pid: &Pid) -> Option<Vec<u8>>;
    fn get_process_name(&self, pid: Pid) -> Option<String>;
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::WindowsProcessQuery as ProcessQueryImpl;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "unix"))]
mod unix;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "unix"))]
pub use self::unix::UnixProcessQuery as ProcessQueryImpl;

pub struct ProcessQuery;

impl ProcessQuery {
    pub fn instance() -> Box<dyn IProcessQueryer> {
        Box::new(ProcessQueryImpl::new())
    }
}
