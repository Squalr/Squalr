use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::results::list::results_list_response::ResultsListResponse;
use crate::commands::results::results_command::ResultsCommand;
use crate::commands::results::results_response::ResultsResponse;
use crate::squalr_engine::SqualrEngine;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_scanning::scan_settings::ScanSettings;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ResultsListRequest {
    #[structopt(short = "p", long)]
    pub page: u64,

    #[structopt(short = "d", long)]
    pub data_type: DataType,
}

impl EngineRequest for ResultsListRequest {
    type ResponseType = ResultsListResponse;

    fn execute(&self) -> Self::ResponseType {
        let results_page_size = ScanSettings::get_instance().get_results_page_size() as u64;
        let initial_index = self.page * results_page_size;
        let end_index = initial_index + results_page_size;
        let snapshot_lock = SqualrEngine::get_snapshot();
        let snapshot = snapshot_lock.read().unwrap();

        for result_index in initial_index..end_index {
            if let Some(scan_result) = snapshot.get_scan_result(result_index, &self.data_type) {
                Logger::get_instance().log(LogLevel::Info, format!("{:?}", scan_result).as_str(), None);
            }
        }

        ResultsListResponse {}
    }

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Results(ResultsCommand::List {
            results_list_request: self.clone(),
        })
    }
}

impl From<ResultsListResponse> for ResultsResponse {
    fn from(results_list_response: ResultsListResponse) -> Self {
        ResultsResponse::List { results_list_response }
    }
}
