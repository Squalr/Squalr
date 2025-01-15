use crate::models::docking::dock_drag_direction::DockDragDirection;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_split_direction::DockSplitDirection;
use serde::{Deserialize, Serialize};

/// A simple tree structure that owns a single root `DockNode` and provides search and update methods.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DockTree {
    pub root: DockNode,
}

impl DockTree {
    /// Create a new DockTree with the given root node.
    pub fn new(root: DockNode) -> Self {
        Self { root }
    }

    pub fn replace_root(
        &mut self,
        new_root: DockNode,
    ) {
        self.root = new_root;
    }

    /// Find the path (series of child indices) to a leaf node by ID. Returns `None` if not found.
    pub fn find_leaf_path(
        &self,
        leaf_id: &str,
    ) -> Option<Vec<usize>> {
        let mut path_stack = Vec::new();
        let mut result = None;

        self.root.walk(&mut path_stack, &mut |node, current_path| {
            if let DockNode::Leaf { window_identifier, .. } = node {
                if window_identifier == leaf_id {
                    result = Some(current_path.to_vec());
                }
            }
        });

        result
    }

    /// Return a mutable reference to the node at the specified path.
    /// Returns `None` if the path is invalid or tries to go beyond a leaf.
    pub fn get_node_mut(
        &mut self,
        path: &[usize],
    ) -> Option<&mut DockNode> {
        let mut current = &mut self.root;
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
    pub fn get_node(
        &self,
        path: &[usize],
    ) -> Option<&DockNode> {
        let mut current = &self.root;
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

    /// Collect the identifiers of all leaf nodes in the entire tree.
    pub fn get_all_leaves(&self) -> Vec<String> {
        let mut leaves = Vec::new();
        let mut path_stack = Vec::new();

        self.root.walk(&mut path_stack, &mut |node, _| {
            if let DockNode::Leaf { window_identifier, .. } = node {
                leaves.push(window_identifier.clone());
            }
        });

        leaves
    }

    /// Find the matching ancestor `DockNode::Split` that should be resized when dragging a particular edge of a leaf.
    pub fn find_ancestor_split_for_drag(
        &self,
        leaf_path: &[usize],
        drag_dir: &DockDragDirection,
    ) -> Option<Vec<usize>> {
        if leaf_path.is_empty() {
            return None;
        }

        // We climb up. The last element is the leaf's index. We'll remove it from the path, leaving us the parent's path.
        let mut path = leaf_path.to_vec();
        let mut child_index = path.pop().unwrap(); // index in parent's children/tabs.

        loop {
            // We have the path that points to the parent. Let's see if that node is a Split with the correct orientation.
            //    - But first, check if we can retrieve it.
            let candidate_node = self.get_node(&path)?;

            // If candidate_node is a Split, see if it matches the drag direction and if the child_index is on the correct side.
            if let DockNode::Split { direction, .. } = candidate_node {
                // If orientation and side match up, we can return it right away.
                if Self::matches_drag_side(direction, drag_dir, child_index) {
                    return Some(path.clone());
                }
            }

            // If that node wasn't a matching Split, we pop up further.
            if path.is_empty() {
                // We’ve reached the root. There's nothing above root, so we can’t climb further.
                return None;
            }
            child_index = path.pop().unwrap();
        }
    }

    fn matches_drag_side(
        direction: &DockSplitDirection,
        drag_dir: &DockDragDirection,
        child_index: usize,
    ) -> bool {
        match (drag_dir, direction) {
            // If we're dragging the right edge, it’s the left child in a vertical split.
            (DockDragDirection::Right, DockSplitDirection::VerticalDivider) => child_index == 0,
            (DockDragDirection::Left, DockSplitDirection::VerticalDivider) => child_index == 1,

            // If we're dragging the bottom edge, it’s the top child in a horizontal split.
            (DockDragDirection::Bottom, DockSplitDirection::HorizontalDivider) => child_index == 0,
            (DockDragDirection::Top, DockSplitDirection::HorizontalDivider) => child_index == 1,

            _ => false,
        }
    }
}
