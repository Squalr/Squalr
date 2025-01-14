use crate::models::docking::dock_drag_direction::DockDragDirection;
use crate::models::docking::dock_node::DockNode;
use crate::models::docking::dock_split_direction::DockSplitDirection;
use crate::models::docking::dock_tree::DockTree;
use crate::models::docking::layout::docking_layout::DockingLayout;
use crate::models::docking::tab_manager::TabManager;

pub struct DockingManager {
    pub tree: DockTree,
    pub layout: DockingLayout,
}

impl DockingManager {
    pub fn new(root_node: DockNode) -> Self {
        Self {
            tree: DockTree::new(root_node),
            layout: DockingLayout::new(),
        }
    }

    /// Replace the entire root node in the tree.
    pub fn set_root(
        &mut self,
        new_root: DockNode,
    ) {
        self.tree.replace_root(new_root);
    }

    /// Just expose the root if needed.
    pub fn get_root(&self) -> &DockNode {
        &self.tree.root
    }

    pub fn get_layout(&self) -> &DockingLayout {
        &self.layout
    }

    pub fn get_layout_mut(&mut self) -> &mut DockingLayout {
        &mut self.layout
    }

    /// Retrieve a node by ID (immutable).
    pub fn get_node_by_id(
        &self,
        identifier: &str,
    ) -> Option<&DockNode> {
        let path = self.tree.find_leaf_path(identifier)?;
        self.tree.get_node(&path)
    }

    /// Retrieve a node by ID (mutable).
    pub fn get_node_by_id_mut(
        &mut self,
        identifier: &str,
    ) -> Option<&mut DockNode> {
        let path = self.tree.find_leaf_path(identifier)?;
        self.tree.get_node_mut(&path)
    }

    /// Collect all leaf IDs from the tree.
    pub fn get_all_leaves(&self) -> Vec<String> {
        self.tree.get_all_leaves()
    }

    /// Find the bounding rectangle for a particular leaf.
    pub fn find_window_rect(
        &self,
        leaf_id: &str,
    ) -> Option<(f32, f32, f32, f32)> {
        self.layout.find_window_rect(&self.tree, leaf_id)
    }

    /// Example: resize a window by adjusting its ratio.
    pub fn resize_window(
        &mut self,
        leaf_id: &str,
        new_ratio: f32,
    ) -> bool {
        let path = match self.tree.find_leaf_path(leaf_id) {
            Some(path) => path,
            None => return false,
        };

        if let Some(node) = self.tree.get_node_mut(&path) {
            node.set_ratio(new_ratio);
        } else {
            return false;
        }

        // If there's a parent with exactly two children, adjust the sibling.
        if !path.is_empty() {
            let (parent_slice, &[leaf_index]) = path.split_at(path.len() - 1) else {
                return true;
            };

            if let Some(parent_node) = self.tree.get_node_mut(parent_slice) {
                match parent_node {
                    DockNode::Split { children, .. } if children.len() == 2 => {
                        let sibling_idx = if leaf_index == 0 { 1 } else { 0 };
                        let sibling_ratio = (1.0 - new_ratio).clamp(0.0, 1.0);
                        children[sibling_idx].set_ratio(sibling_ratio);
                    }
                    DockNode::Tab { tabs, .. } if tabs.len() == 2 => {
                        let sibling_idx = if leaf_index == 0 { 1 } else { 0 };
                        let sibling_ratio = (1.0 - new_ratio).clamp(0.0, 1.0);
                        tabs[sibling_idx].set_ratio(sibling_ratio);
                    }
                    _ => {}
                }
            }
        }

        true
    }

    /// Activate a leaf in its tab (if parent is a tab).
    pub fn select_tab_by_leaf_id(
        &mut self,
        leaf_id: &str,
    ) -> bool {
        let path = match self.tree.find_leaf_path(leaf_id) {
            Some(path) => path,
            None => return false,
        };
        if path.is_empty() {
            return false;
        }
        let (parent_slice, _) = path.split_at(path.len() - 1);

        if let Some(parent_node) = self.tree.get_node_mut(parent_slice) {
            if let DockNode::Tab { active_tab_id, .. } = parent_node {
                *active_tab_id = leaf_id.to_owned();
                return true;
            }
        }
        false
    }

