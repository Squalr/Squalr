pub mod settings_command_list;
pub mod settings_command_set;

use crate::command_handlers::settings::settings_command_list::handle_settings_list;
use crate::command_handlers::settings::settings_command_set::handle_settings_set;
use crate::commands::settings::settings_command::SettingsCommand;
use uuid::Uuid;

pub fn handle_settings_command(
    cmd: SettingsCommand,
    uuid: Uuid,
) {
    match cmd {
        SettingsCommand::Set { .. } => handle_settings_set(cmd, uuid),
        SettingsCommand::List { .. } => handle_settings_list(cmd, uuid),
    }
}
