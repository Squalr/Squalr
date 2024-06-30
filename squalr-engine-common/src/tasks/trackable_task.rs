use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::tasks::trackable_task_manager::TrackableTaskManager;

type ProgressCallback = Box<dyn Fn(f32) + Send>;
type TaskCanceledCallback = Box<dyn Fn(Arc<TrackableTask>) + Send>;
type TaskCompletedCallback = Box<dyn Fn(Arc<TrackableTask>) + Send>;

#[derive(Clone)]
pub struct TrackableTask {
    name: String,
    progress: Arc<Mutex<f32>>,
    pub task_identifier: uuid::Uuid,
    is_canceled: Arc<AtomicBool>,
    is_completed: Arc<AtomicBool>,
    on_canceled_event: Arc<Mutex<Option<TaskCanceledCallback>>>,
    on_completed_event: Arc<Mutex<Option<TaskCompletedCallback>>>,
    on_progress_updated_event: Arc<Mutex<Option<ProgressCallback>>>,
}

impl TrackableTask {
    pub fn new(name: String) -> Arc<Self> {
        let task = TrackableTask {
            name,
            progress: Arc::new(Mutex::new(0.0)),
            task_identifier: uuid::Uuid::new_v4(),
            is_canceled: Arc::new(AtomicBool::new(false)),
            is_completed: Arc::new(AtomicBool::new(false)),
            on_canceled_event: Arc::new(Mutex::new(None)),
            on_completed_event: Arc::new(Mutex::new(None)),
            on_progress_updated_event: Arc::new(Mutex::new(None)),
        };

        let task_arc = Arc::new(task);
        TrackableTaskManager::instance().register_task(task_arc.clone());

        return task_arc;
    }

    pub fn set_progress(&self, progress: f32) {
        let mut progress_guard = self.progress.lock().unwrap();
        *progress_guard = progress;

        if let Some(callback) = &*self.on_progress_updated_event.lock().unwrap() {
            callback(progress);
        }
    }

    pub fn cancel(&self) {
        self.is_canceled.store(true, Ordering::SeqCst);

        if let Some(callback) = &*self.on_canceled_event.lock().unwrap() {
            callback(Arc::new(self.clone()));
        }
    }

    pub fn complete(&self) {
        self.is_completed.store(true, Ordering::SeqCst);

        if let Some(callback) = &*self.on_completed_event.lock().unwrap() {
            callback(Arc::new(self.clone()));
        }

        TrackableTaskManager::instance().remove_task(self.task_identifier);
    }

    pub fn on_canceled<F>(&self, callback: F)
    where
        F: Fn(Arc<TrackableTask>) + Send + 'static,
    {
        *self.on_canceled_event.lock().unwrap() = Some(Box::new(callback));
    }

    pub fn on_completed<F>(&self, callback: F)
    where
        F: Fn(Arc<TrackableTask>) + Send + 'static,
    {
        *self.on_completed_event.lock().unwrap() = Some(Box::new(callback));
    }

    pub fn on_progress_updated<F>(&self, callback: F)
    where
        F: Fn(f32) + Send + 'static,
    {
        *self.on_progress_updated_event.lock().unwrap() = Some(Box::new(callback));
    }
}
