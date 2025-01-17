use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// Return an immutable reference to the node at the specified path. Returns `None` if the path is invalid.
    pub fn get_node_from_path(
        &self,
        path: &[usize],
    ) -> Option<&DockNode> {
        let mut current = self;
        for &idx in path {
            match current {
                DockNode::Split { children, .. } => {
                    if idx >= children.len() {
                        return None;
                    }
                    current = &children[idx].node;
                }
                DockNode::Tab { tabs, .. } => {
                    if idx >= tabs.len() {
                        return None;
                    }
                    current = &tabs[idx];
                }
                DockNode::Window { .. } => {
                    // The path goes deeper than a leaf => invalid
                    return None;
                }
            }
        }
        Some(current)
    }

    /// Return a mutable reference to the node at the specified path.
    /// Returns `None` if the path is invalid or tries to go beyond a leaf.
    pub fn get_node_from_path_mut(
        &mut self,
        path: &[usize],
    ) -> Option<&mut DockNode> {
        let mut current = self;
        for &idx in path {
            match current {
                DockNode::Split { children, .. } => {
                    if idx >= children.len() {
                        return None;
                    }
                    current = &mut children[idx].node;
                }
                DockNode::Tab { tabs, .. } => {
                    if idx >= tabs.len() {
                        return None;
                    }
                    current = &mut tabs[idx];
                }
                DockNode::Window { .. } => {
                    // The path goes deeper than a leaf => invalid
                    return None;
                }
            }
        }
        Some(current)
    }

    /// Walk and collect the identifiers of all leaf nodes in the entire tree.
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
