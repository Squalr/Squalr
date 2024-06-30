use std::sync::{Arc, Mutex};
use crate::memory_queryer_trait::IMemoryQueryer;

pub struct MemoryQueryer;

impl MemoryQueryer {
    pub fn instance() -> Arc<Mutex<dyn IMemoryQueryer>> {
        Arc::new(Mutex::new(MemoryQueryerImpl::new()))
    }
}

#[cfg(target_os = "linux")]
pub use crate::linux::linux_memory_queryer::LinuxMemoryQueryer as MemoryQueryerImpl;

#[cfg(target_os = "macos")]
pub use crate::macos::macos_memory_queryer::MacOsMemoryQueryer as MemoryQueryerImpl;

#[cfg(target_os = "windows")]
pub use crate::windows::windows_memory_queryer::WindowsMemoryQueryer as MemoryQueryerImpl;
