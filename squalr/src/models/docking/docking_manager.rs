use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_tree::DockTree;
use crate::models::docking::layout::dock_drag_direction::DockDragDirection;
use crate::models::docking::layout::dock_layout::DockLayout;

/// Handles a `DockTree` and its corresponding layout information.
pub struct DockingManager {
    pub tree: DockTree,
    pub layout: DockLayout,
}

/// Contains various helper functions to manage an underlying docking hierarchy and its layout.
impl DockingManager {
    pub fn new(root_node: DockNode) -> Self {
        Self {
            tree: DockTree::new(root_node),
            layout: DockLayout::new(),
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
    pub fn get_layout(&self) -> &DockLayout {
        &self.layout
    }

    /// Gets the layout handler that computes the bounds and location of each docked window (mutable).
    pub fn get_layout_mut(&mut self) -> &mut DockLayout {
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
        self.tree.select_tab_by_leaf_id(leaf_id)
    }

    /// Given a `leaf_id` and a `DockTree`, this method determines the list of sibling tabs, as well as which one is active.
    pub fn get_siblings_and_active_tab(
        &self,
        leaf_id: &str,
    ) -> (Vec<String>, String) {
        self.tree.get_siblings_and_active_tab(leaf_id)
    }

    /// Prepare for presentation by fixing up invalid state.
    pub fn prepare_for_presentation(&mut self) {
        self.tree.clean_up_hierarchy();
        self.tree.run_tab_validation();
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
        self.layout
            .adjust_window_size(&mut self.tree, leaf_id, drag_dir, delta_x, delta_y)
    }
}
