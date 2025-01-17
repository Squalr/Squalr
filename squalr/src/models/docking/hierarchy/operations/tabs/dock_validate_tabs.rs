use crate::models::docking::hierarchy::dock_node::DockNode;

/// Validates and corrects any mistakes in tab containers.
impl DockNode {
    /// Recursively clean up the docking hierarchy so that a tab node with only 1 child is replaced by that child.
    pub fn remove_invalid_tabs(&mut self) {
        Self::remove_invalid_tabs_internal(self);
    }

    /// Recursively walk the subtree and remove tabs that have only 1 child.
    fn remove_invalid_tabs_internal(dock_node: &mut DockNode) {
        match dock_node {
            DockNode::Split { .. } => {}
            // Clean each tab first, then see if there's only one tab left.
            DockNode::Tab { tabs, .. } => {
                for tab in tabs.iter_mut() {
                    Self::remove_invalid_tabs_internal(tab);
                }
                // If there's exactly one tab, replace self with that tab.
                if tabs.len() == 1 {
                    let single_tab = tabs.remove(0);
                    *dock_node = single_tab;
                }
            }
            DockNode::Window { .. } => {}
        }
    }
}
