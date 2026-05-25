#[cfg(target_os = "android")]
use crate::logging::android_logcat_appender::AndroidLogcatAppender;
use crate::logging::log_history_appender::LogHistoryAppender;
use crate::logging::remote_log_event_appender::RemoteLogEventAppender;
use log::LevelFilter;
use log4rs::config::{Appender, Config, Logger, Root};
#[cfg(not(target_os = "android"))]
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    encode::pattern::PatternEncoder,
};
use squalr_engine_api::structures::logging::log_event::LogEvent;
#[cfg(not(target_os = "android"))]
use std::fs;
#[cfg(not(target_os = "android"))]
use std::path::PathBuf;
use std::{
    collections::VecDeque,
    sync::{Arc, LazyLock, Mutex, OnceLock, RwLock},
};

pub struct LogDispatcher {
    log_history: Arc<RwLock<VecDeque<LogEvent>>>,
    #[cfg_attr(target_os = "android", allow(dead_code))]
    options: LogDispatcherOptions,
}

static SHARED_LOG_HISTORY: LazyLock<Arc<RwLock<VecDeque<LogEvent>>>> = LazyLock::new(|| Arc::new(RwLock::new(VecDeque::new())));
static LOGGER_HANDLE: OnceLock<log4rs::Handle> = OnceLock::new();
static LOGGER_INIT_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Clone, Copy)]
pub struct LogDispatcherOptions {
    pub enable_console_output: bool,
}

impl Default for LogDispatcherOptions {
    fn default() -> Self {
        Self { enable_console_output: true }
    }
}

impl LogDispatcher {
    pub fn new() -> Self {
        Self::new_with_options(LogDispatcherOptions::default())
    }

    pub fn new_with_options(options: LogDispatcherOptions) -> Self {
        let logger = LogDispatcher {
            log_history: SHARED_LOG_HISTORY.clone(),
            options,
        };

        if let Err(error) = logger.initialize() {
            log::error!("Failed to initialize logging: {}", error);
        }

        logger
    }

    pub fn get_log_history(&self) -> &Arc<RwLock<VecDeque<LogEvent>>> {
        &self.log_history
    }

    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _logger_init_guard = match LOGGER_INIT_LOCK.lock() {
            Ok(lock_guard) => lock_guard,
            Err(error) => {
                return Err(format!("Failed to acquire logger initialization lock: {}", error).into());
            }
        };

        if let Some(existing_logger_handle) = LOGGER_HANDLE.get() {
            let config = self.build_config(false)?;
            existing_logger_handle.set_config(config);
            return Ok(());
        }

        let config = self.build_config(true)?;
        let logger_handle = log4rs::init_config(config)?;

        if LOGGER_HANDLE.set(logger_handle).is_err() {
            return Err("Logger was initialized unexpectedly while setting logger handle.".into());
        }

        Ok(())
    }

    fn build_config(
        &self,
        _should_rotate_log_file: bool,
    ) -> Result<Config, Box<dyn std::error::Error>> {
        let log_history_appender = LogHistoryAppender::new(self.log_history.clone());
        let remote_log_event_appender = RemoteLogEventAppender::new();

        let mut config_builder = Config::builder()
            .appender(Appender::builder().build("log_events", Box::new(log_history_appender)))
            .appender(Appender::builder().build("remote_log_events", Box::new(remote_log_event_appender)));
        let mut root_builder = Root::builder()
            .appender("log_events")
            .appender("remote_log_events");

        #[cfg(not(target_os = "android"))]
        {
            let log_root_dir = Self::get_log_root_path();

            if !log_root_dir.exists() {
                std::fs::create_dir_all(&log_root_dir)?;
            }

            let log_file = Self::get_log_path();
            let backup_file = Self::get_log_backup_path();

            if _should_rotate_log_file && log_file.exists() {
                fs::rename(&log_file, &backup_file)?;
            }

            let file_appender = FileAppender::builder()
                .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} - {l} - {t} - {m}\n")))
                .build(log_file)?;

            config_builder = config_builder.appender(Appender::builder().build("file", Box::new(file_appender)));
            root_builder = root_builder.appender("file");

            if self.options.enable_console_output {
                let stdout_appender = ConsoleAppender::builder()
                    .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} - {l} - {t} - {m}\n")))
                    .build();

                config_builder = config_builder.appender(Appender::builder().build("stdout", Box::new(stdout_appender)));
                root_builder = root_builder.appender("stdout");
            }
        }

        #[cfg(target_os = "android")]
        {
            let logcat_appender = AndroidLogcatAppender::new("Squalr");
            config_builder = config_builder.appender(Appender::builder().build("logcat", Box::new(logcat_appender)));
            root_builder = root_builder.appender("logcat");
        }

        config_builder
            .logger(Logger::builder().build("eframe", LevelFilter::Info))
            .logger(Logger::builder().build("glutin", LevelFilter::Info))
            .logger(Logger::builder().build("android_activity", LevelFilter::Warn))
            .logger(Logger::builder().build("android_activity::game_activity", LevelFilter::Warn))
            .logger(Logger::builder().build("sctk", LevelFilter::Info))
            .logger(Logger::builder().build("tracing::span", LevelFilter::Info))
            .logger(Logger::builder().build("winit", LevelFilter::Info))
            .logger(Logger::builder().build("winit::platform_impl::linux::wayland::window::state", LevelFilter::Error))
            .build(root_builder.build(LevelFilter::Debug))
            .map_err(Into::into)
    }

    #[cfg(not(target_os = "android"))]
    fn get_log_root_path() -> PathBuf {
        match dirs::data_local_dir() {
            Some(mut path) => {
                path.push("Squalr");
                path.push("logs");
                std::fs::create_dir_all(&path).unwrap_or_else(|error| {
                    log::error!("Failed to create logs directory: {}", error);
                });
                path
            }
            None => {
                log::error!("Failed to get local app data directory");
                PathBuf::from("logs")
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    fn get_log_path() -> PathBuf {
        let mut log_path = Self::get_log_root_path();
        log_path.push("application.log");

        log_path
    }

    #[cfg(not(target_os = "android"))]
    fn get_log_backup_path() -> PathBuf {
        let mut log_path = Self::get_log_root_path();
        log_path.push("application.log.bak");

        log_path
    }
}

#[cfg(test)]
mod tests {
    use super::{LogDispatcher, LogDispatcherOptions};
    use std::sync::Arc;

    #[test]
    fn repeated_initialization_uses_shared_log_history() {
        let first_dispatcher = LogDispatcher::new_with_options(LogDispatcherOptions { enable_console_output: false });
        let second_dispatcher = LogDispatcher::new_with_options(LogDispatcherOptions { enable_console_output: false });

        assert!(Arc::ptr_eq(first_dispatcher.get_log_history(), second_dispatcher.get_log_history()));
    }
}
