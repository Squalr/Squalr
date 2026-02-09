use crate::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use crate::commands::scan::element_scan::element_scan_request::ElementScanRequest;
use crate::commands::scan::new::scan_new_request::ScanNewRequest;
use crate::commands::scan::reset::scan_reset_request::ScanResetRequest;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub enum ScanCommand {
    /// Clears the current scan.
    Reset {
        #[structopt(flatten)]
        scan_reset_request: ScanResetRequest,
    },
    /// Starts a new scan with the provided data types and alignments. This does not collect any values,
    /// it only queries virtual memory for the virtual page address ranges for later scans.
    New {
        #[structopt(flatten)]
        scan_new_request: ScanNewRequest,
    },
    /// Collect values for the current scan if one exist, otherwise collect initial values.
    CollectValues {
        #[structopt(flatten)]
        scan_value_collector_request: ScanCollectValuesRequest,
    },
    /// Performs an element-wise scan, potentially collecting values depending on the provided parameters.
    ElementScan {
        #[structopt(flatten)]
        element_scan_request: ElementScanRequest,
    },
}
