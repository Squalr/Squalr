use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_reparent_direction::DockReparentDirection;

impl DockNode {
    /// High-level entry point to re-parent (move) a window node `source_window_id`
    /// relative to a window node `target_window_id` in `direction`.
    ///
    /// Returns `true` if successful, or `false` if something went wrong
    /// (e.g. source/target not found, re-parent onto itself, etc.).
    pub fn reparent_window(
        &mut self,
        source_window_id: &str,
        target_window_id: &str,
        direction: DockReparentDirection,
    ) -> bool {
        // Find the path to the target. If the target is ourself, then we actually want to redirect to the container.
        let target_path = match source_window_id == target_window_id {
            true => match self.find_path_to_window_container(target_window_id) {
                Some(path) => path,
                None => return false,
            },
            false => match self.find_path_to_window_id(target_window_id) {
                Some(path) => path,
                None => return false,
            },
        };

        // Find the path to the source window being moved.
        let source_window_path = match self.find_path_to_window_id(source_window_id) {
            Some(path) => path,
            None => return false,
        };

        // Remove the source node from the tree.
        let source_node = match self.remove_window_by_path(&source_window_path) {
            Some(node) => node,
            None => return false,
        };

        // Delegate to subroutines based on direction.
        match direction {
            DockReparentDirection::Tab => self.reparent_as_tab(source_node, &target_path),
            DockReparentDirection::Left | DockReparentDirection::Right | DockReparentDirection::Top | DockReparentDirection::Bottom => {
                self.reparent_as_split_sibling(source_node, &target_path, direction)
            }
        }
    }
}
