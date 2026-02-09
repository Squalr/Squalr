use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::registries::registry_context::RegistryContext;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use crate::structures::projects::project_items::project_item_type_ref::ProjectItemTypeRef;
use crate::structures::projects::project_items::{project_item::ProjectItem, project_item_type::ProjectItemType};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypeDirectory {}

impl ProjectItemTypeDirectory {
    pub const PROJECT_ITEM_TYPE_ID: &str = "directory";
}

impl ProjectItemType for ProjectItemTypeDirectory {
    fn get_project_item_type_id(&self) -> &str {
        &Self::PROJECT_ITEM_TYPE_ID
    }

    fn on_activated_changed(
        &self,
        _engine_bindings: &Arc<RwLock<dyn EngineApiPrivilegedBindings>>,
        _registry_context: &dyn RegistryContext,
        _project_item_ref: &ProjectItemRef,
    ) {
        /*
        let is_activated = project_item.get_is_activated();

        // Recurse the tick call to all child project items.
        for child in project_item.get_children_mut() {
            child.set_activated(engine_bindings, registries, is_activated);
        }*/
    }

    fn tick(
        &self,
        _engine_bindings: &dyn EngineApiPrivilegedBindings,
        _opened_process: &Option<OpenedProcessInfo>,
        registry_context: &dyn RegistryContext,
        _project_item_ref: &ProjectItemRef,
    ) {
        if let Ok(_project_item_type_registry) = registry_context.get_project_item_type_registry().read() {
            // Recurse the tick call to all child project items.
            /*
            for child in project_item.get_children_mut() {
                if let Some(project_item_type) = project_item_type_registry.get(child.get_item_type().get_project_item_type_id()) {
                    project_item_type.tick(engine_bindings, opened_process, registries, child);
                }
            }*/
        }
    }
}

impl ProjectItemTypeDirectory {
    pub fn new_project_item(project_item_ref: &ProjectItemRef) -> ProjectItem {
        let project_item_type_ref = ProjectItemTypeRef::new(Self::PROJECT_ITEM_TYPE_ID.to_string());

        ProjectItem::new(project_item_type_ref, &project_item_ref.get_file_or_directory_name())
    }
}
