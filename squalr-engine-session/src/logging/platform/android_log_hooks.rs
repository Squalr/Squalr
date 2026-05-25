#[cfg(target_os = "android")]
use super::super::log_dispatcher::{LogDispatcher, LogDispatcherOptions};
#[cfg(target_os = "android")]
use super::platform_log_hooks::PlatformLogHooks;
#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
static ANDROID_LOG_DISPATCHER_INIT: OnceLock<LogDispatcher> = OnceLock::new();

#[cfg(target_os = "android")]
pub struct AndroidLogHooks;

#[cfg(target_os = "android")]
impl PlatformLogHooks for AndroidLogHooks {
    fn initialize_platform_log_hooks_once(
        &self,
        _log_tag: &str,
    ) {
        ANDROID_LOG_DISPATCHER_INIT.get_or_init(|| LogDispatcher::new_with_options(LogDispatcherOptions { enable_console_output: false }));
    }
}
