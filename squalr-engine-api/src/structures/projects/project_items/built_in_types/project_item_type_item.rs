use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::registries::registry_context::RegistryContext;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use crate::structures::projects::project_items::project_item_target::ProjectItemTarget;
use crate::structures::projects::project_items::project_item_type_ref::ProjectItemTypeRef;
use crate::structures::projects::project_items::{project_item::ProjectItem, project_item_type::ProjectItemType};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypeItem {}

impl ProjectItemTypeItem {
    pub const PROJECT_ITEM_TYPE_ID: &str = "item";
    pub const DEFAULT_PROJECT_ITEM_NAME: &str = "New Project Item";

    pub fn new_project_item(name: &str) -> ProjectItem {
        let project_item_type_ref = ProjectItemTypeRef::new(Self::PROJECT_ITEM_TYPE_ID.to_string());
        let project_item_name = if name.trim().is_empty() { Self::DEFAULT_PROJECT_ITEM_NAME } else { name };
        let mut project_item = ProjectItem::new(project_item_type_ref, project_item_name);

        project_item.set_target(ProjectItemTarget::new_address(0, String::new()));

        project_item
    }
}

impl ProjectItemType for ProjectItemTypeItem {
    fn get_project_item_type_id(&self) -> &str {
        Self::PROJECT_ITEM_TYPE_ID
    }

    fn on_activated_changed(
        &self,
        _engine_bindings: &Arc<RwLock<dyn EngineApiPrivilegedBindings>>,
        _registry_context: &dyn RegistryContext,
        _project_item_ref: &ProjectItemRef,
    ) {
    }

    fn tick(
        &self,
        _engine_bindings: &dyn EngineApiPrivilegedBindings,
        _opened_process: &Option<OpenedProcessInfo>,
        _registry_context: &dyn RegistryContext,
        _project_item_ref: &ProjectItemRef,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemTypeItem;
    use crate::structures::projects::project_items::project_item_target::ProjectItemTarget;

    #[test]
    fn new_project_item_defaults_to_address_target() {
        let project_item = ProjectItemTypeItem::new_project_item("Watch");

        assert_eq!(project_item.get_target(), &ProjectItemTarget::new_address(0, String::new()));
    }
}
