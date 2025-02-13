use crate::commands::command_handler::CommandHandler;
use crate::commands::settings::requests::settings_list_request::SettingsListRequest;
use crate::commands::settings::requests::settings_set_request::SettingsSetRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum SettingsCommand {
    List {
        #[structopt(flatten)]
        settings_list_request: SettingsListRequest,
    },
    Set {
        #[structopt(flatten)]
        settings_set_request: SettingsSetRequest,
    },
}

impl CommandHandler for SettingsCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            SettingsCommand::List { settings_list_request } => {
                settings_list_request.handle(uuid);
            }
            SettingsCommand::Set { settings_set_request } => {
                settings_set_request.handle(uuid);
            }
        }
    }
}
