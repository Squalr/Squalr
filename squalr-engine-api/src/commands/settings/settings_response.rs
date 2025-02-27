use crate::commands::settings::list::settings_list_response::SettingsListResponse;
use crate::commands::settings::set::settings_set_response::SettingsSetResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SettingsResponse {
    Set { settings_set_response: SettingsSetResponse },
    List { settings_list_response: SettingsListResponse },
}
