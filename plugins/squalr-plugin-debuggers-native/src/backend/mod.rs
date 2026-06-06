#[cfg(windows)]
mod windows_backend;

#[cfg(windows)]
pub(crate) use windows_backend::DbgEngBackend as NativeDebuggerBackend;

#[cfg(target_os = "macos")]
mod macos_backend;

#[cfg(target_os = "macos")]
pub(crate) use macos_backend::MacOsDebuggerBackend as NativeDebuggerBackend;

#[cfg(not(any(windows, target_os = "macos")))]
mod non_windows_backend;

#[cfg(not(any(windows, target_os = "macos")))]
pub(crate) use non_windows_backend::NativeDebuggerBackend;
