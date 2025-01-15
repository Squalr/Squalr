use crate::models::docking::dock_drag_direction::DockDragDirection;
use crate::models::docking::docking_tab_manager::DockingTabManager;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_split_direction::DockSplitDirection;
use crate::models::docking::hierarchy::dock_tree::DockTree;
use crate::models::docking::layout::docking_layout::DockingLayout;

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

    /// Adjusts a docked window in a given direction by (delta_x, delta_y) in pixels. Returns a bool indicating success.
    /// Adjusts a docked window in a given direction by (delta_x, delta_y) in pixels.
    /// Returns a bool indicating success.
    pub fn adjust_window_size(
        &mut self,
        leaf_id: &str,
        drag_dir: &DockDragDirection,
        delta_x: i32,
        delta_y: i32,
    ) -> bool {
        // Find the path to the leaf in the tree.
        let leaf_path = match self.tree.find_leaf_path(leaf_id) {
            Some(path) => path,
            None => return false,
        };
        if leaf_path.is_empty() {
            // Leaf is the root => no parent or ancestor
            return false;
        }

        // Determine which kind of split orientation we need.
        let desired_split_direction = match drag_dir {
            DockDragDirection::Left | DockDragDirection::Right => DockSplitDirection::VerticalDivider,
            DockDragDirection::Top | DockDragDirection::Bottom => DockSplitDirection::HorizontalDivider,
        };

        // Climb upward to find the first ancestor matching that orientation.
        let ancestor_path = match self.tree.find_ancestor_split_for_drag(&leaf_path, &drag_dir) {
            Some(path) => path,
            None => return false,
        };

        // Get the full layout size of the ancestor node that has the splitter.
        let ancestor_rect = match self.layout.find_node_rect(&self.tree, &ancestor_path) {
            Some(rect) => rect,
            None => return false,
        };
        let (_ancestor_x, _ancestor_y, ancestor_w, ancestor_h) = ancestor_rect;

        // Get a mutable reference to the ancestor node.
        let ancestor_node = match self.tree.get_node_mut(&ancestor_path) {
            Some(n) => n,
            None => return false,
        };

        // Perform ratio-based resizing if it’s a split.
        if let DockNode::Split {
            direction: split_direction,
            children,
        } = ancestor_node
        {
            // Double-check orientation.
            if *split_direction != desired_split_direction {
                return false;
            }

            match (drag_dir, split_direction) {
                // -----------------------------------
                // Vertical resizing (drag left/right)
                // -----------------------------------
                (DockDragDirection::Left | DockDragDirection::Right, DockSplitDirection::VerticalDivider) => {
                    if ancestor_w <= 1.0 {
                        return false;
                    }

                    // Find which child is the one that contains our leaf_id.
                    let child_index = children
                        .iter()
                        .enumerate()
                        .find(|(_, child)| child.node.contains_leaf_id(leaf_id))
                        .map(|(index, _)| index);

                    if let Some(child_index) = child_index {
                        // Decide which sibling to adjust in the “drag direction.”
                        let sibling_index = match drag_dir {
                            // If dragging the right edge of child_index, we grow that child and shrink the next.
                            DockDragDirection::Right => {
                                if child_index + 1 < children.len() {
                                    Some(child_index + 1)
                                } else {
                                    None
                                }
                            }
                            // If dragging the left edge, we shrink child_index and grow the previous child.
                            DockDragDirection::Left => {
                                if child_index > 0 {
                                    Some(child_index - 1)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };

                        // If we can’t find a sibling in that direction, we do nothing.
                        if sibling_index.is_none() {
                            return false;
                        }
                        let sibling_index = sibling_index.unwrap();

                        // Current ratios for the two children in question.
                        let old_child_ratio = children[child_index].ratio;
                        let old_sibling_ratio = children[sibling_index].ratio;
                        let sum_of_two = old_child_ratio + old_sibling_ratio;

                        // Convert to pixels for the child being dragged.
                        let current_child_px = old_child_ratio * ancestor_w;

                        // Determine the sign for the delta_x. If we’re dragging
                        // to the “right edge”, we might add; if we’re on the left edge, we subtract.
                        // But you can also just use the raw drag_dir logic:
                        let drag_sign = match drag_dir {
                            DockDragDirection::Right => 1.0,
                            DockDragDirection::Left => -1.0,
                            _ => 0.0,
                        };

                        // Compute new child pixel width and clamp it so it stays within [0, sum_of_two * ancestor_w].
                        let new_child_px = (current_child_px + drag_sign * (delta_x as f32))
                            .max(0.0)
                            .min(sum_of_two * ancestor_w);

                        // Convert back to ratio.
                        let new_child_ratio = new_child_px / ancestor_w;
                        // Sibling ratio is just the remainder of sum_of_two.
                        let new_sibling_ratio = sum_of_two - new_child_ratio;

                        children[child_index].ratio = new_child_ratio.clamp(0.0, 1.0);
                        children[sibling_index].ratio = new_sibling_ratio.clamp(0.0, 1.0);
                        true
                    } else {
                        false
                    }
                }

                // -----------------------------------
                // Horizontal resizing (drag top/bottom)
                // -----------------------------------
                (DockDragDirection::Top | DockDragDirection::Bottom, DockSplitDirection::HorizontalDivider) => {
                    if ancestor_h <= 1.0 {
                        return false;
                    }

                    let child_index = children
                        .iter()
                        .enumerate()
                        .find(|(_, child)| child.node.contains_leaf_id(leaf_id))
                        .map(|(index, _)| index);

                    if let Some(child_index) = child_index {
                        // Decide the sibling index based on top/bottom edges.
                        let sibling_index = match drag_dir {
                            // Dragging the bottom edge => grow that child, shrink next child
                            DockDragDirection::Bottom => {
                                if child_index + 1 < children.len() {
                                    Some(child_index + 1)
                                } else {
                                    None
                                }
                            }
                            // Dragging the top edge => shrink that child, grow the previous child
                            DockDragDirection::Top => {
                                if child_index > 0 {
                                    Some(child_index - 1)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };

                        if sibling_index.is_none() {
                            return false;
                        }
                        let sibling_index = sibling_index.unwrap();

                        // Current ratios for the two children.
                        let old_child_ratio = children[child_index].ratio;
                        let old_sibling_ratio = children[sibling_index].ratio;
                        let sum_of_two = old_child_ratio + old_sibling_ratio;

                        // Convert child’s ratio to pixel space.
                        let current_child_px = old_child_ratio * ancestor_h;

                        // Decide sign for delta_y.
                        let drag_sign = match drag_dir {
                            DockDragDirection::Bottom => 1.0,
                            DockDragDirection::Top => -1.0,
                            _ => 0.0,
                        };

                        // Compute new pixel height for the child.
                        let new_child_px = (current_child_px + drag_sign * (delta_y as f32))
                            .max(0.0)
                            .min(sum_of_two * ancestor_h);

                        // Convert back to ratio.
                        let new_child_ratio = new_child_px / ancestor_h;
                        let new_sibling_ratio = sum_of_two - new_child_ratio;

                        children[child_index].ratio = new_child_ratio.clamp(0.0, 1.0);
                        children[sibling_index].ratio = new_sibling_ratio.clamp(0.0, 1.0);
                        true
                    } else {
                        false
                    }
                }

                // If there’s a mismatch in drag direction vs. split direction, fail.
                _ => false,
            }
        } else {
            // Ancestor isn’t a split or something else went wrong.
            false
        }
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
}
