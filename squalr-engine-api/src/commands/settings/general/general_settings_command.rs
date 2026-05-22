use crate::commands::settings::general::list::general_settings_list_request::GeneralSettingsListRequest;
use crate::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GeneralSettingsCommand {
    List {
        general_settings_list_request: GeneralSettingsListRequest,
    },
    Set {
        general_settings_set_request: GeneralSettingsSetRequest,
    },
}
