use crate::commands::project::list::project_list_request::ProjectListRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectCommand {
    /// List all projects
    List {
        #[structopt(flatten)]
        project_list_request: ProjectListRequest,
    },
}
