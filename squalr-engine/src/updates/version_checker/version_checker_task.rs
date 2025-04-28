use crate::updates::app_download_endpoints::AppDownloadEndpoints;
use crate::updates::version_checker::github_latest_version_info::GitHubLatestVersionInfo;
use crate::updates::version_checker::version_checker_status::VersionCheckerStatus;
use anyhow::{Context, Result};
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use std::sync::Arc;
use ureq::{
    config::Config,
    tls::{TlsConfig, TlsProvider},
};

pub struct VersionCheckerTask {}

impl VersionCheckerTask {
    pub fn run<F>(notify_status: F) -> Arc<TrackableTask>
    where
        F: Fn(VersionCheckerStatus) + Send + Sync + 'static,
    {
        let task = TrackableTask::create("Version Checker".to_string(), None);
        let task_clone = task.clone();
        let notify_status = Arc::new(notify_status);

        std::thread::spawn(move || {
            VersionCheckerTask::execute(task_clone, notify_status);
        });

        task
    }

    fn execute(
        task: Arc<TrackableTask>,
        notify_status: Arc<dyn Fn(VersionCheckerStatus) + Send + Sync>,
    ) {
        if task
            .get_cancellation_token()
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }

        // Initial status
        notify_status(VersionCheckerStatus::CheckingForVersions);

        // Perform version check
        match Self::check_for_updates() {
            Ok(update_result) => {
                match update_result {
                    Some(latest_version) => {
                        notify_status(VersionCheckerStatus::LatestVersionFound(latest_version.clone()));
                        log::info!("Latest version information found.");
                    }
                    None => {
                        notify_status(VersionCheckerStatus::Error("Failed to get latest app version.".to_string()));
                    }
                }
                task.complete();
            }
            Err(err) => {
                log::error!("Failed to check for updates: {}", err);
                notify_status(VersionCheckerStatus::Error(err.to_string()));
                task.complete();
            }
        }
    }

    fn check_for_updates() -> Result<Option<GitHubLatestVersionInfo>> {
        let tls_config = TlsConfig::builder().provider(TlsProvider::NativeTls).build();
        let config = Config::builder().tls_config(tls_config).build();
        let agent = config.new_agent();
        let response = agent
            .get(AppDownloadEndpoints::get_latest_version_url())
            .header("User-Agent", "squalr-rust-updater")
            .call()
            .context("Failed to send GitHub latest release request")?;
        let body = response
            .into_body()
            .read_to_string()
            .context("Failed to read GitHub release response body")?;
        let release: GitHubLatestVersionInfo = serde_json::from_str(&body).context("Failed to parse GitHub release JSON")?;

        Ok(Some(release))
    }
}
