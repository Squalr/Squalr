use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_command_request::EngineCommandRequest;
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

impl EngineCommandRequest for TrackableTasksCancelRequest {
    type ResponseType = TrackableTasksCancelResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::TrackableTasks(TrackableTasksCommand::Cancel {
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
