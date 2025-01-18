use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// Find the path (series of child indices) to a window node by ID. Returns `None` if not found.
    pub fn find_path_to_window_id(
        &self,
        window_id: &str,
    ) -> Option<Vec<usize>> {
        let mut path_stack = Vec::new();
        let mut result = None;

        self.walk(&mut path_stack, &mut |node, current_path| {
            if let DockNode::Window { window_identifier, .. } = node {
                if window_identifier == window_id {
                    result = Some(current_path.to_vec());
                }
            }
        });

        result
    }

    /// Find the path (series of child indices) to the container (parent) of a window node by ID.
    /// Returns `None` if not found or if at the top level.
    pub fn find_path_to_window_container(
        &self,
        window_id: &str,
    ) -> Option<Vec<usize>> {
        // Reuse the existing logic to get the path to the window
        let path_to_window = self.find_path_to_window_id(window_id)?;

        // If there's at least one element, drop the last index to get the container path
        if path_to_window.is_empty() {
            None
        } else {
            let mut container_path = path_to_window.clone();
            container_path.pop();
            Some(container_path)
        }
    }
}
