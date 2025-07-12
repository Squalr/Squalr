use crate::commands::trackable_tasks::cancel::trackable_tasks_cancel_request::TrackableTasksCancelRequest;
use crate::commands::trackable_tasks::list::trackable_tasks_list_request::TrackableTasksListRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum TrackableTasksCommand {
    List {
        #[structopt(flatten)]
        trackable_tasks_list_request: TrackableTasksListRequest,
    },
    Cancel {
        #[structopt(flatten)]
        trackable_tasks_cancel_request: TrackableTasksCancelRequest,
    },
}
