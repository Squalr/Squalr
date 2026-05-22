use crate::commands::trackable_tasks::cancel::trackable_tasks_cancel_request::TrackableTasksCancelRequest;
use crate::commands::trackable_tasks::list::trackable_tasks_list_request::TrackableTasksListRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrackableTasksCommand {
    List {
        trackable_tasks_list_request: TrackableTasksListRequest,
    },
    Cancel {
        trackable_tasks_cancel_request: TrackableTasksCancelRequest,
    },
}
