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
                    // The path goes deeper than a window => invalid
                    return None;
                }
            }
        }
        Some(current)
    }

    /// Return a mutable reference to the node at the specified path.
    /// Returns `None` if the path is invalid or tries to go beyond a window.
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
                    // The path goes deeper than a window => invalid
                    return None;
                }
            }
        }
        Some(current)
    }
}
