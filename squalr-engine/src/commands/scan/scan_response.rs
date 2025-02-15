use crate::commands::scan::collect_values::scan_collect_values_response::ScanCollectValuesResponse;
use crate::commands::scan::hybrid::scan_hybrid_response::ScanHybridResponse;
use crate::commands::scan::manual::scan_manual_response::ScanManualResponse;
use crate::commands::scan::new::scan_new_response::ScanNewResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanResponse {
    New { scan_new_response: ScanNewResponse },
    CollectValues { scan_value_collector_response: ScanCollectValuesResponse },
    Hybrid { scan_hybrid_response: ScanHybridResponse },
    Manual { scan_manual_response: ScanManualResponse },
}
