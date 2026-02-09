use crate::logging::log_history_appender::LogHistoryAppender;
use log::LevelFilter;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use squalr_engine_api::structures::logging::log_event::LogEvent;
use std::{
    collections::VecDeque,
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub struct LogDispatcher {
    log_history: Arc<RwLock<VecDeque<LogEvent>>>,
}

impl LogDispatcher {
    pub fn new() -> Self {
        let logger = LogDispatcher {
            log_history: Arc::new(RwLock::new(VecDeque::new())),
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
        let log_root_dir = Self::get_log_root_path();

        if !log_root_dir.exists() {
            std::fs::create_dir_all(&log_root_dir)?;
        }

        let log_file = Self::get_log_path();
        let backup_file = Self::get_log_backup_path();

        // If a log file already exists, rename it as a backup before creating a new log.
        if log_file.exists() {
            fs::rename(&log_file, &backup_file)?;
        }

        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} - {l} - {t} - {m}\n")))
            .build();

        let file_appender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} - {l} - {t} - {m}\n")))
            .build(log_file)?;

        let log_history_appender = LogHistoryAppender::new(self.log_history.clone());

        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("file", Box::new(file_appender)))
            .appender(Appender::builder().build("log_events", Box::new(log_history_appender)))
            .build(
                Root::builder()
                    .appender("stdout")
                    .appender("file")
                    .appender("log_events")
                    .build(LevelFilter::Debug),
            )?;

        log4rs::init_config(config)?;

        Ok(())
    }

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

    fn get_log_path() -> PathBuf {
        let mut log_path = Self::get_log_root_path();
        log_path.push("application.log");

        log_path
    }

    fn get_log_backup_path() -> PathBuf {
        let mut log_path = Self::get_log_root_path();
        log_path.push("application.log.bak");

        log_path
    }
}
