pub mod process_query_options;
pub mod process_queryer;

#[cfg(any(target_os = "android"))]
mod android;

#[cfg(any(target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "macos"))]
mod macos;

#[cfg(target_os = "windows")]
mod windows;
