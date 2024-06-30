pub mod memory_reader_trait;
use std::sync::{Arc, Mutex};
use crate::memory_reader::memory_reader_trait::IMemoryReader;

#[cfg(any(target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "macos"))]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use crate::memory_reader::linux::linux_memory_reader::LinuxMemoryReader as MemoryReaderImpl;

#[cfg(target_os = "macos")]
pub use crate::memory_reader::macos::macos_memory_reader::MacOsMemoryReader as MemoryReaderImpl;

#[cfg(target_os = "windows")]
pub use crate::memory_reader::windows::windows_memory_reader::WindowsMemoryReader as MemoryReaderImpl;


pub struct MemoryReader;

impl MemoryReader {
    pub fn instance() -> Arc<Mutex<dyn IMemoryReader>> {
        Arc::new(Mutex::new(MemoryReaderImpl::new()))
    }
}
