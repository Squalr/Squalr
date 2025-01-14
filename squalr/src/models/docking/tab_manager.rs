use crate::models::docking::dock_node::DockNode;

pub struct TabManager;

impl TabManager {
    /// A top-level method that ensures each tab node has a valid active tab.
    pub fn prepare_for_presentation(root_node: &mut DockNode) {
        Self::prepare_for_presentation_node(root_node);
    }

    /// Recursively traverse the dock tree to fix up tab states
    /// (e.g. if the active tab is hidden, pick a new one).
    fn prepare_for_presentation_node(node: &mut DockNode) {
        match node {
            DockNode::Split { children, .. } => {
                for child in children.iter_mut() {
                    Self::prepare_for_presentation_node(child);
                }
            }
            DockNode::Tab { tabs, active_tab_id, .. } => {
                // Recurse into each tab child first
                for child in tabs.iter_mut() {
                    Self::prepare_for_presentation_node(child);
                }

                // If active_tab_id is invalid or hidden, pick a new active tab.
                if !Self::is_current_active_tab_valid(tabs, active_tab_id) {
                    let new_id = Self::pick_first_visible_leaf_id(tabs);
                    if let Some(new_active_id) = new_id {
                        *active_tab_id = new_active_id;
                    } else {
                        active_tab_id.clear();
                    }
                }
            }
            DockNode::Leaf { .. } => {
                // Nothing special for leaves here
            }
        }
    }

    /// Returns whether the current `active_tab_id` in a tab node is valid + visible.
    fn is_current_active_tab_valid(
        tabs: &[DockNode],
        active_tab_id: &str,
    ) -> bool {
        if active_tab_id.is_empty() {
            return false;
        }
        // Check if there's a visible leaf with the same ID
        tabs.iter().any(|child| match child {
            DockNode::Leaf {
                window_identifier, is_visible, ..
            } => window_identifier == active_tab_id && *is_visible,
            _ => false,
        })
    }

    /// Pick the first visible leaf's ID from the tab list, if any.
    fn pick_first_visible_leaf_id(tabs: &[DockNode]) -> Option<String> {
        for child in tabs {
            match child {
                DockNode::Leaf {
                    window_identifier, is_visible, ..
                } if *is_visible => {
                    return Some(window_identifier.clone());
                }
                _ => {}
            }
        }
        None
    }
}
