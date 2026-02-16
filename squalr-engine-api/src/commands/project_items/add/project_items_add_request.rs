use crate::commands::project_items::add::project_items_add_response::ProjectItemsAddResponse;
use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::structures::scan_results::scan_result_ref::ScanResultRef;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsAddRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<ScanResultRef>,
    #[structopt(long)]
    pub target_directory_path: Option<PathBuf>,
}

impl UnprivilegedCommandRequest for ProjectItemsAddRequest {
    type ResponseType = ProjectItemsAddResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::Add {
            project_items_add_request: self.clone(),
        })
    }
}

impl From<ProjectItemsAddResponse> for ProjectItemsResponse {
    fn from(project_items_add_response: ProjectItemsAddResponse) -> Self {
        ProjectItemsResponse::Add { project_items_add_response }
    }
}
