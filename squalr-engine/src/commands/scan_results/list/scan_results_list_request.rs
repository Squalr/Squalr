use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::scan_results::list::scan_results_list_response::ScanResultsListResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::engine_execution_context::EngineExecutionContext;
use serde::{Deserialize, Serialize};
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_scanning::scan_settings::ScanSettings;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanResultsListRequest {
    #[structopt(short = "p", long)]
    pub page_index: u64,

    #[structopt(short = "d", long)]
    pub data_type: DataType,
}

impl EngineRequest for ScanResultsListRequest {
    type ResponseType = ScanResultsListResponse;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> Self::ResponseType {
        let results_page_size = ScanSettings::get_instance().get_results_page_size() as u64;
        let initial_index = self.page_index * results_page_size;
        let end_index = initial_index + results_page_size;
        let mut scan_results = vec![];

        if let Ok(snapshot) = execution_context.get_snapshot().read() {
            for result_index in initial_index..end_index {
                if let Some(scan_result) = snapshot.get_scan_result(result_index, &self.data_type) {
                    scan_results.push(scan_result);
                }
            }
        }

        ScanResultsListResponse {
            scan_results,
            page_index: self.page_index,
            page_size: results_page_size,
        }
    }

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Results(ScanResultsCommand::List {
            results_list_request: self.clone(),
        })
    }
}

impl From<ScanResultsListResponse> for ScanResultsResponse {
    fn from(results_list_response: ScanResultsListResponse) -> Self {
        ScanResultsResponse::List { results_list_response }
    }
}
