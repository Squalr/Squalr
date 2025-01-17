use crate::models::docking::hierarchy::dock_node::DockNode;

/// Validates and corrects any mistakes in tab logic.
impl DockNode {
    /// Recursively clean up the docking hierarchy so that:
    /// - A Split node with only 1 child is replaced by that child.
    /// - A Tab node with only 1 child is replaced by that child.
    pub fn remove_invalid_containers(&mut self) {
        Self::remove_invalid_containers_internal(self);
    }

    /// Recursively walk the subtree and remove containers that have only 1 child.
    fn remove_invalid_containers_internal(dock_node: &mut DockNode) {
        match dock_node {
            // For Split nodes, clean each child first, then see if there's only one child left.
            DockNode::Split { children, .. } => {
                for child in children.iter_mut() {
                    Self::remove_invalid_containers_internal(&mut child.node);
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
                    Self::remove_invalid_containers_internal(tab);
                }
                // If there's exactly one tab, replace self with that tab.
                if tabs.len() == 1 {
                    let single_tab = tabs.remove(0);
                    *dock_node = single_tab;
                }
            }

            // Leaf nodes have no children to clean up.
            DockNode::Window { .. } => {}
        }
    }
}
