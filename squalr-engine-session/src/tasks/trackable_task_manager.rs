use log::error;
use squalr_engine_api::structures::tasks::trackable_task::TrackableTask;
use squalr_engine_api::structures::tasks::trackable_task_handle::TrackableTaskHandle;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct TrackableTaskManager {
    tasks: Arc<RwLock<HashMap<String, Arc<TrackableTask>>>>,
}

impl TrackableTaskManager {
    pub fn new() -> Self {
        TrackableTaskManager {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a task for tracking.
    pub fn register_task(
        &self,
        trackable_task: Arc<TrackableTask>,
    ) {
        match self.tasks.write() {
            Ok(mut tasks_guard) => {
                tasks_guard.insert(trackable_task.get_task_identifier(), trackable_task);
            }
            Err(error) => {
                error!("Error: Failed to acquire write lock in register_task: {}", error);
            }
        }
    }

    /// Unregisters a task from tracking.
    pub fn unregister_task(
        &self,
        task_identifier: &String,
    ) {
        match self.tasks.write() {
            Ok(mut tasks_guard) => {
                tasks_guard.remove(task_identifier);
            }
            Err(error) => {
                error!("Error: Failed to acquire write lock in unregister_task: {}", error);
            }
        }
    }

    /// Cancels a task and unregisters it.
    pub fn cancel_task(
        &self,
        task_identifier: &String,
    ) {
        match self.tasks.read() {
            Ok(tasks_guard) => {
                if let Some(task) = tasks_guard.get(task_identifier) {
                    task.cancel();
                }
            }
            Err(error) => {
                error!("Error: Failed to acquire read lock in cancel_task: {}", error);
                return;
            }
        }

        self.unregister_task(task_identifier);
    }

    /// Gets a handle to a tracked task.
    pub fn get_task_handle(
        &self,
        task_identifier: &String,
    ) -> Option<TrackableTaskHandle> {
        match self.tasks.read() {
            Ok(tasks_guard) => tasks_guard
                .get(task_identifier)
                .map(|task| task.get_task_handle()),
            Err(error) => {
                error!("Error: Failed to acquire read lock in get_task_handle: {}", error);
                None
            }
        }
    }
}
