use crate::commands::command_handler::CommandHandler;
use crate::commands::scan::requests::scan_hybrid_request::ScanHybridRequest;
use crate::commands::scan::requests::scan_manual_request::ScanManualRequest;
use crate::commands::scan::requests::scan_new_request::ScanNewRequest;
use crate::commands::scan::requests::scan_value_collector_request::ScanValueCollectorRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ScanCommand {
    /// Collect values for the current scan if one exist, otherwise collect initial values.
    CollectValues {
        #[structopt(flatten)]
        scan_value_collector_request: ScanValueCollectorRequest,
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

impl CommandHandler for ScanCommand {
    fn handle(
        &self,
        uuid: Uuid,
    ) {
        match self {
            ScanCommand::CollectValues { scan_value_collector_request } => {
                scan_value_collector_request.handle(uuid);
            }
            ScanCommand::Hybrid { scan_hybrid_request } => {
                scan_hybrid_request.handle(uuid);
            }
            ScanCommand::New { scan_new_request } => {
                scan_new_request.handle(uuid);
            }
            ScanCommand::Manual { scan_manual_request } => {
                scan_manual_request.handle(uuid);
            }
        }
    }
}
