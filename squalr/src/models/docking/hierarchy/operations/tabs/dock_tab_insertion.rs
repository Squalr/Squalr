use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// Reparents `source_node` into the same tab group as `target_path`.
    /// If the target’s parent is a Tab, we just insert into that Tab’s list.
    /// Otherwise, if the target itself is a Window, we convert that Window into a Tab.
    /// If the target is a Split, we wrap it in a Tab (but remember, no splits in tabs).
    /// Returns `true` on success.
    pub fn reparent_as_tab(
        &mut self,
        source_node: DockNode,
        target_path: &[usize],
    ) -> bool {
        // If the target is a Window with a parent Tab, just insert.
        if self.is_window_with_tab_parent(target_path) {
            return self.insert_into_tab_parent(source_node, target_path);
        }

        // Otherwise, we get the actual target node.
        let Some(target_node) = self.get_node_from_path_mut(target_path) else {
            return false;
        };

        match target_node {
            DockNode::Tab { tabs, active_tab_id } => {
                // Just push the new node into the existing tab.
                tabs.push(source_node);
                if let Some(last_window_id) = tabs.last().and_then(|node| node.get_window_id()) {
                    *active_tab_id = last_window_id;
                }
                true
            }
            DockNode::Window { .. } => {
                // Convert Window -> Tab with 2 children
                let new_tab = Self::convert_window_to_tab(std::mem::take(target_node), source_node);
                *target_node = new_tab;
                true
            }
            DockNode::Split { .. } => {
                // Tabs cannot contain splits as children.
                false
            }
        }
    }

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

    /// Helper: check if `target_path` is a Window with a parent Tab node.
    fn is_window_with_tab_parent(
        &self,
        target_path: &[usize],
    ) -> bool {
        if target_path.is_empty() {
            return false;
        }
        let child_node = self.get_node_from_path(target_path);
        if !matches!(child_node, Some(DockNode::Window { .. })) {
            return false;
        }
        let parent_slice = &target_path[..target_path.len() - 1];
        match self.get_node_from_path(parent_slice) {
            Some(DockNode::Tab { .. }) => true,
            _ => false,
        }
    }

    /// Helper: Convert a single Window node + an extra node into a Tab node with both children.
    fn convert_window_to_tab(
        window: DockNode,
        other: DockNode,
    ) -> DockNode {
        // We assume `window` is actually a Window. If not, be defensive.
        let window_id = window.get_window_id().unwrap_or_default();
        let other_id = other.get_window_id().unwrap_or_default();
        DockNode::Tab {
            tabs: vec![window, other],
            active_tab_id: other_id.is_empty().then(|| window_id).unwrap_or(other_id),
        }
    }
}
