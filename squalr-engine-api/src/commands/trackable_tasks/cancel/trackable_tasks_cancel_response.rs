use crate::commands::privileged_command_response::PrivilegedCommandResponse;
use crate::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use crate::commands::trackable_tasks::trackable_tasks_response::TrackableTasksResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackableTasksCancelResponse {}

impl TypedPrivilegedCommandResponse for TrackableTasksCancelResponse {
    fn to_engine_response(&self) -> PrivilegedCommandResponse {
        PrivilegedCommandResponse::TrackableTasks(TrackableTasksResponse::Cancel {
            trackable_tasks_cancel_response: self.clone(),
        })
    }

    fn from_engine_response(response: PrivilegedCommandResponse) -> Result<Self, PrivilegedCommandResponse> {
        if let PrivilegedCommandResponse::TrackableTasks(TrackableTasksResponse::Cancel {
            trackable_tasks_cancel_response,
        }) = response
        {
            Ok(trackable_tasks_cancel_response)
        } else {
            Err(response)
        }
    }
}
