use crate::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScanResultsEvent {
    ScanResultsUpdated { scan_results_updated_event: ScanResultsUpdatedEvent },
}
