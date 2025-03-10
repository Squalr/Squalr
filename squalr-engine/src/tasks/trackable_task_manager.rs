use dashmap::DashMap;
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use squalr_engine_api::structures::tasks::trackable_task_handle::TrackableTaskHandle;
use std::sync::Arc;

pub struct TrackableTaskManager {
    tasks: Arc<DashMap<String, Arc<TrackableTask>>>,
}

impl TrackableTaskManager {
    pub fn new() -> Self {
        TrackableTaskManager {
            tasks: Arc::new(DashMap::new()),
        }
    }

    /// Registers a task for tracking.
    pub fn register_task(
        &self,
        trackable_task: Arc<TrackableTask>,
    ) {
        self.tasks
            .insert(trackable_task.get_task_identifier(), trackable_task);
    }

    /// Unregisters a task from tracking
    pub fn unregister_task(
        &self,
        task_identifier: &String,
    ) {
        self.tasks.remove(task_identifier);
    }

    /// Registers a task for tracking.
    pub fn cancel_task(
        &self,
        task_identifier: &String,
    ) {
        if let Some(task) = self.tasks.get(task_identifier) {
            task.cancel();
        }

        self.unregister_task(task_identifier);
    }

    pub fn get_task_handle(
        &self,
        task_identifier: &String,
    ) -> Option<TrackableTaskHandle> {
        match self.tasks.get(task_identifier) {
            Some(task) => Some(task.get_task_handle()),
            None => None,
        }
    }
}
