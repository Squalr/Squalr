use serde::{Deserialize, Serialize};

pub trait ProjectItem {
    fn get_name(&self) -> &'static str;
    fn get_description(&self) -> &'static str;
    fn is_activated(&self) -> bool;
    fn toggle_activated(&self);
    fn set_activated(
        &self,
        is_activated: bool,
    );
}
