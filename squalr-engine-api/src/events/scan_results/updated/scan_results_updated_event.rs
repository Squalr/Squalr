use crate::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    scan_results::scan_results_event::ScanResultsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResultsUpdatedEvent {}

impl EngineEventRequest for ScanResultsUpdatedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::ScanResults(ScanResultsEvent::ScanResultsUpdated {
            scan_results_updated_event: self.clone(),
        })
    }
}
