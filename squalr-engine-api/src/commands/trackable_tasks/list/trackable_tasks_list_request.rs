use crate::commands::trackable_tasks::list::trackable_tasks_list_response::TrackableTasksListResponse;
use crate::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand;
use crate::commands::trackable_tasks::trackable_tasks_response::TrackableTasksResponse;
use crate::commands::{engine_command::EngineCommand, engine_command_request::EngineCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct TrackableTasksListRequest {}

impl EngineCommandRequest for TrackableTasksListRequest {
    type ResponseType = TrackableTasksListResponse;

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::TrackableTasks(TrackableTasksCommand::List {
            trackable_tasks_list_request: self.clone(),
        })
    }
}

impl From<TrackableTasksListResponse> for TrackableTasksResponse {
    fn from(trackable_tasks_list_response: TrackableTasksListResponse) -> Self {
        TrackableTasksResponse::List { trackable_tasks_list_response }
    }
}
