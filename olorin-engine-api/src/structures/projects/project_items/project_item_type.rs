use crate::structures::projects::project_items::project_item::ProjectItem;

pub trait ProjectItemType: Send + Sync {
    fn get_project_item_type_id(&self) -> &str;
    fn on_activated_changed(
        &self,
        project_item: &ProjectItem,
    );
    fn tick(
        &mut self,
        project_item: &mut ProjectItem,
    );
}
