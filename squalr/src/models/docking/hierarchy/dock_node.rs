use crate::models::docking::hierarchy::dock_split_child::DockSplitChild;
use crate::models::docking::hierarchy::dock_split_direction::DockSplitDirection;
use serde::{Deserialize, Serialize};

/// The main enum that models our docking hierarchy.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum DockNode {
    /// A split node containing sub-children.
    Split {
        direction: DockSplitDirection,
        children: Vec<DockSplitChild>,
    },
    /// A tab container, holding multiple children in tabs.
    /// Each child is itself a DockNode, so we can have either Leaf nodes
    /// or even nested splits in a tab if we want to get fancy.
    Tab { tabs: Vec<DockNode>, active_tab_id: String },
    /// A leaf node representing a single panel.
    Leaf { window_identifier: String, is_visible: bool },
}

impl Default for DockNode {
    fn default() -> Self {
        // A default "leaf" node. Your default usage may vary.
        DockNode::Leaf {
            window_identifier: "root".into(),
            is_visible: true,
        }
    }
}

impl DockNode {
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
            DockNode::Leaf { is_visible, .. } => *is_visible,
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
            DockNode::Leaf { is_visible, .. } => {
                *is_visible = is_visible_new;
            }
        }
    }

    pub fn get_leaf_id(&self) -> Option<String> {
        match self {
            DockNode::Leaf { window_identifier, .. } => Some(window_identifier.clone()),
            _ => None,
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

    /// Determines if this node is the specified id, or is a container of a child with the specified id.
    pub fn contains_leaf_id(
        &self,
        target_id: &str,
    ) -> bool {
        match self {
            DockNode::Leaf { window_identifier, .. } => window_identifier == target_id,
            DockNode::Split { children, .. } => children
                .iter()
                .any(|child| child.node.contains_leaf_id(target_id)),
            DockNode::Tab { tabs, .. } => tabs.iter().any(|t| t.contains_leaf_id(target_id)),
        }
    }

    /// Return a mutable reference to the node at the specified path.
    /// Returns `None` if the path is invalid or tries to go beyond a leaf.
    pub fn get_node_from_path_mut(
        &mut self,
        path: &[usize],
    ) -> Option<&mut DockNode> {
        let mut current = self;
        for &idx in path {
            match current {
                DockNode::Split { children, .. } => {
                    if idx >= children.len() {
                        return None;
                    }
                    current = &mut children[idx].node;
                }
                DockNode::Tab { tabs, .. } => {
                    if idx >= tabs.len() {
                        return None;
                    }
                    current = &mut tabs[idx];
                }
                DockNode::Leaf { .. } => {
                    // The path goes deeper than a leaf => invalid
                    return None;
                }
            }
        }
        Some(current)
    }

    /// Return an immutable reference to the node at the specified path. Returns `None` if the path is invalid.
    pub fn get_node_from_path(
        &self,
        path: &[usize],
    ) -> Option<&DockNode> {
        let mut current = self;
        for &idx in path {
            match current {
                DockNode::Split { children, .. } => {
                    if idx >= children.len() {
                        return None;
                    }
                    current = &children[idx].node;
                }
                DockNode::Tab { tabs, .. } => {
                    if idx >= tabs.len() {
                        return None;
                    }
                    current = &tabs[idx];
                }
                DockNode::Leaf { .. } => {
                    // The path goes deeper than a leaf => invalid
                    return None;
                }
            }
        }
        Some(current)
    }

    /// Retrieve a mutable reference to a child node of a split or tab by index.
    pub fn get_mut_child<'a>(
        parent_node: &'a mut DockNode,
        child_index: usize,
    ) -> Option<&'a mut DockNode> {
        match parent_node {
            DockNode::Split { children, .. } => {
                if child_index < children.len() {
                    Some(&mut children[child_index].node)
                } else {
                    None
                }
            }
            DockNode::Tab { tabs, .. } => {
                if child_index < tabs.len() {
                    Some(&mut tabs[child_index])
                } else {
                    None
                }
            }
            DockNode::Leaf { .. } => None,
        }
    }
}
