use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::registries::project_item_types::project_item_type_registry::ProjectItemTypeRegistry;
use crate::registries::registries::Registries;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::project_item::ProjectItem;

pub trait ProjectItemType: Send + Sync {
    fn get_project_item_type_id(&self) -> &str;
    fn on_activated_changed(
        &self,
        project_item_type_registry: &ProjectItemTypeRegistry,
        project_item: &mut ProjectItem,
    );
    fn tick(
        &self,
        engine_bindings: &dyn EngineApiPrivilegedBindings,
        opened_process: &Option<OpenedProcessInfo>,
        registries: &Registries,
        project_item: &mut ProjectItem,
    );
}
