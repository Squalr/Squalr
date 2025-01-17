pub mod memory_protection_enum;
pub mod memory_queryer;
pub mod memory_queryer_trait;
pub mod memory_type_enum;
pub mod region_bounds_handling;

#[cfg(any(target_os = "android"))]
mod android;

#[cfg(any(target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "macos"))]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "android")]
pub use crate::memory_queryer::android::android_memory_queryer::AndroidMemoryQueryer as MemoryQueryerImpl;

#[cfg(target_os = "linux")]
pub use crate::memory_queryer::linux::linux_memory_queryer::LinuxMemoryQueryer as MemoryQueryerImpl;

#[cfg(target_os = "macos")]
pub use crate::memory_queryer::macos::macos_memory_queryer::MacOsMemoryQueryer as MemoryQueryerImpl;

#[cfg(target_os = "windows")]
pub use crate::memory_queryer::windows::windows_memory_queryer::WindowsMemoryQueryer as MemoryQueryerImpl;
