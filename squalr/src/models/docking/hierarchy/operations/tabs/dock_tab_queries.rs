use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// Returns `true` if the window belongs to a multi-tab group.
    pub fn is_window_in_tab_group(
        &self,
        window_id: &str,
    ) -> bool {
        self.find_parent_tab_path(window_id).is_some()
    }

    /// Returns `true` if both windows belong to the same multi-tab group.
    pub fn are_windows_in_same_tab_group(
        &self,
        source_window_id: &str,
        target_window_id: &str,
    ) -> bool {
        let Some(source_parent_tab_path) = self.find_parent_tab_path(source_window_id) else {
            return false;
        };
        let Some(target_parent_tab_path) = self.find_parent_tab_path(target_window_id) else {
            return false;
        };

        source_parent_tab_path == target_parent_tab_path
    }

    /// Given a `window_id`, this method determines which sibling tab is active, if any.
    pub fn get_active_tab(
        &self,
        window_id: &str,
    ) -> String {
        // Find the path to this window.
        let path = match self.find_path_to_window_id(window_id) {
            Some(path) => path,
            None => return String::new(),
        };

        // If the path is empty, there's no parent => return the window itself.
        if path.is_empty() {
            return window_id.to_string();
        }

        // Everything except the last index is the parent path.
        let (parent_path, _) = path.split_at(path.len() - 1);

        // Get the parent node from the tree, which should be a tab group if the window is part of a tab.
        if let Some(parent_node) = self.get_node_from_path(parent_path) {
            if let DockNode::Tab { active_tab_id, .. } = parent_node {
                return active_tab_id.clone();
            }
        }

        // No active tab found, fall back to the window itself.
        return window_id.to_string();
    }

    /// Given a `window_id`, this method determines the list of sibling tabs.
    pub fn get_sibling_tab_ids(
        &self,
        window_id: &str,
        only_visible: bool,
    ) -> Vec<String> {
        // Find the path to this window.
        let path = match self.find_path_to_window_id(window_id) {
            Some(path) => path,
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
                let mut siblings = vec![];

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

    fn find_parent_tab_path(
        &self,
        window_id: &str,
    ) -> Option<Vec<usize>> {
        let window_path = self.find_path_to_window_id(window_id)?;
        let parent_path_index = window_path.len().checked_sub(1)?;
        let parent_tab_path = window_path.get(..parent_path_index)?;

        match self.get_node_from_path(parent_tab_path) {
            Some(DockNode::Tab { tabs, .. }) if tabs.len() > 1 => Some(parent_tab_path.to_vec()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DockNode;
    use crate::models::docking::hierarchy::types::{dock_split_child::DockSplitChild, dock_split_direction::DockSplitDirection};

    fn build_tabbed_layout() -> DockNode {
        DockNode::Split {
            direction: DockSplitDirection::VerticalDivider,
            children: vec![
                DockSplitChild {
                    node: DockNode::Tab {
                        tabs: vec![
                            DockNode::Window {
                                window_identifier: "tab_a".to_string(),
                                is_visible: true,
                            },
                            DockNode::Window {
                                window_identifier: "tab_b".to_string(),
                                is_visible: true,
                            },
                        ],
                        active_tab_id: "tab_a".to_string(),
                    },
                    ratio: 0.5,
                },
                DockSplitChild {
                    node: DockNode::Window {
                        window_identifier: "solo".to_string(),
                        is_visible: true,
                    },
                    ratio: 0.5,
                },
            ],
        }
    }

    #[test]
    fn detects_multi_tab_membership() {
        let dock_root = build_tabbed_layout();

        assert!(dock_root.is_window_in_tab_group("tab_a"));
        assert!(dock_root.is_window_in_tab_group("tab_b"));
        assert!(!dock_root.is_window_in_tab_group("solo"));
    }

    #[test]
    fn detects_shared_tab_group() {
        let dock_root = build_tabbed_layout();

        assert!(dock_root.are_windows_in_same_tab_group("tab_a", "tab_b"));
        assert!(!dock_root.are_windows_in_same_tab_group("tab_a", "solo"));
        assert!(!dock_root.are_windows_in_same_tab_group("solo", "solo"));
    }
}
