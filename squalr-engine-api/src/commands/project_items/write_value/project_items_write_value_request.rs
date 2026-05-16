use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::project_items::write_value::project_items_write_value_response::ProjectItemsWriteValueResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsWriteValueRequest {
    #[structopt(short = "p", long = "project-item-path", parse(from_os_str))]
    pub project_item_path: PathBuf,

    #[structopt(long = "field", default_value = "value")]
    pub field_name: String,

    #[structopt(short = "v", long)]
    pub anonymous_value_string: AnonymousValueString,
}

impl UnprivilegedCommandRequest for ProjectItemsWriteValueRequest {
    type ResponseType = ProjectItemsWriteValueResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::WriteValue {
            project_items_write_value_request: self.clone(),
        })
    }
}

impl From<ProjectItemsWriteValueResponse> for ProjectItemsResponse {
    fn from(project_items_write_value_response: ProjectItemsWriteValueResponse) -> Self {
        ProjectItemsResponse::WriteValue {
            project_items_write_value_response,
        }
    }
}
