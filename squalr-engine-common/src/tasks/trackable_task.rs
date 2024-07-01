use crate::tasks::trackable_task_manager::TrackableTaskManager;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task::JoinHandle;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use futures::future::BoxFuture;

type ProgressCallback = Box<dyn Fn(f32) + Send + Sync>;
type TaskCanceledCallback<T> = Box<dyn Fn(Arc<TrackableTask<T>>) + Send>;
type TaskCompletedCallback<T> = Box<dyn Fn(Arc<TrackableTask<T>>) + Send>;
type UpdateProgress = Arc<dyn Fn(f32) + Send + Sync>;

pub struct TrackableTask<T: Send + Sync> {
    name: String,
    progress: Arc<Mutex<f32>>,
    pub task_identifier: Uuid,
    is_canceled: Arc<AtomicBool>,
    is_completed: Arc<AtomicBool>,
    on_canceled_event: Arc<Mutex<Option<TaskCanceledCallback<T>>>>,
    on_completed_event: Arc<Mutex<Option<TaskCompletedCallback<T>>>>,
    on_progress_updated_event: Arc<Mutex<Option<ProgressCallback>>>,
    result: Arc<Mutex<Option<T>>>,
    handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    cancellation_token: CancellationToken,
    notify: Arc<Notify>,
}

impl<T: Send + Sync + 'static> TrackableTask<T> {
    pub fn create(name: String, task_identifier: Option<Uuid>, with_logging: bool) -> Arc<Self> {
        let task_identifier = task_identifier.unwrap_or_else(Uuid::new_v4);

        let task = TrackableTask {
            name,
            progress: Arc::new(Mutex::new(0.0)),
            task_identifier,
            is_canceled: Arc::new(AtomicBool::new(false)),
            is_completed: Arc::new(AtomicBool::new(false)),
            on_canceled_event: Arc::new(Mutex::new(None)),
            on_completed_event: Arc::new(Mutex::new(None)),
            on_progress_updated_event: Arc::new(Mutex::new(None)),
            result: Arc::new(Mutex::new(None)),
            handle: Arc::new(Mutex::new(None)),
            cancellation_token: CancellationToken::new(),
            notify: Arc::new(Notify::new()),
        };

        Arc::new(task)
    }

    pub fn set_progress(self: &Arc<Self>, progress: f32) {
        let mut progress_guard = self.progress.lock().unwrap();
        *progress_guard = progress;

        if let Some(callback) = &*self.on_progress_updated_event.lock().unwrap() {
            callback(progress);
        }
    }

    pub fn cancel(self: &Arc<Self>) {
        self.is_canceled.store(true, Ordering::SeqCst);
        self.cancellation_token.cancel();

        if let Some(callback) = &*self.on_canceled_event.lock().unwrap() {
            callback(self.clone());
        }
    }

    pub fn complete(self: &Arc<Self>, result: T) {
        self.is_completed.store(true, Ordering::SeqCst);

        {
            let mut result_guard = self.result.lock().unwrap();
            *result_guard = Some(result);
        }

        if let Some(callback) = &*self.on_completed_event.lock().unwrap() {
            callback(self.clone());
        }

        TrackableTaskManager::<T>::instance().lock().unwrap().downcast_mut::<TrackableTaskManager<T>>().unwrap().remove_task(self.task_identifier);
    }

    pub fn progress_callback(self: &Arc<Self>) -> UpdateProgress {
        let progress_arc = self.progress.clone();
        Arc::new(move |progress: f32| {
            let mut progress_guard = progress_arc.lock().unwrap();
            *progress_guard = progress;
        })
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub fn add_handle(self: &Arc<Self>, handle: JoinHandle<()>) {
        let mut handle_guard = self.handle.lock().unwrap();
        *handle_guard = Some(handle);
    }

    pub fn on_canceled<F>(self: &Arc<Self>, callback: F) 
    where F: Fn(Arc<TrackableTask<T>>) + Send + 'static,
    {
        *self.on_canceled_event.lock().unwrap() = Some(Box::new(callback));
    }

    pub fn on_completed<F>(self: &Arc<Self>, callback: F) 
    where F: Fn(Arc<TrackableTask<T>>) + Send + 'static,
    {
        *self.on_completed_event.lock().unwrap() = Some(Box::new(callback));
    }

    pub fn on_progress_updated<F>(self: &Arc<Self>, callback: F) 
    where F: Fn(f32) + Send + Sync + 'static,
    {
        *self.on_progress_updated_event.lock().unwrap() = Some(Box::new(callback));
    }

    pub fn wait_for_completion(self: Arc<Self>) -> BoxFuture<'static, T> {
        let notify = self.notify.clone();
        let is_completed = self.is_completed.clone();
        let result = self.result.clone();

        Box::pin(async move {
            notify.notified().await;
            assert!(is_completed.load(Ordering::SeqCst));
            
            let mut result_guard = self.result.lock().unwrap();
            result_guard.take().unwrap()
        })
    }
}
