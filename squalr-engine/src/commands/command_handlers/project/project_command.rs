use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ProjectCommand {
    /// List all projects
    List,
    // Add other project commands here
}
