#[cfg(target_os = "android")]
use super::android_log_hooks::AndroidLogHooks;

pub trait PlatformLogHooks: Send + Sync {
    fn initialize_platform_log_hooks_once(
        &self,
        log_tag: &str,
    );
}

struct NoOpPlatformLogHooks;

impl PlatformLogHooks for NoOpPlatformLogHooks {
    fn initialize_platform_log_hooks_once(
        &self,
        _log_tag: &str,
    ) {
    }
}

#[cfg(target_os = "android")]
static PLATFORM_LOG_HOOKS: AndroidLogHooks = AndroidLogHooks;

#[cfg(not(target_os = "android"))]
static PLATFORM_LOG_HOOKS: NoOpPlatformLogHooks = NoOpPlatformLogHooks;

pub fn get_platform_log_hooks() -> &'static dyn PlatformLogHooks {
    &PLATFORM_LOG_HOOKS
}

pub fn initialize_platform_log_hooks_once(log_tag: &str) {
    get_platform_log_hooks().initialize_platform_log_hooks_once(log_tag);
}
