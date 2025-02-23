use crate::tasks::trackable_task_handle::TrackableTaskHandle;
use crossbeam_channel::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use uuid::Uuid;

pub struct TrackableTask<ResultType: Send + Sync> {
    name: String,
    progress: Arc<Mutex<f32>>,
    task_identifier: String,
    is_canceled: Arc<AtomicBool>,
    is_completed: Arc<AtomicBool>,
    result: Arc<(Mutex<Option<ResultType>>, Condvar)>,
    progress_sender: Sender<f32>,
    progress_receiver: Receiver<f32>,
}

impl<ResultType: Send + Sync + 'static> TrackableTask<ResultType> {
    pub fn create(
        name: String,
        task_identifier: Option<String>,
    ) -> Arc<Self> {
        let task_identifier = task_identifier.unwrap_or_else(|| Uuid::new_v4().to_string());
        let (progress_sender, progress_receiver) = crossbeam_channel::unbounded();

        let task = Arc::new(TrackableTask {
            name,
            progress: Arc::new(Mutex::new(0.0)),
            task_identifier,
            is_canceled: Arc::new(AtomicBool::new(false)),
            is_completed: Arc::new(AtomicBool::new(false)),
            result: Arc::new((Mutex::new(None), Condvar::new())),
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
            cancellation_token: self.get_cancellation_token().clone(),
        }
    }

    pub fn get_progress(&self) -> f32 {
        if let Ok(progress_guard) = self.progress.lock() {
            *progress_guard
        } else {
            0.0
        }
    }

    pub fn set_progress(
        &self,
        progress: f32,
    ) {
        if let Ok(mut progress_guard) = self.progress.lock() {
            *progress_guard = progress;
            let _ = self.progress_sender.send(progress);
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
    }

    pub fn complete(
        &self,
        result: ResultType,
    ) {
        self.is_completed.store(true, Ordering::SeqCst);

        if let Ok((result_lock, cvar)) = Arc::try_unwrap(self.result.clone()) {
            if let Ok(mut result_guard) = result_lock.lock() {
                *result_guard = Some(result);

                // Notify all waiting threads that the result is available.
                cvar.notify_all();
            }
        }
    }

    pub fn wait_for_completion(&self) -> Option<ResultType> {
        let (result_lock, cvar) = &*self.result;
        let mut result_guard = match result_lock.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };

        while result_guard.is_none() {
            result_guard = match cvar.wait(result_guard) {
                Ok(guard) => guard,
                Err(_) => return None,
            };
        }

        result_guard.take()
    }
}
