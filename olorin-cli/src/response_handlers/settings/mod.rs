pub mod handler_settings_list_response;
pub mod handler_settings_set_response;

use crate::response_handlers::settings::handler_settings_list_response::handle_settings_list_response;
use crate::response_handlers::settings::handler_settings_set_response::handle_settings_set_response;
use olorin_engine_api::commands::settings::settings_response::SettingsResponse;

pub fn handle_settings_response(cmd: SettingsResponse) {
    match cmd {
        SettingsResponse::List { settings_list_response } => handle_settings_list_response(settings_list_response),
        SettingsResponse::Set { settings_set_response } => handle_settings_set_response(settings_set_response),
    }
}
