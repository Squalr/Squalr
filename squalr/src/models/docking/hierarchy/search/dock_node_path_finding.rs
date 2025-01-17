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
}
