pub mod memory_reader_trait;

use std::sync::Once;

#[cfg(any(target_os = "android"))]
mod android;

#[cfg(any(target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "macos"))]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "android")]
pub use crate::memory_reader::android::android_memory_reader::AndroidMemoryReader as MemoryReaderImpl;

#[cfg(target_os = "linux")]
pub use crate::memory_reader::linux::linux_memory_reader::LinuxMemoryReader as MemoryReaderImpl;

#[cfg(target_os = "macos")]
pub use crate::memory_reader::macos::macos_memory_reader::MacOsMemoryReader as MemoryReaderImpl;

#[cfg(target_os = "windows")]
// pub use crate::memory_reader::windows::windows_memory_reader::WindowsMemoryReader as MemoryReaderImpl;
pub use crate::memory_reader::windows::windows_memory_reader_nt::WindowsMemoryReaderNt as MemoryReaderImpl;

pub struct MemoryReader;

impl MemoryReader {
    pub fn get_instance() -> &'static MemoryReaderImpl {
        static mut INSTANCE: Option<MemoryReaderImpl> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = MemoryReaderImpl::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }
}
