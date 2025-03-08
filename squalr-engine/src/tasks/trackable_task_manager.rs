use squalr_engine_api::structures::tasks::engine_trackable_task_handle::EngineTrackableTaskHandle;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct TrackableTaskManager {
    tasks: Arc<Mutex<HashMap<String, EngineTrackableTaskHandle>>>,
}

impl TrackableTaskManager {
    pub fn new() -> Self {
        TrackableTaskManager {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_task(
        &self,
        trackable_task_handle: EngineTrackableTaskHandle,
    ) {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.insert(trackable_task_handle.task_identifier.clone(), trackable_task_handle);
        }
    }

    pub fn unregister_task(
        &self,
        task_identifier: &String,
    ) {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.remove(task_identifier);
        }
    }

    pub fn get_task_handle(
        &self,
        task_identifier: &String,
    ) -> Option<EngineTrackableTaskHandle> {
        self.tasks.lock().ok()?.get(task_identifier).cloned()
    }
}
