use crate::command_handlers::settings::settings_command::SettingsCommand;

pub fn handle_settings_set(cmd: &mut SettingsCommand) {
    if let SettingsCommand::Set {} = cmd {}
}
