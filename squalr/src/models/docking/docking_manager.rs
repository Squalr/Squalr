use crate::models::docking::dock_drag_direction::DockDragDirection;
use crate::models::docking::docking_tab_manager::DockingTabManager;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_split_direction::DockSplitDirection;
use crate::models::docking::hierarchy::dock_tree::DockTree;
use crate::models::docking::layout::docking_layout::DockingLayout;

/// Handles a `DockTree` and its corresponding layout information.
pub struct DockingManager {
    pub tree: DockTree,
    pub layout: DockingLayout,
}

/// Contains various helper functions to manage an underlying docking hierarchy and its layout.
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

    /// Gets the layout handler that computes the bounds and location of each docked window (immutable).
    pub fn get_layout(&self) -> &DockingLayout {
        &self.layout
    }

    /// Gets the layout handler that computes the bounds and location of each docked window (mutable).
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

    /// Activate a window in its tab (if parent is a tab).
    pub fn select_tab_by_leaf_id(
        &mut self,
        leaf_id: &str,
    ) -> bool {
        DockingTabManager::select_tab_by_leaf_id(&mut self.tree, leaf_id)
    }

    /// Given a `leaf_id` and a `DockTree`, this method determines the list of sibling tabs, as well as which one is active.
    pub fn get_siblings_and_active_tab(
        &self,
        leaf_id: &str,
    ) -> (Vec<String>, String) {
        DockingTabManager::get_siblings_and_active_tab(&self.tree, leaf_id)
    }

    /// Prepare for presentation by fixing up invalid state.
    pub fn prepare_for_presentation(&mut self) {
        self.tree.clean_up_hierarchy();
        DockingTabManager::run_tab_validation(&mut self.tree.root);
    }

    /// Tries to resize a window by dragging one of its edges in the given direction
    /// by (delta_x, delta_y) pixels. We climb up the dock hierarchy if the leaf’s
    /// immediate parent split cannot accommodate the drag.
    ///
    /// This approach ensures we don’t simultaneously borrow `self` mutably for both
    /// the layout lookups and the tree mutations.
    pub fn adjust_window_size(
        &mut self,
        leaf_id: &str,
        drag_dir: &DockDragDirection,
        delta_x: i32,
        delta_y: i32,
    ) -> bool {
        // 1) Locate the path to the leaf node we’re resizing.
        let leaf_path = match self.tree.find_leaf_path(leaf_id) {
            Some(path) => path,
            None => return false,
        };

        // 2) Determine the required orientation (vertical or horizontal).
        let desired_split_direction = match drag_dir {
            DockDragDirection::Left | DockDragDirection::Right => DockSplitDirection::VerticalDivider,
            DockDragDirection::Top | DockDragDirection::Bottom => DockSplitDirection::HorizontalDivider,
        };

        // 3) Climb up from the leaf to its ancestors.
        let mut current_path = leaf_path;
        while !current_path.is_empty() {
            // The last index is the child index in the parent's children array (or tabs array).
            let child_index = match current_path.last() {
                Some(&idx) => idx,
                None => return false,
            };
            // Everything up to (but not including) that last index is the "parent" path.
            let parent_path = &current_path[..current_path.len() - 1];

            // Find the bounding rectangle from the layout.
            let bounding_rect_opt = self.layout.find_node_rect(&self.tree, parent_path);
            let (ancestor_w, ancestor_h) = match bounding_rect_opt {
                Some((_, _, w, h)) => (w, h),
                None => {
                    // We have no layout data for this node => can't do ratio resizing here.
                    current_path.pop();
                    continue;
                }
            };

            let Some(parent_node) = self.tree.get_node_mut(parent_path) else {
                return false;
            };

            // Check if this parent is a Split with the correct orientation.
            let is_correct_split = match parent_node {
                DockNode::Split { direction, .. } => *direction == desired_split_direction,
                _ => false,
            };

            // If it matches, attempt resizing siblings.
            if is_correct_split {
                let resized = Self::try_resize_siblings_in_split(parent_node, child_index, drag_dir, delta_x, delta_y, ancestor_w, ancestor_h);
                if resized {
                    return true;
                }
            }

            // Resize failed, climb one level up and try again.
            current_path.pop();
        }

        // If we reach here, we’ve climbed to the root with no success -- there is no sibling in that direction.
        false
    }

    /// A helper that adjusts ratios in a `DockNode::Split` when a user drags
    /// a particular child’s edge. Returns `true` on success, `false` if no valid sibling
    /// was found or if it couldn’t resize for some reason.
    ///
    /// We make this a static method (or free function) so we do NOT need to borrow `self`
    /// again (preventing multiple mutable borrows).
    fn try_resize_siblings_in_split(
        parent_node: &mut DockNode,
        child_index: usize,
        drag_dir: &DockDragDirection,
        delta_x: i32,
        delta_y: i32,
        ancestor_w: f32,
        ancestor_h: f32,
    ) -> bool {
        // Make sure we have a split node
        let DockNode::Split { direction, children } = parent_node else {
            return false;
        };

        // If orientation doesn’t match the drag direction, bail out
        match (drag_dir, &direction) {
            (DockDragDirection::Left | DockDragDirection::Right, DockSplitDirection::VerticalDivider) => {}
            (DockDragDirection::Top | DockDragDirection::Bottom, DockSplitDirection::HorizontalDivider) => {}
            _ => return false,
        }

        // Figure out:
        //  - which dimension to work with (width vs height),
        //  - which delta to use (delta_x or delta_y),
        //  - which sign (left/up => -1, right/down => +1),
        //  - which sibling is affected.
        //
        // If any of these conditions fails (e.g. no sibling in that direction),
        // we return false.
        let (dimension, delta, sign, sibling_idx) = match drag_dir {
            // -- Vertical divider (Left/Right drag) --
            DockDragDirection::Right if *direction == DockSplitDirection::VerticalDivider => {
                // Check if we have a sibling to the right
                if child_index + 1 < children.len() {
                    (ancestor_w, delta_x as f32, 1.0, child_index + 1)
                } else {
                    return false;
                }
            }
            DockDragDirection::Left if *direction == DockSplitDirection::VerticalDivider => {
                // Check if we have a sibling to the left
                if child_index > 0 {
                    (ancestor_w, delta_x as f32, -1.0, child_index - 1)
                } else {
                    return false;
                }
            }

            // -- Horizontal divider (Top/Bottom drag) --
            DockDragDirection::Bottom if *direction == DockSplitDirection::HorizontalDivider => {
                if child_index + 1 < children.len() {
                    (ancestor_h, delta_y as f32, 1.0, child_index + 1)
                } else {
                    return false;
                }
            }
            DockDragDirection::Top if *direction == DockSplitDirection::HorizontalDivider => {
                if child_index > 0 {
                    (ancestor_h, delta_y as f32, -1.0, child_index - 1)
                } else {
                    return false;
                }
            }

            // Anything else is unsupported/mismatched
            _ => return false,
        };

        // If the dimension is too tiny, we can’t meaningfully resize
        if dimension <= 1.0 {
            return false;
        }

        // Calculate new ratios for the child and its sibling
        let old_child_ratio = children[child_index].ratio;
        let old_sibling_ratio = children[sibling_idx].ratio;
        let sum = old_child_ratio + old_sibling_ratio;

        // Convert the child’s ratio to “pixels”
        let current_child_px = old_child_ratio * dimension;
        let new_child_px = (current_child_px + sign * delta)
            // Don’t let the child shrink below zero or push sibling below zero
            .max(0.0)
            .min(sum * dimension);

        // Convert back to ratio
        let new_child_ratio = new_child_px / dimension;
        let new_sibling_ratio = sum - new_child_ratio;

        // Write them back, clamped to [0, 1] (just in case)
        children[child_index].ratio = new_child_ratio.clamp(0.0, 1.0);
        children[sibling_idx].ratio = new_sibling_ratio.clamp(0.0, 1.0);

        true
    }
}
