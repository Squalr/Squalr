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

impl DockNode {
    /// Check if a node is visible.
    pub fn is_visible(&self) -> bool {
        match self {
            DockNode::Split { .. } => true,
            DockNode::Tab { .. } => true,
            DockNode::Leaf { is_visible, .. } => *is_visible,
        }
    }

    /// Set the visibility of a node.
    pub fn set_visible(
        &mut self,
        is_visible_new: bool,
    ) {
        match self {
            DockNode::Split { .. } => log::warn!("Cannot set split ratio on a split container!"),
            DockNode::Tab { .. } => log::warn!("Cannot set split ratio on a tab node!"),
            DockNode::Leaf { is_visible, .. } => *is_visible = is_visible_new,
        }
    }

    /// Get ratio of a node.
    pub fn get_ratio(&self) -> f32 {
        match self {
            DockNode::Split { ratio, .. } => *ratio,
            DockNode::Tab { ratio, .. } => *ratio,
            DockNode::Leaf { ratio, .. } => *ratio,
        }
    }

    /// Set ratio of a node.
    pub fn set_ratio(
        &mut self,
        new_ratio: f32,
    ) {
        match self {
            DockNode::Split { ratio, .. } => *ratio = new_ratio,
            DockNode::Tab { ratio, .. } => *ratio = new_ratio,
            DockNode::Leaf { ratio, .. } => *ratio = new_ratio,
        }
    }

    /// Check if this node is a leaf with a specific ID.
    pub fn is_leaf_with_id(
        &self,
        target_id: &str,
    ) -> bool {
        match self {
            DockNode::Leaf { window_identifier, .. } => window_identifier == target_id,
            _ => false,
        }
    }

    /// Collect the IDs of all leaf nodes in this sub-tree.
    pub fn collect_leaves(
        &self,
        out: &mut Vec<String>,
    ) {
        match self {
            DockNode::Leaf { window_identifier, .. } => {
                out.push(window_identifier.clone());
            }
            DockNode::Split { children, .. } => {
                for child in children {
                    child.collect_leaves(out);
                }
            }
            DockNode::Tab { tabs, .. } => {
                for tab in tabs {
                    tab.collect_leaves(out);
                }
            }
        }
    }

    /// Return a path of indices that leads to the leaf node matching `window_id`.
    /// Example path: [2, 0] means: in this node's `children[2].children[0]` or `tabs[2].tabs[0]`.
    /// Returns `None` if not found in this subtree.
    pub fn find_path_to_leaf(
        &self,
        window_id: &str,
    ) -> Option<Vec<usize>> {
        match self {
            DockNode::Leaf { window_identifier, .. } => {
                if window_identifier == window_id {
                    // Found it! Return an empty path meaning "we are the node."
                    Some(vec![])
                } else {
                    None
                }
            }
            DockNode::Split { children, .. } => {
                for (i, child) in children.iter().enumerate() {
                    if let Some(mut path) = child.find_path_to_leaf(window_id) {
                        path.insert(0, i);
                        return Some(path);
                    }
                }
                None
            }
            DockNode::Tab { tabs, .. } => {
                for (i, tab) in tabs.iter().enumerate() {
                    if let Some(mut path) = tab.find_path_to_leaf(window_id) {
                        path.insert(0, i);
                        return Some(path);
                    }
                }
                None
            }
        }
    }

    /// Find the bounding rectangle of a child leaf node by ID (if it exists in this sub-tree).
    /// Returns `(x, y, width, height)`.
    pub fn find_window_rect(
        &self,
        target_id: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        match self {
            DockNode::Leaf {
                window_identifier, is_visible, ..
            } => {
                if *is_visible && window_identifier == target_id {
                    Some((x, y, width, height))
                } else {
                    None
                }
            }
            DockNode::Split { direction, children, .. } => {
                let visible_children: Vec<&DockNode> = children.iter().filter(|c| c.is_visible()).collect();
                if visible_children.is_empty() {
                    return None;
                }

                // Sum ratios for normalization
                let total_ratio: f32 = visible_children.iter().map(|c| c.get_ratio()).sum();
                let mut offset = 0.0;
                let visible_len = visible_children.len();

                for child in visible_children {
                    let child_ratio = if total_ratio > 0.0 {
                        child.get_ratio() / total_ratio
                    } else {
                        1.0 / visible_len as f32
                    };

                    let (cw, ch) = match direction {
                        DockSplitDirection::Horizontal => (width * child_ratio, height),
                        DockSplitDirection::Vertical => (width, height * child_ratio),
                    };

                    let (cx, cy) = match direction {
                        DockSplitDirection::Horizontal => (x + offset, y),
                        DockSplitDirection::Vertical => (x, y + offset),
                    };

                    // Recurse
                    if let Some(rect) = child.find_window_rect(target_id, cx, cy, cw, ch) {
                        return Some(rect);
                    }

                    match direction {
                        DockSplitDirection::Horizontal => offset += cw,
                        DockSplitDirection::Vertical => offset += ch,
                    }
                }
                None
            }
            DockNode::Tab { tabs, .. } => {
                // Typically only one tab is "active" (visible),
                // but for simplicity, we just check them all.
                for child in tabs {
                    if let Some(rect) = child.find_window_rect(target_id, x, y, width, height) {
                        return Some(rect);
                    }
                }
                None
            }
        }
    }
}
