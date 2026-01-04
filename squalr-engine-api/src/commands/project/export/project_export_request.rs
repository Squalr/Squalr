use crate::commands::project::project_command::ProjectCommand;
use crate::commands::project::project_response::ProjectResponse;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::commands::{project::export::project_export_response::ProjectExportResponse, unprivileged_command::UnprivilegedCommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectExportRequest {
    #[structopt(short = "p", long)]
    pub project_directory_path: Option<PathBuf>,

    #[structopt(short = "n", long)]
    pub project_name: Option<String>,

    #[structopt(short = "o", long)]
    pub open_export_folder: bool,
}

impl UnprivilegedCommandRequest for ProjectExportRequest {
    type ResponseType = ProjectExportResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::Project(ProjectCommand::Export {
            project_export_request: self.clone(),
        })
    }
}

impl From<ProjectExportResponse> for ProjectResponse {
    fn from(project_export_response: ProjectExportResponse) -> Self {
        ProjectResponse::Export { project_export_response }
    }
}
