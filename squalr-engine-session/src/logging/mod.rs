#[cfg(target_os = "android")]
pub mod android_logcat_appender;
pub mod log_dispatcher;
pub mod log_history_appender;
pub mod platform;
