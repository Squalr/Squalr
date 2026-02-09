use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::registries::registry_context::RegistryContext;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use crate::structures::projects::project_items::project_item_type::ProjectItemType;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypePointer {}

impl ProjectItemTypePointer {
    pub const PROJECT_ITEM_TYPE_ID: &str = "pointer";
}

impl ProjectItemType for ProjectItemTypePointer {
    fn get_project_item_type_id(&self) -> &str {
        &Self::PROJECT_ITEM_TYPE_ID
    }

    fn on_activated_changed(
        &self,
        _engine_bindings: &Arc<RwLock<dyn EngineApiPrivilegedBindings>>,
        _registry_context: &dyn RegistryContext,
        _project_item: &ProjectItemRef,
    ) {
        // JIRA: Implement
    }

    fn tick(
        &self,
        _engine_bindings: &dyn EngineApiPrivilegedBindings,
        _opened_process: &Option<OpenedProcessInfo>,
        _registry_context: &dyn RegistryContext,
        _project_item: &ProjectItemRef,
    ) {
    }
}

impl ProjectItemTypePointer {
    // JIRA: Make properties for these.
    /*
    module_name: String,
    module_offset: u64,
    pointer_offsets: Vec<i32>,
    */
}
