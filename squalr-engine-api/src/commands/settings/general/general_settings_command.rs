use crate::commands::settings::general::list::general_settings_list_request::GeneralSettingsListRequest;
use crate::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum GeneralSettingsCommand {
    List {
        #[structopt(flatten)]
        general_settings_list_request: GeneralSettingsListRequest,
    },
    Set {
        #[structopt(flatten)]
        general_settings_set_request: GeneralSettingsSetRequest,
    },
}
