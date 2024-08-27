use crate::tasks::trackable_task::TrackableTask;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{
    Arc,
    Mutex,
    Once,
};

pub struct TrackableTaskManager<T: Send + Sync> {
    tasks: Arc<Mutex<HashMap<String, Arc<TrackableTask<T>>>>>,
}

impl<T: Send + Sync + 'static> TrackableTaskManager<T> {
    fn new() -> Self {
        TrackableTaskManager {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_instance() -> &'static Mutex<Box<dyn Any + Send + Sync>> {
        static mut INSTANCE: *const Mutex<Box<dyn Any + Send + Sync>> = 0 as *const _;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let singleton = Mutex::new(Box::new(TrackableTaskManager::<T>::new()) as Box<dyn Any + Send + Sync>);
                INSTANCE = Box::into_raw(Box::new(singleton));
            });

            return &*INSTANCE;
        }
    }

    pub fn register_task(
        &self,
        task: Arc<TrackableTask<T>>,
    ) {
        self.tasks
            .lock()
            .unwrap()
            .insert(task.get_task_identifier(), task);
    }

    pub fn remove_task(
        &self,
        task_identifier: &String,
    ) {
        self.tasks.lock().unwrap().remove(task_identifier);
    }

    pub fn get_task(
        &self,
        task_identifier: &String,
    ) -> Option<Arc<TrackableTask<T>>> {
        self.tasks.lock().unwrap().get(task_identifier).cloned()
    }
}
