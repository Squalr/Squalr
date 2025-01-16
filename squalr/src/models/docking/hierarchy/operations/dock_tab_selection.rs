use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_tree::DockTree;

impl DockTree {
    /// Activate a window in its tab (if parent is a tab).
    pub fn select_tab_by_leaf_id(
        &mut self,
        leaf_id: &str,
    ) -> bool {
        let path = match self.find_leaf_path(leaf_id) {
            Some(path) => path,
            None => return false,
        };
        if path.is_empty() {
            return false;
        }
        let (parent_slice, _) = path.split_at(path.len() - 1);

        if let Some(parent_node) = self.get_node_mut(parent_slice) {
            if let DockNode::Tab { active_tab_id, .. } = parent_node {
                *active_tab_id = leaf_id.to_owned();
                return true;
            }
        }
        false
    }
}
