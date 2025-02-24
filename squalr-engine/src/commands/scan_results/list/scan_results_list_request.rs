use crate::commands::engine_command::EngineCommand;
use crate::commands::engine_request::EngineRequest;
use crate::commands::scan_results::list::scan_results_list_response::ScanResultsListResponse;
use crate::commands::scan_results::scan_results_command::ScanResultsCommand;
use crate::commands::scan_results::scan_results_response::ScanResultsResponse;
use crate::engine_execution_context::EngineExecutionContext;
use serde::{Deserialize, Serialize};
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use squalr_engine_memory::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_scanning::results::scan_result::ScanResult;
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

        // Collect modules if possible so that we can resolve whether individual addresses are static later.
        let modules = if let Some(opened_process_info) = execution_context.get_opened_process() {
            MemoryQueryer::get_instance().get_modules(&opened_process_info)
        } else {
            vec![]
        };

        if let Ok(snapshot) = execution_context.get_snapshot().read() {
            for result_index in initial_index..end_index {
                if let Some(mut scan_result_base_address) = snapshot.get_scan_result_address(result_index, &self.data_type) {
                    let mut current_value = self.data_type.to_default_value();
                    let previous_value = self.data_type.to_default_value();
                    let mut module_name = String::default();

                    // Best-effort attempt to read the values for this scan result.
                    if let Some(opened_process_info) = execution_context.get_opened_process() {
                        let _ = MemoryReader::get_instance().read(&opened_process_info, scan_result_base_address, &mut current_value);
                    }

                    // Check whether this scan result belongs to a module (ie check if the address is static).
                    if let Some((found_module_name, address)) = MemoryQueryer::get_instance().address_to_module(scan_result_base_address, &modules) {
                        module_name = found_module_name;
                        scan_result_base_address = address;
                    }

                    scan_results.push(ScanResult::new(scan_result_base_address, module_name, current_value, previous_value));
                } else {
                    break;
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
