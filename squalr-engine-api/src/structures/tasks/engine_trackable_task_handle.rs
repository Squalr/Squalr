use crate::structures::tasks::trackable_task_handle::TrackableTaskHandle;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug)]
pub struct EngineTrackableTaskHandle {
    pub name: String,
    pub progress: f32,
    pub task_identifier: String,
    pub cancellation_token: Arc<AtomicBool>,
}

impl EngineTrackableTaskHandle {
    pub fn to_user_handle(&self) -> TrackableTaskHandle {
        TrackableTaskHandle {
            name: self.name.clone(),
            progress: self.progress,
            task_identifier: self.task_identifier.clone(),
        }
    }

    pub fn set_progress(
        &self,
        progress: f32,
    ) {
    }

    pub fn complete(&self) {
        self.cancellation_token.store(true, Ordering::SeqCst);
    }

    pub fn get_cancellation_token(&self) -> Arc<AtomicBool> {
        self.cancellation_token.clone()
    }

    pub fn cancel(&self) {
        self.cancellation_token.store(true, Ordering::SeqCst);
    }
}
