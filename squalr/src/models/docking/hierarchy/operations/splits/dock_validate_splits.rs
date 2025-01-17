use crate::models::docking::hierarchy::dock_node::DockNode;

/// Validates and corrects any mistakes in split splits.
impl DockNode {
    /// Recursively clean up the docking hierarchy so that a split node with only 1 child is replaced by that child.
    pub fn remove_invalid_splits(&mut self) {
        Self::remove_invalid_splits_internal(self);
    }

    /// Recursively walk the subtree and remove splits that have only 1 child.
    fn remove_invalid_splits_internal(dock_node: &mut DockNode) {
        match dock_node {
            // For Split nodes, clean each child first, then see if there's only one child left.
            DockNode::Split { children, .. } => {
                for child in children.iter_mut() {
                    Self::remove_invalid_splits_internal(&mut child.node);
                }
                // If there's exactly one child, replace self with that child.
                if children.len() == 1 {
                    let single_child = children.remove(0).node;
                    *dock_node = single_child;
                }
            }
            DockNode::Tab { .. } => {}
            DockNode::Window { .. } => {}
        }
    }
}
