use crate::models::docking::drag_drop::dock_drag_state::DockDragState;
use crate::models::docking::drag_drop::dock_drop_zone::{DockDragOverlay, DockDropTarget};
use crate::models::docking::hierarchy::dock_layout::DockLayout;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_reparent_direction::DockReparentDirection;
use crate::models::docking::hierarchy::types::dock_splitter_drag_direction::DockSplitterDragDirection;
#[cfg(not(test))]
use crate::models::docking::settings::dockable_window_settings::DockableWindowSettings;
use epaint::{Pos2, Rect, pos2, vec2};

/// Handles a `DockLayout`, which contains a root `DockNode` and manages its layout.
pub struct DockingManager {
    pub main_window_layout: DockLayout,
    active_drag_state: Option<DockDragState>,
}

/// Contains various helper functions to manage an underlying docking hierarchy and its layout.
impl DockingManager {
    pub fn new(root_node: DockNode) -> Self {
        Self {
            main_window_layout: DockLayout::new(root_node),
            active_drag_state: None,
        }
    }

    /// Replace the entire root node in the root_node and saves the new root to disk.
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

    /// Get the root node of the docking main_window_layout (mutable).
    pub fn get_root_mut(&mut self) -> &mut DockNode {
        self.main_window_layout.get_root_mut()
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

    /// Given a `window_id`, this method determines which sibling tab is active, if any.
    pub fn get_active_tab(
        &self,
        window_id: &str,
    ) -> String {
        self.main_window_layout.get_root().get_active_tab(window_id)
    }

    /// Given a `window_id`, this method determines the list of sibling tabs.
    pub fn get_sibling_tab_ids(
        &self,
        window_id: &str,
        only_visible: bool,
    ) -> Vec<String> {
        self.main_window_layout
            .get_root()
            .get_sibling_tab_ids(window_id, only_visible)
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
        if source_id == target_id && direction == DockReparentDirection::Tab {
            return true;
        }

        let root = self.main_window_layout.get_root_mut();
        let reparent_succeeded = root.reparent_window(source_id, target_id, direction);

        if reparent_succeeded {
            self.persist_layout();
        }

        reparent_succeeded
    }

    pub fn begin_drag(
        &mut self,
        source_window_identifier: &str,
        pointer_press_origin: Pos2,
    ) -> bool {
        if self.get_node_by_id(source_window_identifier).is_none() {
            return false;
        }

        self.active_drag_state = Some(DockDragState::new(source_window_identifier.to_string(), pointer_press_origin));

        true
    }

    pub fn update_drag_pointer_position(
        &mut self,
        current_pointer_position: Option<Pos2>,
    ) {
        if let Some(active_drag_state) = self.active_drag_state.as_mut() {
            active_drag_state.update_pointer_position(current_pointer_position);
        }
    }

    pub fn active_dragged_window_id(&self) -> Option<&str> {
        self.active_drag_state
            .as_ref()
            .map(|active_drag_state| active_drag_state.source_window_identifier.as_str())
    }

    pub fn get_drag_overlay(
        &self,
        root_screen_rect: Rect,
    ) -> Option<DockDragOverlay> {
        let active_drag_state = self.active_drag_state.as_ref()?;

        if !active_drag_state.is_drop_overlay_visible() {
            return None;
        }

        let current_pointer_position = active_drag_state.current_pointer_position()?;
        let target_window_rectangles = self.collect_rendered_window_rectangles(root_screen_rect);

        Some(DockDragOverlay::from_window_rectangles(&target_window_rectangles, current_pointer_position))
    }

    pub fn finish_drag(
        &mut self,
        root_screen_rect: Rect,
    ) -> bool {
        let Some(active_drag_state) = self.active_drag_state.clone() else {
            return false;
        };

        let maybe_drop_target = self.resolve_drop_target(root_screen_rect);
        self.active_drag_state = None;

        if !active_drag_state.is_drop_overlay_visible() {
            return false;
        }

        let Some(dock_drop_target) = maybe_drop_target else {
            return false;
        };

        self.reparent_window(
            &active_drag_state.source_window_identifier,
            &dock_drop_target.target_window_identifier,
            dock_drop_target.direction,
        )
    }

    fn resolve_drop_target(
        &self,
        root_screen_rect: Rect,
    ) -> Option<DockDropTarget> {
        self.get_drag_overlay(root_screen_rect)
            .and_then(|dock_drag_overlay| dock_drag_overlay.hovered_drop_target)
    }

    fn collect_rendered_window_rectangles(
        &self,
        root_screen_rect: Rect,
    ) -> Vec<(String, Rect)> {
        self.get_all_child_window_ids()
            .into_iter()
            .filter(|window_identifier| self.get_active_tab(window_identifier) == *window_identifier)
            .filter(|window_identifier| {
                self.get_node_by_id(window_identifier)
                    .is_some_and(|dock_node| dock_node.is_visible())
            })
            .filter_map(|window_identifier| {
                self.find_window_rect(&window_identifier)
                    .map(|(x, y, width, height)| {
                        (
                            window_identifier,
                            Rect::from_min_size(pos2(root_screen_rect.min.x + x, root_screen_rect.min.y + y), vec2(width, height)),
                        )
                    })
            })
            .collect()
    }

    #[cfg(not(test))]
    fn persist_layout(&self) {
        DockableWindowSettings::set_dock_layout_settings(self.main_window_layout.get_root());
    }

    #[cfg(test)]
    fn persist_layout(&self) {}
}
