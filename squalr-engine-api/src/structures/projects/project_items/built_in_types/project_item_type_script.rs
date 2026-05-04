use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::registries::registry_context::RegistryContext;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::project_item::ProjectItem;
use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use crate::structures::projects::project_items::project_item_type::ProjectItemType;
use crate::structures::projects::project_items::project_item_type_ref::ProjectItemTypeRef;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypeScript {}

impl ProjectItemTypeScript {
    pub const PROJECT_ITEM_TYPE_ID: &str = "script";
    pub const DEFAULT_PROJECT_ITEM_NAME: &str = "New Script";

    pub fn new_project_item(name: &str) -> ProjectItem {
        let project_item_type_ref = ProjectItemTypeRef::new(Self::PROJECT_ITEM_TYPE_ID.to_string());
        let project_item_name = if name.trim().is_empty() { Self::DEFAULT_PROJECT_ITEM_NAME } else { name };

        ProjectItem::new(project_item_type_ref, project_item_name)
    }
}

impl ProjectItemType for ProjectItemTypeScript {
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
    use super::ProjectItemTypeScript;

    #[test]
    fn new_project_item_uses_script_type() {
        let project_item = ProjectItemTypeScript::new_project_item("");

        assert_eq!(project_item.get_field_name(), ProjectItemTypeScript::DEFAULT_PROJECT_ITEM_NAME);
        assert_eq!(
            project_item.get_item_type().get_project_item_type_id(),
            ProjectItemTypeScript::PROJECT_ITEM_TYPE_ID
        );
    }
}
