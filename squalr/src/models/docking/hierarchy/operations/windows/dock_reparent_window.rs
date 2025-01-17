use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_reparent_direction::DockReparentDirection;

impl DockNode {
    /// High-level entry point to re-parent (move) a window node `source_id`
    /// relative to a window node `target_id` in `direction`.
    ///
    /// Returns `true` if successful, or `false` if something went wrong
    /// (e.g. source/target not found, re-parent onto itself, etc.).
    pub fn reparent_window(
        &mut self,
        source_id: &str,
        target_id: &str,
        direction: DockReparentDirection,
    ) -> bool {
        // Find both source + target.
        let source_path = match self.find_path_to_window_id(source_id) {
            Some(path) => path,
            None => return false,
        };
        let target_path = match self.find_path_to_window_id(target_id) {
            Some(path) => path,
            None => return false,
        };
        if source_path == target_path {
            // Disallow re-parenting onto itself.
            return false;
        }

        // Remove the source node from the tree.
        let source_node = match self.remove_window_by_path(&source_path) {
            Some(node) => node,
            None => return false,
        };

        // Re-fetch the target path in case it changed.
        let new_target_path = match self.find_path_to_window_id(target_id) {
            Some(path) => path,
            None => {
                // If target somehow disappeared, bail (Optionally re-insert source in old place, or just discard).
                return false;
            }
        };

        // 3) Delegate to subroutines based on direction.
        match direction {
            DockReparentDirection::Tab => self.reparent_as_tab(source_node, &new_target_path),
            DockReparentDirection::Left | DockReparentDirection::Right | DockReparentDirection::Top | DockReparentDirection::Bottom => {
                self.reparent_as_split_sibling(source_node, &new_target_path, direction)
            }
        }
    }
}
