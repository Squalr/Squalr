use crate::commands::engine_response::EngineResponse;
use crate::commands::scan::responses::scan_new_response::ScanNewResponse;
use crate::commands::scan::scan_response::ScanResponse;
use crate::{
    commands::{command_handler::CommandHandler, engine_command::EngineCommand, request_sender::RequestSender, scan::scan_command::ScanCommand},
    squalr_engine::SqualrEngine,
    squalr_session::SqualrSession,
};
use serde::{Deserialize, Serialize};
use squalr_engine_common::values::{data_type::DataType, endian::Endian};
use squalr_engine_scanning::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct ScanNewRequest {
    #[structopt(short = "d", long, use_delimiter = true)]
    pub scan_filter_parameters: Vec<ScanFilterParameters>,
    #[structopt(short = "a", long)]
    pub scan_all_primitives: bool,
}

impl CommandHandler for ScanNewRequest {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
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

        if let Some(process_info) = SqualrSession::get_opened_process() {
            let snapshot = SqualrSession::get_snapshot();
            let mut snapshot = snapshot.write().unwrap();

            snapshot.new_scan(&process_info, scan_filter_parameters);
        }
    }
}

impl RequestSender for ScanNewRequest {
    type ResponseType = ScanNewResponse;

    fn send<F>(
        &self,
        callback: F,
    ) where
        F: FnOnce(Self::ResponseType) + Send + Sync + 'static,
    {
        SqualrEngine::dispatch_command(self.to_command(), move |engine_response| match engine_response {
            EngineResponse::Scan(process_response) => match process_response {
                ScanResponse::New { scan_new_response } => callback(scan_new_response),
                _ => {}
            },
            _ => {}
        });
    }

    fn to_command(&self) -> EngineCommand {
        EngineCommand::Scan(ScanCommand::New {
            scan_new_request: self.clone(),
        })
    }
}
