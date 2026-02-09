use crate::registries::registry_context::RegistryContext;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::{engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings, structures::projects::project_items::project_item_ref::ProjectItemRef};
use std::sync::{Arc, RwLock};

pub trait ProjectItemType: Send + Sync {
    fn get_project_item_type_id(&self) -> &str;
    fn on_activated_changed(
        &self,
        engine_bindings: &Arc<RwLock<dyn EngineApiPrivilegedBindings>>,
        registry_context: &dyn RegistryContext,
        project_item_ref: &ProjectItemRef,
    );
    fn tick(
        &self,
        engine_bindings: &dyn EngineApiPrivilegedBindings,
        opened_process: &Option<OpenedProcessInfo>,
        registry_context: &dyn RegistryContext,
        project_item_ref: &ProjectItemRef,
    );
}
