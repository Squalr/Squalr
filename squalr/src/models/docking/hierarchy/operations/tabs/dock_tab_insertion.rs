use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// Insert `source_node` into the Tab parent of the window at `target_path`.
    pub fn insert_into_tab_parent(
        &mut self,
        source_node: DockNode,
        target_path: &[usize],
    ) -> bool {
        let parent_slice = &target_path[..target_path.len() - 1];
        let Some(parent_node) = self.get_node_from_path_mut(parent_slice) else {
            return false;
        };

        if let DockNode::Tab { tabs, active_tab_id } = parent_node {
            tabs.push(source_node);
            if let Some(last_window_id) = tabs.last().and_then(|node| node.get_window_id()) {
                *active_tab_id = last_window_id;
            }
            true
        } else {
            false
        }
    }
}
