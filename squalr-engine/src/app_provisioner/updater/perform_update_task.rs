use crate::app_provisioner::installer::install_phase::InstallPhase;
use crate::app_provisioner::installer::install_progress::InstallProgress;
use crate::app_provisioner::progress_tracker::ProgressTracker;
use crate::app_provisioner::updater::update_status::UpdateStatus;
use anyhow::Result;
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use std::sync::Arc;

pub struct PerformUpdateTask;

impl PerformUpdateTask {
    pub fn run<F>(notify_status: F) -> Arc<TrackableTask>
    where
        F: Fn(UpdateStatus) + Send + Sync + 'static,
    {
        let task = TrackableTask::create("PerformUpdateTask".to_string(), None);
        let task_clone = task.clone();
        let notify_status = Arc::new(notify_status);

        std::thread::spawn(move || {
            if let Err(err) = Self::execute(task_clone.clone(), notify_status.clone()) {
                log::error!("Update failed: {}", err);
                notify_status(UpdateStatus::Error(err.to_string()));
                task_clone.complete();
            }
        });

        task
    }

    fn execute(
        task: Arc<TrackableTask>,
        notify_status: Arc<dyn Fn(UpdateStatus) + Send + Sync>,
    ) -> Result<()> {
        let progress_tracker = ProgressTracker::new();
        let task_clone = task.clone();
        let progress_receiver = progress_tracker.subscribe();

        // Single thread to handle both progress updates and task completion
        std::thread::spawn(move || {
            while let Ok(progress) = progress_receiver.recv() {
                // Check cancellation.
                if task_clone
                    .get_cancellation_token()
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    notify_status(UpdateStatus::Cancelled);
                    break;
                }

                Self::map_progress_to_status(progress, &notify_status);

                // Check for completion
                if matches!(progress.phase, InstallPhase::Complete) {
                    task_clone.complete();
                    break;
                }
            }
        });

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
