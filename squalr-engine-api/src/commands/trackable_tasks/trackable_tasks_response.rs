use crate::commands::trackable_tasks::cancel::trackable_tasks_cancel_response::TrackableTasksCancelResponse;
use crate::commands::trackable_tasks::list::trackable_tasks_list_response::TrackableTasksListResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrackableTasksResponse {
    Cancel {
        trackable_tasks_cancel_response: TrackableTasksCancelResponse,
    },
    List {
        trackable_tasks_list_response: TrackableTasksListResponse,
    },
}
