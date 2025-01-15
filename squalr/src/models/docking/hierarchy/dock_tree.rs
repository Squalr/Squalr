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

    /// Find the matching ancestor `DockNode::Split` that has the correct orientation
    /// for a particular drag direction (left/right = vertical, top/bottom = horizontal).
    /// We climb up the tree from the given `leaf_path`, returning the path to the first
    /// ancestor split that matches.
    pub fn find_ancestor_split_for_drag(
        &self,
        leaf_path: &[usize],
        drag_dir: &DockDragDirection,
    ) -> Option<Vec<usize>> {
        if leaf_path.is_empty() {
            return None;
        }

        // We'll climb up. Remove the leaf's index from the path to see its parent path.
        let mut path = leaf_path.to_vec();
        path.pop(); // remove the leaf's own index

        // Now climb up the tree, checking each ancestor node
        while let Some(_) = self.get_node(&path) {
            if let Some(node) = self.get_node(&path) {
                if let DockNode::Split { direction, .. } = node {
                    if Self::has_compatible_orientation(direction, drag_dir) {
                        return Some(path.clone());
                    }
                }
            }
            // Move one step higher
            if path.is_empty() {
                break; // no more parents
            }
            path.pop();
        }

        None
    }

    /// Instead of matching child_index == 0/1, just check if the direction is vertical vs horizontal
    /// and see if it matches the drag direction (left/right vs top/bottom).
    fn has_compatible_orientation(
        direction: &DockSplitDirection,
        drag_dir: &DockDragDirection,
    ) -> bool {
        match (direction, drag_dir) {
            (DockSplitDirection::VerticalDivider, DockDragDirection::Left) | (DockSplitDirection::VerticalDivider, DockDragDirection::Right) => true,
            (DockSplitDirection::HorizontalDivider, DockDragDirection::Top) | (DockSplitDirection::HorizontalDivider, DockDragDirection::Bottom) => true,
            _ => false,
        }
    }

    /// Recursively clean up the docking hierarchy so that:
    /// - A Split node with only 1 child is replaced by that child.
    /// - A Tab node with only 1 child is replaced by that child.
    pub fn clean_up_hierarchy(&mut self) {
        Self::clean_up_node(&mut self.root);
    }

    /// Recursively walk the subtree and remove containers that have only 1 child.
    pub fn clean_up_node(dock_node: &mut DockNode) {
        match dock_node {
            // For Split nodes, clean each child first, then see if there's only one child left.
            DockNode::Split { children, .. } => {
                for child in children.iter_mut() {
                    Self::clean_up_node(&mut child.node);
                }
                // If there's exactly one child, replace self with that child.
                if children.len() == 1 {
                    let single_child = children.remove(0).node;
                    *dock_node = single_child;
                }
            }

            // For Tab nodes, clean each tab first, then see if there's only one tab left.
            DockNode::Tab { tabs, .. } => {
                for tab in tabs.iter_mut() {
                    Self::clean_up_node(tab);
                }
                // If there's exactly one tab, replace self with that tab.
                if tabs.len() == 1 {
                    let single_tab = tabs.remove(0);
                    *dock_node = single_tab;
                }
            }

            // Leaf nodes have no children to clean up.
            DockNode::Leaf { .. } => {}
        }
    }
}
