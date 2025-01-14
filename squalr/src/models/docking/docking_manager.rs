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
        drag_dir: DockDragDirection,
        delta_x: i32,
        delta_y: i32,
    ) -> bool {
        // 1) Find the leaf’s path
        let leaf_path = match self.tree.find_leaf_path(leaf_id) {
            Some(path) => path,
            None => return false,
        };
        if leaf_path.is_empty() {
            // Leaf is the root => no parent or ancestor
            return false;
        }

        // 2) Figure out the needed split direction from the drag direction
        let desired_split_direction = match drag_dir {
            DockDragDirection::Left | DockDragDirection::Right => DockSplitDirection::VerticalDivider,
            DockDragDirection::Top | DockDragDirection::Bottom => DockSplitDirection::HorizontalDivider,
        };

        // 3) Climb upward to find the first ancestor matching that direction
        let ancestor_path = match self
            .tree
            .find_ancestor_split(&leaf_path, &desired_split_direction)
        {
            Some(path) => path,
            None => {
                // e.g. we never found a vertical/horizontal split up the chain
                return false;
            }
        };

        // 4) Gather rectangle info *before* mutably borrowing anything
        //    (to avoid the "cannot borrow as mutable and immutable" problem).
        let ancestor_rect = match self.layout.find_node_rect(&self.tree, &ancestor_path) {
            Some(rect) => rect,
            None => return false,
        };
        let (_ancestor_x, _ancestor_y, ancestor_w, ancestor_h) = ancestor_rect;

        // For convenience, let's also get the *leaf* rect (to figure out old width/height).
        let leaf_rect = match self.layout.find_node_rect(&self.tree, &leaf_path) {
            Some(rect) => rect,
            None => return false,
        };
        let (_child_x, _child_y, child_w, child_h) = leaf_rect;

        // 5) Now do the *mutable* borrowing of that ancestor node
        let ancestor_node = match self.tree.get_node_mut(&ancestor_path) {
            Some(n) => n,
            None => return false,
        };

        // 6) Perform ratio-based resizing on that ancestor’s children
        if let DockNode::Split {
            direction: split_direction,
            children,
            ..
        } = ancestor_node
        {
            // Double-check we got the direction we expected
            if *split_direction != desired_split_direction {
                return false;
            }

            // Now do the ratio math
            match (drag_dir, split_direction) {
                (DockDragDirection::Left | DockDragDirection::Right, DockSplitDirection::VerticalDivider) => {
                    if ancestor_w <= 1.0 {
                        return false;
                    }

                    // Next, we must figure out: which child in `children` corresponds to the `leaf_id`?
                    // Because `leaf_path` might be a deep path, not necessarily direct child of `ancestor_node`.
                    // So we search for the child *subtree* containing `leaf_id`.
                    let child_index = children
                        .iter()
                        .enumerate()
                        .find(|(_, c)| c.contains_leaf_id(leaf_id)) // `contains_leaf_id` is an optional helper
                        .map(|(i, _)| i);

                    if let Some(child_index) = child_index {
                        let old_width = child_w;
                        let sign = if child_index == 0 { 1.0 } else { -1.0 };
                        let new_width = old_width + sign * (delta_x as f32);
                        let new_ratio = (new_width / ancestor_w).clamp(0.0, 1.0);

                        // Set the ratio
                        if let Some(child_node) = children.get_mut(child_index) {
                            child_node.set_ratio(new_ratio);
                        }

                        // If there's exactly two children, set sibling ratio to remainder
                        if children.len() == 2 {
                            let sibling_index = if child_index == 0 { 1 } else { 0 };
                            if let Some(sibling) = children.get_mut(sibling_index) {
                                sibling.set_ratio((1.0 - new_ratio).clamp(0.0, 1.0));
                            }
                        }
                        true
                    } else {
                        // We didn't find which direct child to apply the ratio
                        false
                    }
                }

                (DockDragDirection::Top | DockDragDirection::Bottom, DockSplitDirection::HorizontalDivider) => {
                    if ancestor_h <= 1.0 {
                        return false;
                    }

                    let child_index = children
                        .iter()
                        .enumerate()
                        .find(|(_, c)| c.contains_leaf_id(leaf_id))
                        .map(|(i, _)| i);

                    if let Some(child_index) = child_index {
                        let old_height = child_h;
                        let sign = if child_index == 0 { 1.0 } else { -1.0 };
                        let new_height = old_height + sign * (delta_y as f32);
                        let new_ratio = (new_height / ancestor_h).clamp(0.0, 1.0);

                        if let Some(child_node) = children.get_mut(child_index) {
                            child_node.set_ratio(new_ratio);
                        }

                        if children.len() == 2 {
                            let sibling_index = if child_index == 0 { 1 } else { 0 };
                            if let Some(sibling) = children.get_mut(sibling_index) {
                                sibling.set_ratio((1.0 - new_ratio).clamp(0.0, 1.0));
                            }
                        }
                        true
                    } else {
                        false
                    }
                }

                _ => false,
            }
        } else {
            // The ancestor node we found is not a split? Or direction mismatch.
            false
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
