use std::sync::{Arc, Mutex};
use crate::imemory_queryer::IMemoryQueryer;

pub struct MemoryQueryer;

impl MemoryQueryer {
    pub fn instance() -> Arc<Mutex<dyn IMemoryQueryer>> {
        Arc::new(Mutex::new(MemoryQueryerImpl::new()))
    }
}

#[cfg(target_os = "windows")]
pub use crate::windows::windows_memory_query::WindowsMemoryQuery as MemoryQueryerImpl;

#[cfg(target_os = "linux")]
pub use crate::linux::linux_memory_query::LinuxMemoryQuery as MemoryQueryerImpl;
