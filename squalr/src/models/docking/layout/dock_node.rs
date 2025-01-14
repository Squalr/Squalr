use crate::models::docking::layout::dock_split_direction::DockSplitDirection;
use serde::{Deserialize, Serialize};

/// The main enum that models our docking hierarchy.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum DockNode {
    /// A split node containing sub-children.
    Split {
        /// The ratio that the entire container occupies.
        ratio: f32,
        direction: DockSplitDirection,
        children: Vec<DockNode>,
    },
    /// A tab container, holding multiple children in tabs.
    /// Each child is itself a `DockNode`, so we can have either Leaf nodes
    /// or even nested splits in a tab if we want to get fancy.
    Tab {
        /// The ratio that the entire container occupies.
        ratio: f32,
        tabs: Vec<DockNode>,
        active_tab_id: String,
    },
    /// A leaf node representing a single panel.
    Leaf { window_identifier: String, is_visible: bool, ratio: f32 },
}

impl Default for DockNode {
    fn default() -> Self {
        // A default "leaf" node. Your default usage may vary.
        DockNode::Leaf {
            ratio: 1.0,
            window_identifier: "root".into(),
            is_visible: true,
        }
    }
}
