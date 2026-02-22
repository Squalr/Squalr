#[cfg(target_os = "android")]
use super::platform_log_hooks::PlatformLogHooks;
#[cfg(target_os = "android")]
use log::LevelFilter;
#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
static ANDROID_LOGCAT_INIT: OnceLock<()> = OnceLock::new();

#[cfg(target_os = "android")]
pub struct AndroidLogHooks;

#[cfg(target_os = "android")]
impl PlatformLogHooks for AndroidLogHooks {
    fn initialize_platform_log_hooks_once(
        &self,
        log_tag: &str,
    ) {
        ANDROID_LOGCAT_INIT.get_or_init(|| {
            android_logger::init_once(
                android_logger::Config::default()
                    .with_max_level(LevelFilter::Info)
                    .with_tag(log_tag),
            );
        });
    }
}
