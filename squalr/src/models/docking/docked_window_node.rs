use crate::models::docking::dock_split_direction::DockSplitDirection;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct DockedWindowNode {
    pub window_identifier: String,
    pub direction: DockSplitDirection,
    pub is_visible: bool,
    pub ratio: f32,
    pub children: Vec<DockedWindowNode>,
}

impl Default for DockedWindowNode {
    fn default() -> Self {
        Self {
            window_identifier: String::from("root"),
            direction: DockSplitDirection::Horizontal,
            is_visible: true,
            ratio: 1.0,
            children: Vec::new(),
        }
    }
}
