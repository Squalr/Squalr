use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::commands::trackable_tasks::cancel::trackable_tasks_cancel_response::TrackableTasksCancelResponse;
use crate::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand;
use crate::commands::trackable_tasks::trackable_tasks_response::TrackableTasksResponse;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct TrackableTasksCancelRequest {
    #[structopt(short = "t", long)]
    pub task_id: String,
}

impl PrivilegedCommandRequest for TrackableTasksCancelRequest {
    type ResponseType = TrackableTasksCancelResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::TrackableTasks(TrackableTasksCommand::Cancel {
            trackable_tasks_cancel_request: self.clone(),
        })
    }
}

impl From<TrackableTasksCancelResponse> for TrackableTasksResponse {
    fn from(trackable_tasks_cancel_response: TrackableTasksCancelResponse) -> Self {
        TrackableTasksResponse::Cancel {
            trackable_tasks_cancel_response,
        }
    }
}
