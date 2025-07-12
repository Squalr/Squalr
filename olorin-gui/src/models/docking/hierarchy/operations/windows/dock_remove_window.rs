use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// Removes a window by path and returns the removed `DockNode`.
    pub fn remove_window_by_path(
        &mut self,
        window_path: &[usize],
    ) -> Option<DockNode> {
        // We expect window_path not to be empty.
        let (child_index, parent_slice) = window_path.split_last()?;
        let child_index = *child_index;

        let parent_node = self.get_node_from_path_mut(parent_slice)?;
        match parent_node {
            DockNode::Split { children, .. } => {
                if child_index < children.len() {
                    let removed = children.remove(child_index);
                    Some(removed.node)
                } else {
                    None
                }
            }
            DockNode::Tab { tabs, .. } => {
                if child_index < tabs.len() {
                    Some(tabs.remove(child_index))
                } else {
                    None
                }
            }
            DockNode::Window { .. } => {
                // This should not be possible, but just replace the window with the parent in this case.
                let old_root = std::mem::replace(parent_node, DockNode::default());
                Some(old_root)
            }
        }
    }
}
