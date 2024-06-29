pub mod emulator_type;
pub mod imemory_queryer;
pub mod memory_protection_enum;
pub mod memory_queryer;
pub mod memory_type_enum;
pub mod normalized_flags;
pub mod normalized_module;
pub mod normalized_region;
pub mod region_bounds_handling;

#[cfg(any(target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "macos"))]
mod macos;

#[cfg(target_os = "windows")]
mod windows;