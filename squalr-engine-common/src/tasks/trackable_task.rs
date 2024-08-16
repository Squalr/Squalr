use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use uuid::Uuid;
use std::sync::mpsc::{self, Sender, Receiver};

pub struct TrackableTask<T: Send + Sync> {
    name: String,
    progress: Arc<Mutex<f32>>,
    task_identifier: String,
    is_canceled: Arc<AtomicBool>,
    is_completed: Arc<AtomicBool>,
    result: Arc<(Mutex<Option<T>>, Condvar)>, // Use a tuple of Mutex and Condvar
    progress_sender: Sender<f32>,
    listener_senders: Arc<Mutex<Vec<Sender<f32>>>>, // To store multiple listener senders
}

impl<T: Send + Sync + 'static> TrackableTask<T> {
    pub fn create(name: String, task_identifier: Option<String>) -> Arc<Self> {
        let task_identifier = task_identifier.unwrap_or_else(|| Uuid::new_v4().to_string());
        let (progress_sender, progress_receiver) = mpsc::channel();

        let task = TrackableTask {
            name,
            progress: Arc::new(Mutex::new(0.0)),
            task_identifier,
            is_canceled: Arc::new(AtomicBool::new(false)),
            is_completed: Arc::new(AtomicBool::new(false)),
            result: Arc::new((Mutex::new(None), Condvar::new())), // Initialize Condvar
            progress_sender,
            listener_senders: Arc::new(Mutex::new(vec![])), // Initialize empty Vec for listeners
        };

        // Start a thread that listens to progress updates and broadcasts them to all listeners
        let listener_senders_clone = task.listener_senders.clone();
        thread::spawn(move || {
            let receiver = progress_receiver;
            while let Ok(progress) = receiver.recv() {
                let listener_senders = listener_senders_clone.lock().unwrap();
                for sender in listener_senders.iter() {
                    let _ = sender.send(progress);
                }
            }
        });

        Arc::new(task)
    }

    pub fn get_progress(self: &Arc<Self>) -> f32 {
        let progress_guard = self.progress.lock().unwrap();
        
        return *progress_guard;
    }

    pub fn set_progress(self: &Arc<Self>, progress: f32) {
        let mut progress_guard = self.progress.lock().unwrap();
        *progress_guard = progress;
        let _ = self.progress_sender.send(progress);
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_task_identifier(&self) -> String {
        return self.task_identifier.clone();
    }

    pub fn get_cancellation_token(&self) -> Arc<AtomicBool> {
        return self.is_canceled.clone();
    }

    pub fn set_canceled(self: &Arc<Self>, value: bool) {
        self.is_canceled.store(value, Ordering::SeqCst);
    }

    pub fn is_completed(&self) -> bool {
        return self.is_completed.load(Ordering::SeqCst);
    }

    pub fn set_completed(self: &Arc<Self>, value: bool) {
        self.is_completed.store(value, Ordering::SeqCst);
    }

    pub fn cancel(self: &Arc<Self>) {
        self.set_canceled(true);
    }

    pub fn complete(self: &Arc<Self>, result: T) {
        self.set_completed(true);

        {
            let (result_lock, cvar) = &*self.result;
            let mut result_guard = result_lock.lock().unwrap();
            *result_guard = Some(result);

            cvar.notify_all(); // Notify all waiting threads that the result is available
        }
    }

    pub fn wait_for_completion(self: Arc<Self>) -> T {
        let (result_lock, cvar) = &*self.result;
        let mut result_guard = result_lock.lock().unwrap();
        
        // Wait until the result is available
        while result_guard.is_none() {
            result_guard = cvar.wait(result_guard).unwrap();
        }
        
        return result_guard.take().unwrap();
    }

    pub fn add_listener(&self) -> Receiver<f32> {
        let (sender, receiver) = mpsc::channel();
        self.listener_senders.lock().unwrap().push(sender);

        return receiver;
    }
}
