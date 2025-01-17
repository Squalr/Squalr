use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// Given a `window_id`, this method determines which sibling tab is active, if any.
    pub fn get_active_tab(
        &self,
        window_id: &str,
    ) -> String {
        // Find the path to this window.
        let path = match self.find_path_to_window_id(window_id) {
            Some(p) => p,
            None => return String::new(),
        };

        // If the path is empty, there's no parent => return fallback.
        if path.is_empty() {
            return String::new();
        }

        // Everything except the last index is the parent path.
        let (parent_path, _) = path.split_at(path.len() - 1);

        // Get the parent node from the tree, which should be a tab group if the window is part of a tab.
        if let Some(parent_node) = self.get_node_from_path(parent_path) {
            if let DockNode::Tab { active_tab_id, .. } = parent_node {
                return active_tab_id.clone();
            }
        }

        // No active tab found.
        String::new()
    }

    /// Given a `window_id`, this method determines the list of sibling tabs.
    pub fn get_sibling_tab_ids(
        &self,
        window_id: &str,
        only_visible: bool,
    ) -> Vec<String> {
        // Find the path to this window.
        let path = match self.find_path_to_window_id(window_id) {
            Some(p) => p,
            None => return vec![],
        };

        // If the path is empty, there's no parent => return fallback.
        if path.is_empty() {
            return vec![];
        }

        // Everything except the last index is the parent path.
        let (parent_path, _) = path.split_at(path.len() - 1);

        // Get the parent node from the tree, which should be a tab group if the window is part of a tab.
        if let Some(parent_node) = self.get_node_from_path(parent_path) {
            if let DockNode::Tab { tabs, .. } = parent_node {
                // Collect all siblings in this Tab, filtering by visibility if requested.
                let mut siblings = Vec::new();
                for tab_node in tabs {
                    if let DockNode::Window {
                        window_identifier, is_visible, ..
                    } = tab_node
                    {
                        if !only_visible || *is_visible {
                            siblings.push(window_identifier.clone());
                        }
                    }
                }
                return siblings;
            }
        }

        // No siblings found.
        vec![]
    }
}
