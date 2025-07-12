use crate::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use crate::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ScanSettingsCommand {
    List {
        #[structopt(flatten)]
        scan_settings_list_request: ScanSettingsListRequest,
    },
    Set {
        #[structopt(flatten)]
        scan_settings_set_request: ScanSettingsSetRequest,
    },
}
