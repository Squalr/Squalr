use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use std::fs;
use std::path::PathBuf;

pub struct FileSystemLogger {}

impl FileSystemLogger {
    pub fn new() -> Self {
        FileSystemLogger {}
    }

    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
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

        let config = Config::builder()
            .appender(Appender::builder().build("file", Box::new(file_appender)))
            .build(Root::builder().appender("file").build(LevelFilter::Debug))?;

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
