use crate::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use crate::commands::scan::element_scan::element_scan_response::ElementScanResponse;
use crate::commands::scan::new::scan_new_response::ScanNewResponse;
use crate::commands::scan::reset::scan_reset_response::ScanResetResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanResponse {
    New { scan_new_response: ScanNewResponse },
    Reset { scan_reset_response: ScanResetResponse },
    CollectValues { scan_value_collector_response: ScanCollectValuesResponse },
    ElementScan { element_scan_response: ElementScanResponse },
}

impl ScanResponse {
    pub fn failure_message(&self) -> Option<&str> {
        match self {
            Self::New { scan_new_response: reply } if !reply.success => Some("The command failed."),
            Self::Reset { scan_reset_response: reply } if !reply.success => Some("The command failed."),
            Self::CollectValues {
                scan_value_collector_response: reply,
            } if !reply.success => Some("The command failed."),
            Self::ElementScan { element_scan_response: reply } if !reply.success => Some("The command failed."),
            _ => None,
        }
    }
}
