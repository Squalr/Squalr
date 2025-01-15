use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum DockSplitDirection {
    HorizontalDivider,
    VerticalDivider,
}

impl Default for DockSplitDirection {
    fn default() -> Self {
        DockSplitDirection::HorizontalDivider
    }
}
