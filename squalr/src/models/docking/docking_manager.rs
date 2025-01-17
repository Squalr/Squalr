use crate::models::docking::hierarchy::dock_layout::DockLayout;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_reparent_direction::DockReparentDirection;
use crate::models::docking::hierarchy::types::dock_splitter_drag_direction::DockSplitterDragDirection;

/// Handles a `DockLayout`, which contains a root `DockNode` and manages its layout.
pub struct DockingManager {
    pub main_window_layout: DockLayout,
}

/// Contains various helper functions to manage an underlying docking hierarchy and its layout.
impl DockingManager {
    pub fn new(root_node: DockNode) -> Self {
        Self {
            main_window_layout: DockLayout::new(root_node),
        }
    }

    /// Replace the entire root node in the root_node.
    pub fn set_root(
        &mut self,
        new_root: DockNode,
    ) {
        self.main_window_layout.set_root(new_root);
    }

    /// Get the root node of the docking main_window_layout.
    pub fn get_root(&self) -> &DockNode {
        &self.main_window_layout.get_root()
    }

    /// Gets the layout handler that computes the bounds and location of each docked window (immutable).
    pub fn get_main_window_layout(&self) -> &DockLayout {
        &self.main_window_layout
    }

    /// Gets the layout handler that computes the bounds and location of each docked window (mutable).
    pub fn get_main_window_layout_mut(&mut self) -> &mut DockLayout {
        &mut self.main_window_layout
    }

    /// Retrieve a node by ID (immutable).
    pub fn get_node_by_id(
        &self,
        identifier: &str,
    ) -> Option<&DockNode> {
        let path = self
            .main_window_layout
            .get_root()
            .find_path_to_window_id(identifier)?;
        self.main_window_layout.get_root().get_node_from_path(&path)
    }

    /// Retrieve a node by ID (mutable).
    pub fn get_node_by_id_mut(
        &mut self,
        identifier: &str,
    ) -> Option<&mut DockNode> {
        let path = self
            .main_window_layout
            .get_root()
            .find_path_to_window_id(identifier)?;
        let root = self.main_window_layout.get_root_mut();
        root.get_node_from_path_mut(&path)
    }

    /// Collect all window IDs from the root_node.
    pub fn get_all_child_window_ids(&self) -> Vec<String> {
        self.main_window_layout.get_root().get_all_child_window_ids()
    }

    /// Find the bounding rectangle for a particular window.
    pub fn find_window_rect(
        &self,
        window_id: &str,
    ) -> Option<(f32, f32, f32, f32)> {
        self.main_window_layout
            .find_window_rect(&self.main_window_layout.get_root(), window_id)
    }

    /// Activate a window in its tab (if parent is a tab).
    pub fn select_tab_by_window_id(
        &mut self,
        window_id: &str,
    ) -> bool {
        let root = self.main_window_layout.get_root_mut();
        root.select_tab_by_window_id(window_id)
    }

    /// Given a `window_id` and a `DockTree`, this method determines the list of sibling tabs, as well as which one is active.
    pub fn get_siblings_and_active_tab(
        &self,
        window_id: &str,
    ) -> (Vec<String>, String) {
        self.main_window_layout
            .get_root()
            .get_siblings_and_active_tab(window_id)
    }

    /// Prepare for presentation by fixing up invalid state.
    pub fn prepare_for_presentation(&mut self) {
        let root = self.main_window_layout.get_root_mut();
        root.remove_invalid_splits();
        root.remove_invalid_tabs();
        root.run_active_tab_validation();
    }

    /// Tries to resize a window by dragging one of its edges in the given direction
    /// by (delta_x, delta_y) pixels. We climb up the dock hierarchy if the window’s
    /// immediate parent split cannot accommodate the drag.
    ///
    /// This approach ensures we don’t simultaneously borrow `self` mutably for both
    /// the layout lookups and the root_node mutations.
    pub fn adjust_window_size(
        &mut self,
        window_id: &str,
        drag_dir: &DockSplitterDragDirection,
        delta_x: i32,
        delta_y: i32,
    ) -> bool {
        self.main_window_layout
            .adjust_window_size(window_id, drag_dir, delta_x, delta_y)
    }

    pub fn reparent_window(
        &mut self,
        source_id: &str,
        target_id: &str,
        direction: DockReparentDirection,
    ) -> bool {
        let root = self.main_window_layout.get_root_mut();
        root.reparent_window(source_id, target_id, direction)
    }
}
