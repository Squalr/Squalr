use crate::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use crate::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanSettingsCommand {
    List { scan_settings_list_request: ScanSettingsListRequest },
    Set { scan_settings_set_request: ScanSettingsSetRequest },
}
