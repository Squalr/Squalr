use std::any::Any;

#[typetag::serde(tag = "kind")]
pub trait ProjectItemType: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
    fn is_activated(&self) -> bool;
    fn toggle_activated(&mut self);
    fn set_activated(
        &mut self,
        is_activated: bool,
    );
    fn get_has_unsaved_changes(&self) -> bool;
    fn set_has_unsaved_changes(
        &mut self,
        has_unsaved_changes: bool,
    );
}
