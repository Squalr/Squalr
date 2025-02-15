use crate::commands::settings::list::settings_list_request::SettingsListRequest;
use crate::commands::settings::set::settings_set_request::SettingsSetRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

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
