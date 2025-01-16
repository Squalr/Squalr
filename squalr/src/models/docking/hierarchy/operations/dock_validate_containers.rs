use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_tree::DockTree;

/// Validates and corrects any mistakes in tab logic.
impl DockTree {
    /// Recursively clean up the docking hierarchy so that:
    /// - A Split node with only 1 child is replaced by that child.
    /// - A Tab node with only 1 child is replaced by that child.
    pub fn clean_up_hierarchy(&mut self) {
        Self::clean_up_node(&mut self.root);
    }

    /// Recursively walk the subtree and remove containers that have only 1 child.
    fn clean_up_node(dock_node: &mut DockNode) {
        match dock_node {
            // For Split nodes, clean each child first, then see if there's only one child left.
            DockNode::Split { children, .. } => {
                for child in children.iter_mut() {
                    Self::clean_up_node(&mut child.node);
                }
                // If there's exactly one child, replace self with that child.
                if children.len() == 1 {
                    let single_child = children.remove(0).node;
                    *dock_node = single_child;
                }
            }

            // For Tab nodes, clean each tab first, then see if there's only one tab left.
            DockNode::Tab { tabs, .. } => {
                for tab in tabs.iter_mut() {
                    Self::clean_up_node(tab);
                }
                // If there's exactly one tab, replace self with that tab.
                if tabs.len() == 1 {
                    let single_tab = tabs.remove(0);
                    *dock_node = single_tab;
                }
            }

            // Leaf nodes have no children to clean up.
            DockNode::Leaf { .. } => {}
        }
    }
}
