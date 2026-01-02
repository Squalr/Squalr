use crate::commands::trackable_tasks::list::trackable_tasks_list_response::TrackableTasksListResponse;
use crate::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand;
use crate::commands::trackable_tasks::trackable_tasks_response::TrackableTasksResponse;
use crate::commands::{privileged_command::PrivilegedCommand, privileged_command_request::PrivilegedCommandRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct TrackableTasksListRequest {}

impl PrivilegedCommandRequest for TrackableTasksListRequest {
    type ResponseType = TrackableTasksListResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::TrackableTasks(TrackableTasksCommand::List {
            trackable_tasks_list_request: self.clone(),
        })
    }
}

impl From<TrackableTasksListResponse> for TrackableTasksResponse {
    fn from(trackable_tasks_list_response: TrackableTasksListResponse) -> Self {
        TrackableTasksResponse::List { trackable_tasks_list_response }
    }
}
