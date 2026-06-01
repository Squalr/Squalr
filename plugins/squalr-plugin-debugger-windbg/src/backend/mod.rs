#[cfg(windows)]
mod windows_backend;

#[cfg(windows)]
pub(crate) use windows_backend::WindbgBackend;

#[cfg(not(windows))]
mod non_windows_backend;

#[cfg(not(windows))]
pub(crate) use non_windows_backend::WindbgBackend;
