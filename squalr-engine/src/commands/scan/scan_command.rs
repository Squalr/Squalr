use crate::commands::engine_response::TypedEngineResponse;
use crate::commands::scan::hybrid::scan_hybrid_request::ScanHybridRequest;
use crate::commands::scan::manual::scan_manual_request::ScanManualRequest;
use crate::commands::scan::new::scan_new_request::ScanNewRequest;
use crate::commands::scan::scan_request::ScanRequest;
use crate::commands::{engine_response::EngineResponse, scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ScanCommand {
    /// Collect values for the current scan if one exist, otherwise collect initial values.
    CollectValues {
        #[structopt(flatten)]
        scan_value_collector_request: ScanCollectValuesRequest,
    },
    /// Collect values and scan in the same parallel thread pool.
    Hybrid {
        #[structopt(flatten)]
        scan_hybrid_request: ScanHybridRequest,
    },
    /// Starts a new scan with the provided data types / alignments
    New {
        #[structopt(flatten)]
        scan_new_request: ScanNewRequest,
    },
    /// Standard scan that operates on existing collected values.
    Manual {
        #[structopt(flatten)]
        scan_manual_request: ScanManualRequest,
    },
}

impl ScanCommand {
    pub fn execute(&self) -> EngineResponse {
        match self {
            ScanCommand::CollectValues { scan_value_collector_request } => scan_value_collector_request.execute().to_engine_response(),
            ScanCommand::Hybrid { scan_hybrid_request } => scan_hybrid_request.execute().to_engine_response(),
            ScanCommand::New { scan_new_request } => scan_new_request.execute().to_engine_response(),
            ScanCommand::Manual { scan_manual_request } => scan_manual_request.execute().to_engine_response(),
        }
    }
}
