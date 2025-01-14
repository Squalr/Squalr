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
    /// Each child is itself a DockNode, so we can have either Leaf nodes
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

    pub fn walk<'a, F>(
        &'a self,
        path: &mut Vec<usize>,
        visitor: &mut F,
    ) where
        F: FnMut(&'a DockNode, &[usize]),
    {
        // Visit the current node.
        visitor(self, path);

        // Recurse into children, if any.
        match self {
            DockNode::Split { children, .. } => {
                for (i, child) in children.iter().enumerate() {
                    path.push(i);
                    child.walk(path, visitor);
                    path.pop();
                }
            }
            DockNode::Tab { tabs, .. } => {
                for (i, tab) in tabs.iter().enumerate() {
                    path.push(i);
                    tab.walk(path, visitor);
                    path.pop();
                }
            }
            DockNode::Leaf { .. } => {}
        }
    }

    pub fn walk_with_layout<F>(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        visitor: &mut F,
    ) where
        F: FnMut(&DockNode, (f32, f32, f32, f32)),
    {
        // Invoke callback with the bounds of this node.
        visitor(self, (x, y, width, height));

        // Now figure out how to split/allocate that rect to children.
        match self {
            DockNode::Split { direction, children, .. } => {
                // Filter out invisible children.
                let visible_children: Vec<&DockNode> = children.iter().filter(|c| c.is_visible()).collect();

                if visible_children.is_empty() {
                    return;
                }

                let total_ratio: f32 = visible_children.iter().map(|c| c.get_ratio()).sum();
                let mut offset = 0.0;
                let num_children = visible_children.len();

                for child in visible_children {
                    // Normalize ratio.
                    let child_ratio = if total_ratio > 0.0 {
                        child.get_ratio() / total_ratio
                    } else {
                        1.0 / num_children as f32
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
                    child.walk_with_layout(cx, cy, cw, ch, visitor);

                    // Accumulate offset
                    match direction {
                        DockSplitDirection::Horizontal => offset += cw,
                        DockSplitDirection::Vertical => offset += ch,
                    }
                }
            }

            DockNode::Tab { tabs, .. } => {
                // Each tab gets the entire rectangle.
                for tab_node in tabs {
                    if tab_node.is_visible() {
                        tab_node.walk_with_layout(x, y, width, height, visitor);
                    }
                }
            }
            // Leaves have no children, so nothing further to do.
            DockNode::Leaf { .. } => {}
        }
    }

    /// Collect the IDs of all leaf nodes in this sub-tree.
    pub fn collect_leaves(
        &self,
        out: &mut Vec<String>,
    ) {
        let mut path = Vec::new();

        self.walk(&mut path, &mut |node, _current_path| {
            if let DockNode::Leaf { window_identifier, .. } = node {
                out.push(window_identifier.clone());
            }
        });
    }

    /// Return a path of indices that leads to the panel matching the given identifier.
    /// Example path: [2, 0] means: in this node's children[2].children[0] or tabs[2].tabs[0].
    /// Returns None if not found in this subtree.
    pub fn find_path_to_leaf(
        &self,
        window_id: &str,
    ) -> Option<Vec<usize>> {
        // Instead of manual recursion, we can do a single pass with walk.
        let mut path_stack = Vec::new();
        let mut result = None;

        self.walk(&mut path_stack, &mut |node, current_path| {
            if let DockNode::Leaf { window_identifier, .. } = node {
                if window_identifier == window_id {
                    // Found it! Capture the current path (copy it).
                    result = Some(current_path.to_vec());
                }
            }
        });

        result
    }

    /// Find the bounding rectangle of a child leaf node by ID (if it exists in this sub-tree).
    /// Returns (x, y, width, height).
    pub fn find_window_rect(
        &self,
        target_id: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        let mut found_rect = None;

        // Single pass with walk_with_layout.
        self.walk_with_layout(x, y, width, height, &mut |node, (cx, cy, cw, ch)| {
            if let DockNode::Leaf {
                window_identifier, is_visible, ..
            } = node
            {
                if *is_visible && window_identifier == target_id {
                    // Found it: store rectangle.
                    found_rect = Some((cx, cy, cw, ch));
                }
            }
        });

        found_rect
    }
}
