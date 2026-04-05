use crate::events::plugins::plugins_event::PluginsEvent;
use crate::events::process::process_event::ProcessEvent;
use crate::events::project::project_event::ProjectEvent;
use crate::events::project_items::project_items_event::ProjectItemsEvent;
use crate::events::registry::registry_event::RegistryEvent;
use crate::events::scan_results::scan_results_event::ScanResultsEvent;
use crate::events::trackable_task::trackable_task_event::TrackableTaskEvent;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineEvent {
    Plugins(PluginsEvent),
    Process(ProcessEvent),
    Project(ProjectEvent),
    ProjectItems(ProjectItemsEvent),
    Registry(RegistryEvent),
    TrackableTask(TrackableTaskEvent),
    ScanResults(ScanResultsEvent),
}

pub trait EngineEventRequest: Clone + Serialize + DeserializeOwned {
    fn to_engine_event(&self) -> EngineEvent;
}
