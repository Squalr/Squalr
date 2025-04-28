use crate::updates::downloader::download_progress::DownloadProgress;
use anyhow::Result;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use ureq::config::Config;
use ureq::tls::{TlsConfig, TlsProvider};

pub struct FileDownloader {
    progress: Arc<Mutex<Option<DownloadProgress>>>,
    progress_callback: Box<dyn Fn(u64, u64) + Send + Sync>,
}

impl FileDownloader {
    pub fn new(
        progress: Arc<Mutex<Option<DownloadProgress>>>,
        progress_callback: Box<dyn Fn(u64, u64) + Send + Sync>,
    ) -> Self {
        Self { progress, progress_callback }
    }

    fn update_progress(
        &self,
        bytes_downloaded: u64,
        total_bytes: Option<u64>,
    ) {
        if let Ok(mut progress) = self.progress.lock() {
            if let Some(ref mut progress) = *progress {
                progress.bytes_downloaded = bytes_downloaded;
                if let Some(total) = total_bytes {
                    progress.total_bytes = total;
                    (self.progress_callback)(bytes_downloaded, total);
                }
            }
        }
    }

    pub fn download_file(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<()> {
        log::info!("Downloading from: {}", url);

        let tls_config = TlsConfig::builder().provider(TlsProvider::NativeTls).build();
        let config = Config::builder().tls_config(tls_config).build();
        let agent = config.new_agent();
        let mut response = agent.get(url).call()?;

        // Status is ureq::http::StatusCode. Convert it to u16 first.
        let status = response.status();
        if status.as_u16() < 200 || status.as_u16() >= 300 {
            anyhow::bail!("Download failed: HTTP {}", status);
        }

        log::info!("Download response status: {}", status);

        // Get Content-Length manually.
        let total_size = response
            .headers()
            .get("Content-Length")
            .and_then(|v| v.to_str().ok()?.parse::<u64>().ok())
            .unwrap_or(0);

        log::info!("Expected download size: {} bytes", total_size);

        self.update_progress(0, Some(total_size));

        let mut file = File::create(target_path)?;
        let mut bytes_downloaded = 0u64;

        let mut reader = response.body_mut().as_reader();
        let mut buffer = [0u8; 8192];

        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])?;
            bytes_downloaded += read as u64;
            self.update_progress(bytes_downloaded, None);
        }

        log::info!("Download complete. Total bytes: {}", bytes_downloaded);
        file.sync_all()?;
        log::info!("File synced to disk");

        Ok(())
    }
}
