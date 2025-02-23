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

        let task = TrackableTask {
            name,
            progress: Arc::new(Mutex::new(0.0)),
            task_identifier,
            is_canceled: Arc::new(AtomicBool::new(false)),
            is_completed: Arc::new(AtomicBool::new(false)),
            result: Arc::new((Mutex::new(None), Condvar::new())),
            progress_sender,
            progress_receiver,
        };

        Arc::new(task)
    }

    pub fn get_progress(self: &Arc<Self>) -> f32 {
        if let Ok(progress_guard) = self.progress.lock() {
            *progress_guard
        } else {
            0.0
        }
    }

    pub fn set_progress(
        self: &Arc<Self>,
        progress: f32,
    ) {
        if let Ok(mut progress_guard) = self.progress.lock() {
            *progress_guard = progress;
            let _ = self.progress_sender.send(progress);
        }
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

    pub fn set_canceled(
        self: &Arc<Self>,
        value: bool,
    ) {
        self.is_canceled.store(value, Ordering::SeqCst);
    }

    pub fn is_completed(&self) -> bool {
        self.is_completed.load(Ordering::SeqCst)
    }

    pub fn set_completed(
        self: &Arc<Self>,
        value: bool,
    ) {
        self.is_completed.store(value, Ordering::SeqCst);
    }

    pub fn cancel(self: &Arc<Self>) {
        self.set_canceled(true);
    }

    pub fn complete(
        self: &Arc<Self>,
        result: ResultType,
    ) {
        self.set_completed(true);

        let (result_lock, cvar) = &*self.result;
        let mut result_guard = result_lock.lock().unwrap();
        *result_guard = Some(result);

        // Notify all waiting threads that the result is available.
        cvar.notify_all();
    }

    pub fn wait_for_completion(self: Arc<Self>) -> ResultType {
        let (result_lock, cvar) = &*self.result;
        let mut result_guard = result_lock.lock().unwrap();

        // Wait until the result is available.
        while result_guard.is_none() {
            result_guard = cvar.wait(result_guard).unwrap();
        }

        result_guard.take().unwrap()
    }

    pub fn subscribe_to_progress_updates(&self) -> Receiver<f32> {
        self.progress_receiver.clone()
    }
}
