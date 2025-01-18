use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// Walk and collect the identifiers of all window nodes in the entire tree.
    pub fn get_all_child_window_ids(&self) -> Vec<String> {
        let mut leaves = Vec::new();
        let mut path_stack = Vec::new();

        self.walk(&mut path_stack, &mut |node, _| {
            if let DockNode::Window { window_identifier, .. } = node {
                leaves.push(window_identifier.clone());
            }
        });

        leaves
    }
}
