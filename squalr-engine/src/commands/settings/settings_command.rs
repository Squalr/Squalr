use std::sync::Arc;

use crate::commands::engine_request::EngineRequest;
use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::settings::set::settings_set_request::SettingsSetRequest;
use crate::commands::{engine_response::EngineResponse, settings::list::settings_list_request::SettingsListRequest};
use crate::engine_execution_context::EngineExecutionContext;
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

impl SettingsCommand {
    pub fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> EngineResponse {
        match self {
            SettingsCommand::List { settings_list_request } => settings_list_request
                .execute(execution_context)
                .to_engine_response(),
            SettingsCommand::Set { settings_set_request } => settings_set_request
                .execute(execution_context)
                .to_engine_response(),
        }
    }
}
