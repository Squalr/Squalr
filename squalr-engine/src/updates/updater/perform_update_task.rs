use crate::updates::shared::install_phase::InstallPhase;
use crate::updates::shared::install_progress::InstallProgress;
use crate::updates::updater::app_updater::AppUpdater;
use crate::updates::updater::update_status::UpdateStatus;
use anyhow::Result;
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use std::sync::{Arc, RwLock};

pub struct PerformUpdateTask;

/*
impl PerformUpdateTask {
    pub fn run<F>(notify_status: F) -> Arc<RwLock<TrackableTask<()>>>
    where
        F: Fn(UpdateStatus) + Send + Sync + 'static,
    {
        let task = TrackableTask::create("PerformUpdateTask".to_string());
        let task_clone = task.clone();
        let notify_status = Arc::new(notify_status);

        std::thread::spawn(move || {
            if let Err(e) = Self::execute(task_clone.clone(), notify_status.clone()) {
                log::error!("Update failed: {}", e);
                notify_status(UpdateStatus::Error(e.to_string()));
                task_clone.write().unwrap().complete(());
            }
        });

        task
    }

    fn execute(
        task: Arc<RwLock<TrackableTask<()>>>,
        notify_status: Arc<dyn Fn(UpdateStatus) + Send + Sync>,
    ) -> Result<()> {
        let installer = AppUpdater::get_instance();
        let installer_guard = installer.read().unwrap();

        // Subscribe to installer progress updates
        let progress_receiver = installer_guard.subscribe();
        let task_clone = task.clone();

        // Single thread to handle both progress updates and task completion
        std::thread::spawn(move || {
            while let Ok(progress) = progress_receiver.recv() {
                // Check cancellation
                if let Ok(task_guard) = task_clone.read() {
                    if task_guard
                        .get_cancellation_token()
                        .load(std::sync::atomic::Ordering::Relaxed)
                    {
                        notify_status(UpdateStatus::Cancelled);
                        break;
                    }
                }

                Self::map_progress_to_status(progress, &notify_status);

                // Check for completion
                if matches!(progress.phase, InstallPhase::Complete) {
                    task_clone.write().unwrap().complete(());
                    break;
                }
            }
        });

        installer_guard.begin_update();
        Ok(())
    }

    fn map_progress_to_status(
        progress: InstallProgress,
        notify_status: &Arc<dyn Fn(UpdateStatus) + Send + Sync>,
    ) {
        let status = match progress.phase {
            InstallPhase::Download => UpdateStatus::Downloading(progress.progress_percent),
            InstallPhase::Extraction => UpdateStatus::Installing(progress.progress_percent),
            InstallPhase::Complete => UpdateStatus::Complete,
        };

        notify_status(status);
    }
}
*/
