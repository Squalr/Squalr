use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// Find the path (series of child indices) to a leaf node by ID. Returns `None` if not found.
    pub fn find_path_to_window_id(
        &self,
        leaf_id: &str,
    ) -> Option<Vec<usize>> {
        let mut path_stack = Vec::new();
        let mut result = None;

        self.walk(&mut path_stack, &mut |node, current_path| {
            if let DockNode::Leaf { window_identifier, .. } = node {
                if window_identifier == leaf_id {
                    result = Some(current_path.to_vec());
                }
            }
        });

        result
    }

    /// Collect the identifiers of all leaf nodes in the entire tree.
    pub fn get_all_child_window_ids(&self) -> Vec<String> {
        let mut leaves = Vec::new();
        let mut path_stack = Vec::new();

        self.walk(&mut path_stack, &mut |node, _| {
            if let DockNode::Leaf { window_identifier, .. } = node {
                leaves.push(window_identifier.clone());
            }
        });

        leaves
    }
}
