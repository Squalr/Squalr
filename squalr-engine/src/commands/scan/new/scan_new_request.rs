use std::sync::Arc;

use crate::commands::engine_request::EngineRequest;
use crate::commands::scan::new::scan_new_response::ScanNewResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::commands::{engine_command::EngineCommand, scan::scan_command::ScanCommand};
use crate::engine_execution_context::EngineExecutionContext;
use serde::{Deserialize, Serialize};
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use squalr_engine_common::values::{data_type::DataType, endian::Endian};
use squalr_engine_scanning::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanNewRequest {
    #[structopt(short = "d", long, use_delimiter = true)]
    pub scan_filter_parameters: Vec<ScanFilterParameters>,
    #[structopt(short = "a", long)]
    pub scan_all_primitives: bool,
}

impl EngineRequest for ScanNewRequest {
    type ResponseType = ScanNewResponse;

    fn execute(
        &self,
        execution_context: &Arc<EngineExecutionContext>,
    ) -> Self::ResponseType {
        let mut scan_filter_parameters = self.scan_filter_parameters.clone();

        if self.scan_all_primitives {
            scan_filter_parameters = vec![
                ScanFilterParameters::new_with_value(None, DataType::U8()),
                ScanFilterParameters::new_with_value(None, DataType::U16(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::U32(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::U64(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::I8()),
                ScanFilterParameters::new_with_value(None, DataType::I16(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::I32(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::I64(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::F32(Endian::Little)),
                ScanFilterParameters::new_with_value(None, DataType::F64(Endian::Little)),
            ];
        }

        if let Some(process_info) = execution_context.get_opened_process() {
            let snapshot = execution_context.get_snapshot();
            if let Ok(mut snapshot) = snapshot.write() {
                snapshot.new_scan(&process_info, scan_filter_parameters);
            } else {
                Logger::log(LogLevel::Error, "Failed to create new snapshot!", None);
            }
        }

        ScanNewResponse {}
    }

    fn to_engine_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::New {
            scan_new_request: self.clone(),
        })
    }
}

impl From<ScanNewResponse> for ScanResponse {
    fn from(scan_new_response: ScanNewResponse) -> Self {
        ScanResponse::New { scan_new_response }
    }
}
