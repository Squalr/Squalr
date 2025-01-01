use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub enum DockSplitDirection {
    Horizontal,
    Vertical,
}

impl Default for DockSplitDirection {
    fn default() -> Self {
        DockSplitDirection::Horizontal
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DockedWindowNode {
    pub window_identifier: String,
    pub direction: DockSplitDirection,
    pub ratio: f32,
    pub children: Vec<DockedWindowNode>,
}

impl Default for DockedWindowNode {
    fn default() -> Self {
        Self {
            window_identifier: String::from("root"),
            direction: DockSplitDirection::Horizontal,
            ratio: 1.0,
            children: Vec::new(),
        }
    }
}
