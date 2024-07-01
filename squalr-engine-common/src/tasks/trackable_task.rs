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
    pub task_identifier: Uuid,
    is_canceled: Arc<AtomicBool>,
    is_completed: Arc<AtomicBool>,
    result: Arc<Mutex<Option<T>>>,
    handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    cancellation_token: CancellationToken,
    notify: Arc<Notify>,
    pub progress_sender: broadcast::Sender<f32>,
    progress_receiver: broadcast::Receiver<f32>,
    completion_sender: broadcast::Sender<()>,
    completion_receiver: broadcast::Receiver<()>,
    cancel_sender: watch::Sender<bool>,
    cancel_receiver: watch::Receiver<bool>,
}

impl<T: Send + Sync + 'static> TrackableTask<T> {
    pub fn create(name: String, task_identifier: Option<Uuid>) -> Arc<Self> {
        let task_identifier = task_identifier.unwrap_or_else(Uuid::new_v4);
        let (progress_sender, progress_receiver) = broadcast::channel(100);
        let (completion_sender, completion_receiver) = broadcast::channel(1);
        let (cancel_sender, cancel_receiver) = watch::channel(false);

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
            progress_receiver,
            completion_sender,
            completion_receiver,
            cancel_sender,
            cancel_receiver,
        };

        Arc::new(task)
    }

    pub fn set_progress(self: &Arc<Self>, progress: f32) {
        let mut progress_guard = self.progress.lock().unwrap();
        *progress_guard = progress;
        let _ = self.progress_sender.send(progress);
    }

    pub fn cancel(self: &Arc<Self>) {
        self.is_canceled.store(true, Ordering::SeqCst);
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
        TrackableTaskManager::<T>::instance().lock().unwrap().downcast_mut::<TrackableTaskManager<T>>().unwrap().remove_task(self.task_identifier);
    }

    pub fn progress_receiver(&self) -> broadcast::Receiver<f32> {
        self.progress_sender.subscribe()
    }

    pub fn completion_receiver(&self) -> broadcast::Receiver<()> {
        self.completion_sender.subscribe()
    }

    pub fn cancel_receiver(&self) -> watch::Receiver<bool> {
        self.cancel_receiver.clone()
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub fn add_handle(self: &Arc<Self>, handle: JoinHandle<()>) {
        let mut handle_guard = self.handle.lock().unwrap();
        *handle_guard = Some(handle);
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
