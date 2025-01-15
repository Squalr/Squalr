use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq)]
pub enum DockSplitDirection {
    HorizontalDivider,
    VerticalDivider,
}

impl Default for DockSplitDirection {
    fn default() -> Self {
        DockSplitDirection::HorizontalDivider
    }
}
