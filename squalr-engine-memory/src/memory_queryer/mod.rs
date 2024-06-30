pub mod memory_protection_enum;
pub mod memory_queryer_trait;
pub mod memory_type_enum;
pub mod region_bounds_handling;
use std::sync::Once;

#[cfg(any(target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "macos"))]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use crate::memory_queryer::linux::linux_memory_queryer::LinuxMemoryQueryer as MemoryQueryerImpl;

#[cfg(target_os = "macos")]
pub use crate::memory_queryer::macos::macos_memory_queryer::MacOsMemoryQueryer as MemoryQueryerImpl;

#[cfg(target_os = "windows")]
pub use crate::memory_queryer::windows::windows_memory_queryer::WindowsMemoryQueryer as MemoryQueryerImpl;

pub struct MemoryQueryer;

impl MemoryQueryer {
    pub fn instance() -> &'static MemoryQueryerImpl {
        static mut SINGLETON: Option<MemoryQueryerImpl> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = MemoryQueryerImpl::new();
                SINGLETON = Some(instance);
            });

            SINGLETON.as_ref().unwrap()
        }
    }
}
