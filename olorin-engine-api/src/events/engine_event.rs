use crate::events::process::process_event::ProcessEvent;
use crate::events::project::project_event::ProjectEvent;
use crate::events::scan_results::scan_results_event::ScanResultsEvent;
use crate::events::trackable_task::trackable_task_event::TrackableTaskEvent;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineEvent {
    Process(ProcessEvent),
    Project(ProjectEvent),
    TrackableTask(TrackableTaskEvent),
    ScanResults(ScanResultsEvent),
}

pub trait EngineEventRequest: Clone + Serialize + DeserializeOwned {
    fn to_engine_event(&self) -> EngineEvent;
}
