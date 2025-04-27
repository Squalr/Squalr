pub trait ProjectItemType: Send + Sync {
    fn get_project_item_type_id(&self) -> &str;
}
