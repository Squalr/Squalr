use crate::tasks::trackable_task_manager::TrackableTaskManager;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task::JoinHandle;
use tokio::sync::{Notify, broadcast, watch};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use futures::future::BoxFuture;

pub struct TrackableTask<T: Send + Sync> {
    name: String,
    progress: Arc<Mutex<f32>>,
    task_identifier: String,
    is_canceled: Arc<AtomicBool>,
    is_completed: Arc<AtomicBool>,
    result: Arc<Mutex<Option<T>>>,
    handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    cancellation_token: CancellationToken,
    notify: Arc<Notify>,
    progress_sender: broadcast::Sender<f32>,
    completion_sender: broadcast::Sender<()>,
    cancel_sender: watch::Sender<bool>,
}

impl<T: Send + Sync + 'static> TrackableTask<T> {
    pub fn create(name: String, task_identifier: Option<String>) -> Arc<Self> {
        let task_identifier = task_identifier.unwrap_or_else(|| Uuid::new_v4().to_string());
        let (progress_sender, _) = broadcast::channel(100);
        let (completion_sender, _) = broadcast::channel(1);
        let (cancel_sender, _) = watch::channel(false);

        let task = TrackableTask {
            name,
            progress: Arc::new(Mutex::new(0.0)),
            task_identifier,
            is_canceled: Arc::new(AtomicBool::new(false)),
            is_completed: Arc::new(AtomicBool::new(false)),
            result: Arc::new(Mutex::new(None)),
            handle: Arc::new(Mutex::new(None)),
            cancellation_token: CancellationToken::new(),
            notify: Arc::new(Notify::new()),
            progress_sender,
            completion_sender,
            cancel_sender,
        };

        Arc::new(task)
    }

    pub fn add_handle(self: &Arc<Self>, handle: JoinHandle<()>) {
        let mut handle_guard = self.handle.lock().unwrap();
        *handle_guard = Some(handle);
    }

    pub fn get_progress(self: &Arc<Self>) -> f32 {
        let progress_guard = self.progress.lock().unwrap();
        *progress_guard
    }

    pub fn set_progress(self: &Arc<Self>, progress: f32) {
        let mut progress_guard = self.progress.lock().unwrap();
        *progress_guard = progress;
        let _ = self.progress_sender.send(progress);
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_task_identifier(&self) -> String {
        self.task_identifier.clone()
    }

    pub fn is_canceled(self: &Arc<Self>) -> bool {
        self.is_canceled.load(Ordering::SeqCst)
    }

    pub fn set_canceled(self: &Arc<Self>, value: bool) {
        self.is_canceled.store(value, Ordering::SeqCst);
    }

    pub fn get_cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub fn get_progress_sender(&self) -> broadcast::Sender<f32> {
        self.progress_sender.clone()
    }

    pub fn get_progress_receiver(&self) -> broadcast::Receiver<f32> {
        self.progress_sender.subscribe()
    }

    pub fn is_completed(self: &Arc<Self>) -> bool {
        self.is_completed.load(Ordering::SeqCst)
    }

    pub fn set_completed(self: &Arc<Self>, value: bool) {
        self.is_completed.store(value, Ordering::SeqCst);
    }
    
    pub fn cancel(self: &Arc<Self>) {
        self.set_canceled(true);
        self.cancellation_token.cancel();
        let _ = self.cancel_sender.send(true);
    }

    pub fn complete(self: &Arc<Self>, result: T) {
        self.is_completed.store(true, Ordering::SeqCst);

        {
            let mut result_guard = self.result.lock().unwrap();
            *result_guard = Some(result);
        }

        let _ = self.completion_sender.send(()); 
        self.notify.notify_one();
        TrackableTaskManager::<T>::get_instance().lock().unwrap().downcast_mut::<TrackableTaskManager<T>>().unwrap().remove_task(&self.task_identifier);
    }

    pub fn wait_for_completion(self: Arc<Self>) -> BoxFuture<'static, T> {
        let notify = self.notify.clone();
        let is_completed = self.is_completed.clone();

        Box::pin(async move {
            notify.notified().await;
            assert!(is_completed.load(Ordering::SeqCst));

            let mut result_guard = self.result.lock().unwrap();
            result_guard.take().unwrap()
        })
    }
}
