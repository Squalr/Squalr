#[cfg(windows)]
mod windows_backend;

#[cfg(windows)]
pub(crate) use windows_backend::DbgEngBackend as NativeDebuggerBackend;

#[cfg(target_os = "macos")]
mod macos_backend;

#[cfg(target_os = "macos")]
pub(crate) use macos_backend::MacOsDebuggerBackend as NativeDebuggerBackend;

#[cfg(any(all(target_os = "linux", target_arch = "x86_64"), all(target_os = "android", target_arch = "aarch64")))]
mod linux_backend;

#[cfg(any(all(target_os = "linux", target_arch = "x86_64"), all(target_os = "android", target_arch = "aarch64")))]
pub(crate) use linux_backend::LinuxDebuggerBackend as NativeDebuggerBackend;

#[cfg(not(any(
    windows,
    target_os = "macos",
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "android", target_arch = "aarch64")
)))]
mod non_windows_backend;

#[cfg(not(any(
    windows,
    target_os = "macos",
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "android", target_arch = "aarch64")
)))]
pub(crate) use non_windows_backend::NativeDebuggerBackend;
