use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_tree::DockTree;

pub struct DockingTabManager;

impl DockingTabManager {
    /// Activate a window in its tab (if parent is a tab).
    pub fn select_tab_by_leaf_id(
        tree: &mut DockTree,
        leaf_id: &str,
    ) -> bool {
        let path = match tree.find_leaf_path(leaf_id) {
            Some(path) => path,
            None => return false,
        };
        if path.is_empty() {
            return false;
        }
        let (parent_slice, _) = path.split_at(path.len() - 1);

        if let Some(parent_node) = tree.get_node_mut(parent_slice) {
            if let DockNode::Tab { active_tab_id, .. } = parent_node {
                *active_tab_id = leaf_id.to_owned();
                return true;
            }
        }
        false
    }

    /// Given a `leaf_id` and a `DockTree`, this method determines the list of sibling tabs, as well as which one is active.
    pub fn get_siblings_and_active_tab(
        tree: &DockTree,
        leaf_id: &str,
    ) -> (Vec<String>, String) {
        // Find the path to this leaf.
        let path = match tree.find_leaf_path(leaf_id) {
            Some(p) => p,
            None => return (Vec::new(), leaf_id.to_owned()),
        };

        // If the path is empty, there's no parent => return fallback.
        if path.is_empty() {
            return (Vec::new(), leaf_id.to_owned());
        }

        // Everything except the last index is the parent path.
        let (parent_path, _) = path.split_at(path.len() - 1);

        // Get the parent node from the tree
        if let Some(parent_node) = tree.get_node(parent_path) {
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
        (Vec::new(), leaf_id.to_owned())
    }

    /// A top-level method that ensures each tab node has a valid active tab.
    pub fn run_tab_validation(node: &mut DockNode) {
        match node {
            DockNode::Split { children, .. } => {
                for child in children.iter_mut() {
                    Self::run_tab_validation(&mut child.node);
                }
            }
            DockNode::Tab { tabs, active_tab_id, .. } => {
                // Recurse into each tab child first
                for child in tabs.iter_mut() {
                    Self::run_tab_validation(child);
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
            DockNode::Leaf { .. } => {}
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
