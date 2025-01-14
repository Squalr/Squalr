use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum DockSplitDirection {
    Horizontal,
    Vertical,
}

impl Default for DockSplitDirection {
    fn default() -> Self {
        DockSplitDirection::Horizontal
    }
}
