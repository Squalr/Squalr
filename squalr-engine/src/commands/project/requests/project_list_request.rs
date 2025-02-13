use crate::commands::command_handler::CommandHandler;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectListRequest {}

impl CommandHandler for ProjectListRequest {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
    }
}
