use crate::structures::tasks::trackable_task_handle::TrackableTaskHandle;
use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use uuid::Uuid;

pub struct TrackableTask {
    name: String,
    progress: Arc<Mutex<f32>>,
    task_identifier: String,
    is_canceled: Arc<AtomicBool>,
    is_completed: Arc<AtomicBool>,
    completed_cv: Condvar,
    progress_sender: Sender<f32>,
    progress_receiver: Receiver<f32>,
}

impl TrackableTask {
    pub fn create(
        name: String,
        task_identifier: Option<String>,
    ) -> Arc<Self> {
        let task_identifier = task_identifier.unwrap_or(Uuid::new_v4().to_string());
        let (progress_sender, progress_receiver) = crossbeam_channel::unbounded();

        let task = Arc::new(TrackableTask {
            name,
            progress: Arc::new(Mutex::new(0.0)),
            task_identifier,
            is_canceled: Arc::new(AtomicBool::new(false)),
            is_completed: Arc::new(AtomicBool::new(false)),
            completed_cv: Condvar::new(),
            progress_sender,
            progress_receiver,
        });

        task
    }

    pub fn get_task_handle(&self) -> TrackableTaskHandle {
        TrackableTaskHandle {
            name: self.get_name().clone(),
            progress: self.get_progress(),
            task_identifier: self.get_task_identifier(),
        }
    }

    pub fn get_progress(&self) -> f32 {
        if let Ok(progress_guard) = self.progress.lock() {
            *progress_guard
        } else {
            log::error!("Failed to get task progress.");
            0.0
        }
    }

    pub fn set_progress(
        &self,
        progress: f32,
    ) {
        if let Ok(mut progress_guard) = self.progress.lock() {
            *progress_guard = progress;

            if let Err(error) = self.progress_sender.send(progress) {
                log::error!("Failed set task progress: {}", error);
            }
        } else {
            log::error!("Failed to get lock to set task progress.");
        }
    }

    pub fn subscribe_to_progress_updates(&self) -> Receiver<f32> {
        self.progress_receiver.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(
        &mut self,
        name: String,
    ) {
        self.name = name;
    }

    pub fn get_task_identifier(&self) -> String {
        self.task_identifier.clone()
    }

    pub fn get_cancellation_token(&self) -> Arc<AtomicBool> {
        self.is_canceled.clone()
    }

    pub fn is_completed(&self) -> bool {
        self.is_completed.load(Ordering::SeqCst)
    }

    pub fn cancel(&self) {
        self.is_canceled.store(true, Ordering::SeqCst);
        self.complete();
    }

    pub fn complete(&self) {
        self.is_completed.store(true, Ordering::SeqCst);
        self.completed_cv.notify_all();
    }

    pub fn wait_for_completion(&self) {
        let mutex = Mutex::new(());
        let mut lock = match mutex.lock() {
            Ok(lock) => lock,
            Err(error) => {
                log::error!("Error waiting for event completion: {}", error);
                return;
            }
        };

        while !self.is_completed() {
            lock = match self.completed_cv.wait(lock) {
                Ok(updated_lock) => updated_lock,
                Err(error) => {
                    log::error!("Error waiting for event completion: {}", error);
                    return;
                }
            };
        }
    }
}
