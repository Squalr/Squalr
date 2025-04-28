use crate::app_provisioner::app_download_endpoints::AppDownloadEndpoints;
use crate::app_provisioner::installer::install_phase::InstallPhase;
use crate::app_provisioner::installer::install_progress::InstallProgress;
use crate::app_provisioner::operations::download::download_progress::DownloadProgress;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
pub struct ProgressTracker {
    progress: Arc<Mutex<Option<DownloadProgress>>>,
    subscriber_senders: Arc<RwLock<Vec<Sender<InstallProgress>>>>,
    current_progress: Arc<RwLock<InstallProgress>>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(Mutex::new(None)),
            subscriber_senders: Arc::new(RwLock::new(Vec::new())),
            current_progress: Arc::new(RwLock::new(InstallProgress {
                phase: InstallPhase::Download,
                progress_percent: 0.0,
                bytes_processed: 0,
                total_bytes: 0,
            })),
        }
    }

    pub fn subscribe(&self) -> Receiver<InstallProgress> {
        let (sender, receiver) = mpsc::channel();

        if let Ok(mut senders) = self.subscriber_senders.write() {
            senders.push(sender.clone());
        }

        if let Ok(current) = self.current_progress.read() {
            let _ = sender.send(*current);
        }

        receiver
    }

    pub fn update_progress(
        &self,
        progress: InstallProgress,
    ) {
        if let Ok(mut current) = self.current_progress.write() {
            *current = progress;
        }

        if let Ok(senders) = self.subscriber_senders.read() {
            for sender in senders.iter() {
                let _ = sender.send(progress);
            }
        }
    }

    pub fn init_progress(&self) {
        if let Ok(mut progress) = self.progress.lock() {
            *progress = Some(DownloadProgress {
                filename: AppDownloadEndpoints::FILENAME.to_string(),
                bytes_downloaded: 0,
                total_bytes: 0,
            });
        }
    }

    pub fn get_progress(&self) -> Arc<Mutex<Option<DownloadProgress>>> {
        self.progress.clone()
    }
}
