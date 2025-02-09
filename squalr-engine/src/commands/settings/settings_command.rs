use crate::commands::command_handler::CommandHandler;
use crate::commands::settings::handlers::settings_command_list::handle_settings_list;
use crate::commands::settings::handlers::settings_command_set::handle_settings_set;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum SettingsCommand {
    List {
        #[structopt(short = "s", long)]
        scan: bool,
        #[structopt(short = "m", long)]
        memory: bool,
        #[structopt(short = "a", long)]
        list_all: bool,
    },
    Set {
        #[structopt(name = "setting")]
        setting_command: String,
    },
}

impl CommandHandler for SettingsCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            SettingsCommand::List { scan, memory, list_all } => {
                handle_settings_list(*scan, *memory, *list_all, uuid);
            }
            SettingsCommand::Set { setting_command } => {
                handle_settings_set(setting_command, uuid);
            }
        }
    }
}
