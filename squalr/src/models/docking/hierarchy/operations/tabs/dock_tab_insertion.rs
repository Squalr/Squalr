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
        // If the target's parent is a Tab, we insert into the parent of `target_path`
        if let Some(parent) = self.get_node_from_path_mut(&target_path[..target_path.len().saturating_sub(1)]) {
            if let DockNode::Tab { tabs, active_tab_id } = parent {
                Self::insert_tab_child(tabs, active_tab_id, source_node);
                return true;
            }
        }

        // Otherwise, get or convert the target node to a tab.
        let Some(target_node) = self.get_node_from_path_mut(target_path) else {
            return false;
        };

        match target_node {
            DockNode::Tab { tabs, active_tab_id } => {
                Self::insert_tab_child(tabs, active_tab_id, source_node);
                true
            }
            DockNode::Window { .. } => {
                let new_tab = Self::convert_window_to_tab(std::mem::take(target_node), source_node);
                *target_node = new_tab;
                true
            }
            DockNode::Split { .. } => false,
        }
    }

    /// Inserts a new child window into an existing tab group.
    fn insert_tab_child(
        tabs: &mut Vec<DockNode>,
        active_tab_id: &mut String,
        child: DockNode,
    ) {
        tabs.push(child);
        if let Some(last_window_id) = tabs.last().and_then(|n| n.get_window_id()) {
            *active_tab_id = last_window_id;
        }
    }

    /// Convert a single Window node + an extra node into a Tab node with both children.
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
