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
    pub fn adjust_window_size(
        &mut self,
        leaf_id: &str,
        drag_dir: DockDragDirection,
        delta_x: i32,
        delta_y: i32,
    ) -> bool {
        let leaf_path = match self.tree.find_leaf_path(leaf_id) {
            Some(path) => path,
            None => return false,
        };
        if leaf_path.is_empty() {
            // Leaf is the root => no parent or ancestor
            return false;
        }

        // Figure out the needed split direction from the drag direction (this will be inverse to drag direction).
        let desired_split_direction = match drag_dir {
            DockDragDirection::Left | DockDragDirection::Right => DockSplitDirection::VerticalDivider,
            DockDragDirection::Top | DockDragDirection::Bottom => DockSplitDirection::HorizontalDivider,
        };

        // Climb upward to find the first ancestor matching that direction.
        let ancestor_path = match self.tree.find_ancestor_split_for_drag(&leaf_path, &drag_dir) {
            Some(path) => path,
            None => {
                return false;
            }
        };

        // Get the full layout size of the matching ancestor that contains the splitter.
        let ancestor_rect = match self.layout.find_node_rect(&self.tree, &ancestor_path) {
            Some(rect) => rect,
            None => return false,
        };
        let (_ancestor_x, _ancestor_y, ancestor_w, ancestor_h) = ancestor_rect;

        // Get the target window rect so that we can save off the starting width/height.
        let leaf_rect = match self.layout.find_node_rect(&self.tree, &leaf_path) {
            Some(rect) => rect,
            None => return false,
        };
        let (_child_x, _child_y, child_w, child_h) = leaf_rect;

        let ancestor_node = match self.tree.get_node_mut(&ancestor_path) {
            Some(n) => n,
            None => return false,
        };

        // Perform ratio-based resizing.
        if let DockNode::Split {
            direction: split_direction,
            children,
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
                    // So we search for the child subtree containing `leaf_id`.
                    let child_index = children
                        .iter()
                        .enumerate()
                        .find(|(_, child)| child.node.contains_leaf_id(leaf_id))
                        .map(|(index, _)| index);

                    if let Some(child_index) = child_index {
                        let old_width = child_w;
                        let sign = if child_index == 0 { 1.0 } else { -1.0 };
                        let new_width = old_width + sign * (delta_x as f32);
                        let new_ratio = (new_width / ancestor_w).clamp(0.0, 1.0);

                        children[child_index].ratio = new_ratio;

                        // TODO: Suppert N children
                        if children.len() == 2 {
                            let sibling_index = if child_index == 0 { 1 } else { 0 };
                            children[sibling_index].ratio = (1.0 - new_ratio).clamp(0.0, 1.0);
                        }
                        true
                    } else {
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
                        .find(|(_, child)| child.node.contains_leaf_id(leaf_id))
                        .map(|(index, _)| index);

                    if let Some(child_index) = child_index {
                        let old_height = child_h;
                        let sign = if child_index == 0 { 1.0 } else { -1.0 };
                        let new_height = old_height + sign * (delta_y as f32);
                        let new_ratio = (new_height / ancestor_h).clamp(0.0, 1.0);

                        children[child_index].ratio = new_ratio;

                        if children.len() == 2 {
                            let sibling_index = if child_index == 0 { 1 } else { 0 };
                            children[sibling_index].ratio = (1.0 - new_ratio).clamp(0.0, 1.0);
                        }
                        true
                    } else {
                        false
                    }
                }

                _ => false,
            }
        } else {
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

    /// Prepare for presentation by fixing up tabs, etc.
    pub fn prepare_for_presentation(&mut self) {
        DockingTabManager::run_tab_validation(&mut self.tree.root);
    }
}