    /// Drags a leaf node in a given direction by (delta_x, delta_y) in pixels. Returns a bool indicating success.
    pub fn drag_leaf(
        &mut self,
        leaf_id: &str,
        direction: DockDragDirection,
        delta_x: i32,
        delta_y: i32,
    ) -> bool {
        // The parent's bounding rectangle is needed to convert pixel deltas to ratio changes.
        // (This uses the entire root rect — if you actually want the parent's own rect,
        //  consider path-based logic, but here’s the direct fix requested.)
        let parent_rect = match self.layout.find_node_rect(&self.tree, &[]) {
            Some(rect) => rect,
            None => return false,
        };
        let (parent_left, parent_top, parent_width, parent_height) = parent_rect;

        // The child's rectangle
        let child_rect = match self.find_window_rect(leaf_id) {
            Some(rect) => rect,
            None => return false,
        };
        let (child_left, child_top, child_width, child_height) = child_rect;

        let (parent_node, leaf_index) = match self.tree.get_parent_node_mut(leaf_id) {
            Some(pair) => pair,
            None => {
                // The provided leaf seems not to have a parent!
                return false;
            }
        };

        match parent_node {
            DockNode::Split {
                direction: split_direction,
                children,
                ..
            } => {
                match (direction, split_direction) {
                    // --- If we drag left/right, we want a Horizontal split (children side by side) ---
                    (DockDragDirection::Left | DockDragDirection::Right, DockSplitDirection::Horizontal) => {
                        if parent_width <= 1.0 {
                            return false;
                        }
                        let old_width = child_width;
                        let new_width = old_width + delta_x as f32;
                        let new_ratio = (new_width / parent_width).clamp(0.0, 1.0);

                        if let Some(child_node) = children.get_mut(leaf_index) {
                            child_node.set_ratio(new_ratio);
                        }
                        if children.len() == 2 {
                            let sibling_index = if leaf_index == 0 { 1 } else { 0 };
                            if let Some(sibling) = children.get_mut(sibling_index) {
                                sibling.set_ratio((1.0 - new_ratio).clamp(0.0, 1.0));
                            }
                        }
                        true
                    }

                    // --- If we drag top/bottom, we want a Vertical split (children stacked) ---
                    (DockDragDirection::Top | DockDragDirection::Bottom, DockSplitDirection::Vertical) => {
                        if parent_height <= 1.0 {
                            return false;
                        }
                        let old_height = child_height;
                        let new_height = old_height + delta_y as f32;
                        let new_ratio = (new_height / parent_height).clamp(0.0, 1.0);

                        if let Some(child_node) = children.get_mut(leaf_index) {
                            child_node.set_ratio(new_ratio);
                        }
                        if children.len() == 2 {
                            let sibling_index = if leaf_index == 0 { 1 } else { 0 };
                            if let Some(sibling) = children.get_mut(sibling_index) {
                                sibling.set_ratio((1.0 - new_ratio).clamp(0.0, 1.0));
                            }
                        }
                        true
                    }

                    // Otherwise do nothing (mismatch between drag direction & split direction).
                    _ => false,
                }
            }
            // If parent is not a Split, do nothing.
            _ => false,
        }
    }

    pub fn get_siblings_and_active_tab(
        &self,
        leaf_id: &str,
    ) -> (Vec<String>, String) {
        // Find the path to this leaf.
        let path = match self.tree.find_leaf_path(leaf_id) {
            Some(p) => p,
            None => return (Vec::new(), leaf_id.to_owned()),
        };

        // If the path is empty, there's no parent => return fallback.
        if path.is_empty() {
            return (Vec::new(), leaf_id.to_owned());
        }

        // Everything except the last index is the parent path.
        let (parent_path, _) = path.split_at(path.len() - 1);

        // Get the parent node from the tree
        if let Some(parent_node) = self.tree.get_node(parent_path) {
            if let DockNode::Tab { tabs, active_tab_id, .. } = parent_node {
                // Collect all visible siblings in this Tab.
                let mut siblings = Vec::new();
                for tab_node in tabs {
                    if let DockNode::Leaf {
                        window_identifier, is_visible, ..
                    } = tab_node
                    {
                        if *is_visible {
                            siblings.push(window_identifier.clone());
                        }
                    }
                }
                return (siblings, active_tab_id.clone());
            }
        }

        // If not found or parent not a Tab, fallback.
        (Vec::new(), leaf_id.to_owned())
    }

    /// Prepare for presentation by fixing up tabs, etc.
    pub fn prepare_for_presentation(&mut self) {
        TabManager::prepare_for_presentation(&mut self.tree.root);
    }
}
