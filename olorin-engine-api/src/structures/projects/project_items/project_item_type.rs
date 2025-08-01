use crate::engine::engine_execution_context::EngineExecutionContext;
use crate::registries::project_item_types::project_item_type_registry::ProjectItemTypeRegistry;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::project_item::ProjectItem;
use std::sync::Arc;

pub trait ProjectItemType: Send + Sync {
    fn get_project_item_type_id(&self) -> &str;
    fn on_activated_changed(
        &self,
        project_item: &ProjectItem,
    );
    fn tick(
        &self,
        engine_execution_context: &Arc<EngineExecutionContext>,
        opened_process: &Option<OpenedProcessInfo>,
        project_item_type_registry: &ProjectItemTypeRegistry,
        project_item: &mut ProjectItem,
    );
}
