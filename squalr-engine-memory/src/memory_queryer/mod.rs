pub mod memory_protection_enum;
pub mod memory_queryer_trait;
pub mod memory_type_enum;
pub mod region_bounds_handling;

#[cfg(any(target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "macos"))]
mod macos;

#[cfg(target_os = "windows")]
mod windows;
