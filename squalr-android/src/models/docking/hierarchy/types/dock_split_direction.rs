use serde::{Deserialize, Serialize};

/// Represents the two possible split orientations for a docking split container.
#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq)]
pub enum DockSplitDirection {
    /// A divider that splits a container with a horizontal divider, resulting in a vertical layout.
    HorizontalDivider,
    /// A divider that splits a container with a vertical divider, resulting in a horizontal layout.
    VerticalDivider,
}

impl Default for DockSplitDirection {
    fn default() -> Self {
        DockSplitDirection::HorizontalDivider
    }
}
