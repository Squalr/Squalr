use crate::logging::output_log_collector::OutputLogCollector;
use crossbeam_channel::{Receiver, Sender};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use std::fs;
use std::path::PathBuf;

pub struct FileSystemLogger {
    log_receiver: Receiver<String>,
}

impl FileSystemLogger {
    pub fn new() -> Self {
        let (log_sender, log_receiver) = crossbeam_channel::unbounded();

        let file_system_logger = FileSystemLogger { log_receiver };

        if let Err(err) = file_system_logger.initialize(log_sender) {
            log::error!("Failed to initialize file system logging: {err}");
        }

        file_system_logger
    }

    pub fn subscribe_to_logs(&self) -> Receiver<String> {
        self.log_receiver.clone()
    }

    fn initialize(
        &self,
        log_sender: Sender<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        let file_appender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} - {l} - {t} - {m}\n")))
            .build(log_file)?;

        let output_log_collector = OutputLogCollector::new(log_sender);

        let config = Config::builder()
            .appender(Appender::builder().build("file", Box::new(file_appender)))
            .appender(Appender::builder().build("output", Box::new(output_log_collector)))
            .build(
                Root::builder()
                    .appender("file")
                    .appender("output")
                    .build(LevelFilter::Debug),
            )?;

        log4rs::init_config(config)?;

        return Ok(());
    }

    fn get_log_root_path() -> PathBuf {
        match dirs::data_local_dir() {
            Some(mut path) => {
                path.push("Squalr");
                path.push("logs");
                std::fs::create_dir_all(&path).unwrap_or_else(|err| {
                    log::error!("Failed to create logs directory: {err}");
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

        return log_path;
    }

    fn get_log_backup_path() -> PathBuf {
        let mut log_path = Self::get_log_root_path();
        log_path.push("application.log.bak");

        return log_path;
    }
}
