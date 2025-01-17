use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::operations::dock_reparent_direction::DockReparentDirection;
use crate::models::docking::layout::dock_layout::DockLayout;
use crate::models::docking::layout::dock_splitter_drag_direction::DockSplitterDragDirection;

/// Handles a `DockNode` and its corresponding layout information.
pub struct DockingManager {
    pub root_node: DockNode,
    pub layout: DockLayout,
}

/// Contains various helper functions to manage an underlying docking hierarchy and its layout.
impl DockingManager {
    pub fn new(root_node: DockNode) -> Self {
        Self {
            root_node: root_node,
            layout: DockLayout::new(),
        }
    }

    /// Replace the entire root node in the root_node.
    pub fn set_root(
        &mut self,
        new_root: DockNode,
    ) {
        self.root_node = new_root;
    }

    /// Just expose the root if needed.
    pub fn get_root(&self) -> &DockNode {
        &self.root_node
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
        let path = self.root_node.find_path_to_window_id(identifier)?;
        self.root_node.get_node_from_path(&path)
    }

    /// Retrieve a node by ID (mutable).
    pub fn get_node_by_id_mut(
        &mut self,
        identifier: &str,
    ) -> Option<&mut DockNode> {
        let path = self.root_node.find_path_to_window_id(identifier)?;
        self.root_node.get_node_from_path_mut(&path)
    }

    /// Collect all leaf IDs from the root_node.
    pub fn get_all_child_window_ids(&self) -> Vec<String> {
        self.root_node.get_all_child_window_ids()
    }

    /// Find the bounding rectangle for a particular leaf.
    pub fn find_window_rect(
        &self,
        leaf_id: &str,
    ) -> Option<(f32, f32, f32, f32)> {
        self.layout.find_window_rect(&self.root_node, leaf_id)
    }

    /// Activate a window in its tab (if parent is a tab).
    pub fn select_tab_by_leaf_id(
        &mut self,
        leaf_id: &str,
    ) -> bool {
        self.root_node.select_tab_by_leaf_id(leaf_id)
    }

    /// Given a `leaf_id` and a `DockTree`, this method determines the list of sibling tabs, as well as which one is active.
    pub fn get_siblings_and_active_tab(
        &self,
        leaf_id: &str,
    ) -> (Vec<String>, String) {
        self.root_node.get_siblings_and_active_tab(leaf_id)
    }

    /// Prepare for presentation by fixing up invalid state.
    pub fn prepare_for_presentation(&mut self) {
        self.root_node.remove_invalid_containers();
        self.root_node.run_active_tab_validation();
    }

    /// Tries to resize a window by dragging one of its edges in the given direction
    /// by (delta_x, delta_y) pixels. We climb up the dock hierarchy if the leaf’s
    /// immediate parent split cannot accommodate the drag.
    ///
    /// This approach ensures we don’t simultaneously borrow `self` mutably for both
    /// the layout lookups and the root_node mutations.
    pub fn adjust_window_size(
        &mut self,
        leaf_id: &str,
        drag_dir: &DockSplitterDragDirection,
        delta_x: i32,
        delta_y: i32,
    ) -> bool {
        self.layout
            .adjust_window_size(&mut self.root_node, leaf_id, drag_dir, delta_x, delta_y)
    }

    pub fn reparent_leaf(
        &mut self,
        source_id: &str,
        target_id: &str,
        direction: DockReparentDirection,
    ) -> bool {
        self.root_node.reparent_leaf(source_id, target_id, direction)
    }
}
