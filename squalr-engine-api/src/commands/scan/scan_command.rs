use crate::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use crate::commands::scan::hybrid::scan_hybrid_request::ScanHybridRequest;
use crate::commands::scan::manual::scan_manual_request::ScanManualRequest;
use crate::commands::scan::new::scan_new_request::ScanNewRequest;
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
