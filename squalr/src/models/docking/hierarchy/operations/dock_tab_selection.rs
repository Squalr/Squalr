use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// Activate a window in its tab (if parent is a tab).
    pub fn select_tab_by_window_id(
        &mut self,
        window_id: &str,
    ) -> bool {
        let path = match self.find_path_to_window_id(window_id) {
            Some(path) => path,
            None => return false,
        };
        if path.is_empty() {
            return false;
        }
        let (parent_slice, _) = path.split_at(path.len() - 1);

        if let Some(parent_node) = self.get_node_from_path_mut(parent_slice) {
            if let DockNode::Tab { active_tab_id, .. } = parent_node {
                *active_tab_id = window_id.to_owned();
                return true;
            }
        }
        false
    }

    /// Given a `window_id` and a `DockTree`, this method determines the list of sibling tabs, as well as which one is active.
    pub fn get_siblings_and_active_tab(
        &self,
        window_id: &str,
    ) -> (Vec<String>, String) {
        // Find the path to this leaf.
        let path = match self.find_path_to_window_id(window_id) {
            Some(p) => p,
            None => return (Vec::new(), window_id.to_owned()),
        };

        // If the path is empty, there's no parent => return fallback.
        if path.is_empty() {
            return (Vec::new(), window_id.to_owned());
        }

        // Everything except the last index is the parent path.
        let (parent_path, _) = path.split_at(path.len() - 1);

        // Get the parent node from the tree.
        if let Some(parent_node) = self.get_node_from_path(parent_path) {
            if let DockNode::Tab { tabs, active_tab_id, .. } = parent_node {
                // Collect all visible siblings in this Tab.
                let mut siblings = Vec::new();
                for tab_node in tabs {
                    if let DockNode::Leaf {
                        window_identifier, is_visible, ..
                    } = tab_node
                    {
                        if *is_visible {
                            siblings.push(window_identifier.clone());
                        }
                    }
                }
                return (siblings, active_tab_id.clone());
            }
        }

        // Default to returning self if no siblings or parent found.
        (Vec::new(), window_id.to_owned())
    }
}
