use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::engine::engine_execution_context::EngineExecutionContext;
use crate::registries::project_item_types::project_item_type_registry::ProjectItemTypeRegistry;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::project_item_type_ref::ProjectItemTypeRef;
use crate::structures::projects::project_items::{project_item::ProjectItem, project_item_type::ProjectItemType};
use serde::{Deserialize, Serialize};
use std::path::Path;

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
        project_item: &ProjectItem,
    ) {
        // JIRA: Implement
    }

    fn tick(
        &self,
        engine_bindings: &dyn EngineApiPrivilegedBindings,
        opened_process: &Option<OpenedProcessInfo>,
        project_item_type_registry: &ProjectItemTypeRegistry,
        project_item: &mut ProjectItem,
    ) {
        // Recurse the tick call to all child project items.
        for child in project_item.get_children_mut() {
            if let Some(project_item_type) = project_item_type_registry.get(child.get_item_type().get_project_item_type_id()) {
                project_item_type.tick(engine_bindings, opened_process, project_item_type_registry, child);
            }
        }
    }
}

impl ProjectItemTypeDirectory {
    pub fn new_project_item(directory: &Path) -> ProjectItem {
        let directory_type = ProjectItemTypeRef::new(Self::PROJECT_ITEM_TYPE_ID.to_string());

        ProjectItem::new(directory.to_path_buf(), directory_type, true)
    }
}
