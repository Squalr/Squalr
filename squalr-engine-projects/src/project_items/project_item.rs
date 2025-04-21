#[typetag::serde(tag = "kind")]
pub trait ProjectItem {
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
    fn is_activated(&self) -> bool;
    fn toggle_activated(&mut self);
    fn set_activated(
        &mut self,
        is_activated: bool,
    );
}
