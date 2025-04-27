use crate::updates::updater::update_status::UpdateStatus;
use anyhow::{Context, Result};
use semver::Version;
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use std::sync::{Arc, RwLock};

pub struct CheckForUpdatesTask {}

/*
impl CheckForUpdatesTask {
    const BUCKET_NAME: &str = "wisp-134d6.appspot.com";
    const UPDATES_PREFIX: &str = "releases/windows";
    const VERSION_FILE: &str = "latest_version";

    pub fn run<F>(notify_status: F) -> Arc<RwLock<TrackableTask<Option<Version>>>>
    where
        F: Fn(UpdateStatus) + Send + Sync + 'static,
    {
        let task = TrackableTask::create("CheckForUpdatesTask".to_string());
        let task_clone = task.clone();
        let notify_status = Arc::new(notify_status);

        std::thread::spawn(move || {
            CheckForUpdatesTask::execute(task_clone, notify_status);
        });

        task
    }

    fn execute(
        task: Arc<RwLock<TrackableTask<Option<Version>>>>,
        notify_status: Arc<dyn Fn(UpdateStatus) + Send + Sync>,
    ) {
        let task_guard = task.read().unwrap();
        if task_guard
            .get_cancellation_token()
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }

        if task_guard
            .get_pause_token()
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }

        // Initial status
        notify_status(UpdateStatus::CheckingForUpdates);

        // Construct version URL
        let version_url = format!(
            "https://firebasestorage.googleapis.com/v0/b/{}/o/{}%2F{}?alt=media",
            Self::BUCKET_NAME,
            urlencoding::encode(Self::UPDATES_PREFIX),
            urlencoding::encode(Self::VERSION_FILE)
        );

        // Perform version check
        match Self::check_for_updates(&version_url, &notify_status) {
            Ok(update_result) => {
                // Update task result
                task.write().unwrap().complete(update_result);
            }
            Err(e) => {
                log::error!("Failed to check for updates: {}", e);
                notify_status(UpdateStatus::Error(e.to_string()));
                task.write().unwrap().complete(None);
            }
        }
    }

    fn check_for_updates(
        version_url: &str,
        notify_status: &Arc<dyn Fn(UpdateStatus) + Send + Sync>,
    ) -> Result<Option<Version>> {
        let response = reqwest::blocking::Client::new()
            .get(version_url)
            .send()
            .context("Failed to send version request")?
            .text()
            .context("Failed to read version response")?;

        let latest_version: Version = response.trim().parse()?;
        let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();

        if latest_version > current_version {
            notify_status(UpdateStatus::UpdateAvailable(latest_version.clone()));
            log::info!("An update is available.");
            Ok(Some(latest_version))
        } else {
            notify_status(UpdateStatus::NoUpdateRequired);
            log::info!("App is up to date, no update is required.");
            Ok(None)
        }
    }
}
*/
