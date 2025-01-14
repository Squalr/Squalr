use crate::models::docking::dock_node::DockNode;
use crate::models::docking::dock_split_direction::DockSplitDirection;
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

    /// Find the path (series of child indices) to a leaf node by ID.
    /// Returns `None` if not found.
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
                    current = &mut children[idx];
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

    /// Return an immutable reference to the node at the specified path.
    /// Returns `None` if the path is invalid.
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
                    current = &children[idx];
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

    /// Returns a mutable reference to the parent of a particular leaf node
    /// along with the index of the leaf in its parent's children.
    ///
    /// For example, if `leaf_id` is found at path `[0, 1]` relative to the root,
    /// then the parent is whatever node is at `[0]`, and `leaf_index` is `1`.
    ///
    /// Returns None if `leaf_id` is not found or the leaf has no parent (i.e., the leaf is the root).
    pub fn get_parent_node_mut(
        &mut self,
        leaf_id: &str,
    ) -> Option<(&mut DockNode, usize)> {
        let path = self.find_leaf_path(leaf_id)?;
        if path.is_empty() {
            // The leaf is the root node; there is no parent.
            return None;
        }

        // Separate the parent's path from the leaf index.
        let (parent_path, &[leaf_index]) = path.split_at(path.len() - 1) else {
            return None;
        };

        // Get a mutable reference to the parent node.
        let parent_node = self.get_node_mut(parent_path)?;
        Some((parent_node, leaf_index))
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

    /// Finds the path of the *first ancestor* that is a `DockNode::Split`
    /// with a matching `direction`. Returns `None` if not found.
    ///
    /// For example:
    /// - If `desired_direction` is `DockSplitDirection::HorizontalDivider`,
    ///   we climb upward until we find a `DockNode::Split { direction: Horizontal, .. }`.
    /// - If we reach the root without finding a match, return `None`.
    pub fn find_ancestor_split(
        &self,
        leaf_path: &[usize],
        desired_direction: &DockSplitDirection,
    ) -> Option<Vec<usize>> {
        // If the leaf_path is empty, that means the leaf *is* the root. No ancestor to find.
        if leaf_path.is_empty() {
            return None;
        }

        // Make a local copy, so we can pop without mutating the original
        let mut current_path = leaf_path.to_vec();

        // First pop once so we skip the leaf itself
        current_path.pop();

        // Now climb upward until we find a split with the correct direction
        loop {
            // Check the node at current_path (which may now be the root if path is empty).
            if let Some(node) = self.get_node(&current_path) {
                if let DockNode::Split { direction, .. } = node {
                    if *direction == *desired_direction {
                        return Some(current_path.clone());
                    }
                }
            } else {
                // If we can't retrieve a node, the path is invalid. Bail out.
                return None;
            }

            // If we have popped all the way up to an empty path
            // and haven't matched, there's no ancestor of that direction.
            if current_path.is_empty() {
                return None;
            }

            current_path.pop();
        }
    }
}
