use std::collections::HashMap;
use std::sync::{Arc, Mutex, Once};
use crate::tasks::trackable_task::TrackableTask;

pub struct TrackableTaskManager {
    tasks: Arc<Mutex<HashMap<String, Arc<TrackableTask>>>>,
}

impl TrackableTaskManager {
    fn new() -> Self {
        TrackableTaskManager {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn instance() -> &'static TrackableTaskManager {
        static mut SINGLETON: Option<TrackableTaskManager> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = TrackableTaskManager::new();
                SINGLETON = Some(instance);
            });

            SINGLETON.as_ref().unwrap()
        }
    }

    pub fn register_task(&self, task: Arc<TrackableTask>) {
        let identifier = task.task_identifier;
        self.tasks.lock().unwrap().insert(identifier.to_string(), task);
    }

    pub fn remove_task(&self, task_identifier: uuid::Uuid) {
        self.tasks.lock().unwrap().remove(&task_identifier.to_string());
    }

    pub fn get_task(&self, task_identifier: uuid::Uuid) -> Option<Arc<TrackableTask>> {
        self.tasks.lock().unwrap().get(&task_identifier.to_string()).cloned()
    }
}
