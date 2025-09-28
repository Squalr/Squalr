use crate::models::docking::hierarchy::types::dock_split_child::DockSplitChild;
use crate::models::docking::hierarchy::types::dock_split_direction::DockSplitDirection;
use serde::{Deserialize, Serialize};

/// The main enum that models our docking hierarchy.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum DockNode {
    /// A split container, holding multiple children side-by-side vertically or horizontally.
    Split {
        direction: DockSplitDirection,
        children: Vec<DockSplitChild>,
    },
    /// A tab container, holding multiple children in tabs.
    Tab { tabs: Vec<DockNode>, active_tab_id: String },
    /// A window node representing a single window.
    Window { window_identifier: String, is_visible: bool },
}

impl Default for DockNode {
    fn default() -> Self {
        // A default "window" node. Your default usage may vary.
        DockNode::Window {
            window_identifier: "root".into(),
            is_visible: true,
        }
    }
}

impl DockNode {
    /// Gets the `window_id` of this dock node, if this dock node represents a window type.
    pub fn get_window_id(&self) -> Option<String> {
        match self {
            DockNode::Window { window_identifier, .. } => Some(window_identifier.clone()),
            _ => None,
        }
    }

    /// Check if a node is visible.
    pub fn is_visible(&self) -> bool {
        match self {
            DockNode::Split { children, .. } => {
                // A split node is visible if any of its children is visible.
                children.iter().any(|child| child.node.is_visible())
            }
            DockNode::Tab { tabs, .. } => {
                // A tab node is visible if at least one of its tabs is visible.
                tabs.iter().any(|tab| tab.is_visible())
            }
            DockNode::Window { is_visible, .. } => *is_visible,
        }
    }

    /// Set the visibility of a node (only applicable to leaves in this minimal approach).
    pub fn set_visible(
        &mut self,
        is_visible_new: bool,
    ) {
        match self {
            DockNode::Split { .. } => {
                log::warn!("Cannot directly set visibility on a split node!");
            }
            DockNode::Tab { .. } => {
                log::warn!("Cannot directly set visibility on a tab node!");
            }
            DockNode::Window { is_visible, .. } => {
                *is_visible = is_visible_new;
            }
        }
    }
}
