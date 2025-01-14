use crate::models::docking::dock_node::DockNode;
use crate::models::docking::dock_tree::DockTree;
use crate::models::docking::layout::docking_layout::DockingLayout;
use crate::models::docking::tab_manager::TabManager;

pub struct DockingManager {
    pub tree: DockTree,
    pub layout: DockingLayout,
}

impl DockingManager {
    pub fn new(root_node: DockNode) -> Self {
        Self {
            tree: DockTree::new(root_node),
            layout: DockingLayout::new(),
        }
    }

    /// Replace the entire root node in the tree.
    pub fn set_root(
        &mut self,
        new_root: DockNode,
    ) {
        self.tree.replace_root(new_root);
    }

    /// Just expose the root if needed.
    pub fn get_root(&self) -> &DockNode {
        &self.tree.root
    }

    pub fn get_layout(&self) -> &DockingLayout {
        &self.layout
    }

    pub fn get_layout_mut(&mut self) -> &mut DockingLayout {
        &mut self.layout
    }

    /// Retrieve a node by ID (immutable).
    pub fn get_node_by_id(
        &self,
        identifier: &str,
    ) -> Option<&DockNode> {
        let path = self.tree.find_leaf_path(identifier)?;
        self.tree.get_node(&path)
    }

    /// Retrieve a node by ID (mutable).
    pub fn get_node_by_id_mut(
        &mut self,
        identifier: &str,
    ) -> Option<&mut DockNode> {
        let path = self.tree.find_leaf_path(identifier)?;
        self.tree.get_node_mut(&path)
    }

    /// Collect all leaf IDs from the tree.
    pub fn get_all_leaves(&self) -> Vec<String> {
        self.tree.get_all_leaves()
    }

    /// Find the bounding rectangle for a particular leaf.
    pub fn find_window_rect(
        &self,
        leaf_id: &str,
    ) -> Option<(f32, f32, f32, f32)> {
        self.layout.find_window_rect(&self.tree, leaf_id)
    }

    /// Example: resize a window by adjusting its ratio
    pub fn resize_window(
        &mut self,
        leaf_id: &str,
        new_ratio: f32,
    ) -> bool {
        let path = match self.tree.find_leaf_path(leaf_id) {
            Some(path) => path,
            None => return false,
        };

        if let Some(node) = self.tree.get_node_mut(&path) {
            node.set_ratio(new_ratio);
        } else {
            return false;
        }

        // If there's a parent with exactly two children, adjust the sibling.
        if !path.is_empty() {
            let (parent_slice, &[leaf_index]) = path.split_at(path.len() - 1) else {
                return true;
            };

            if let Some(parent_node) = self.tree.get_node_mut(parent_slice) {
                match parent_node {
                    DockNode::Split { children, .. } if children.len() == 2 => {
                        let sibling_idx = if leaf_index == 0 { 1 } else { 0 };
                        let sibling_ratio = (1.0 - new_ratio).clamp(0.0, 1.0);
                        children[sibling_idx].set_ratio(sibling_ratio);
                    }
                    DockNode::Tab { tabs, .. } if tabs.len() == 2 => {
                        let sibling_idx = if leaf_index == 0 { 1 } else { 0 };
                        let sibling_ratio = (1.0 - new_ratio).clamp(0.0, 1.0);
                        tabs[sibling_idx].set_ratio(sibling_ratio);
                    }
                    _ => {}
                }
            }
        }

        true
    }

    /// Activate a leaf in its tab (if parent is a tab).
    pub fn select_tab_by_leaf_id(
        &mut self,
        leaf_id: &str,
    ) -> bool {
        let path = match self.tree.find_leaf_path(leaf_id) {
            Some(path) => path,
            None => return false,
        };
        if path.is_empty() {
            return false;
        }
        let (parent_slice, _) = path.split_at(path.len() - 1);

        if let Some(parent_node) = self.tree.get_node_mut(parent_slice) {
            if let DockNode::Tab { active_tab_id, .. } = parent_node {
                *active_tab_id = leaf_id.to_owned();
                return true;
            }
        }
        false
    }

    pub fn get_siblings_and_active_tab(
        &self,
        leaf_id: &str,
    ) -> (Vec<String>, String) {
        // Find the path to this leaf
        let path = match self.tree.find_leaf_path(leaf_id) {
            Some(p) => p,
            None => return (Vec::new(), leaf_id.to_owned()),
        };

        // If the path is empty, there's no parent => return fallback
        if path.is_empty() {
            return (Vec::new(), leaf_id.to_owned());
        }

        // Everything except the last index is the parent path
        let (parent_path, _) = path.split_at(path.len() - 1);

        // Get the parent node from the tree
        if let Some(parent_node) = self.tree.get_node(parent_path) {
            if let DockNode::Tab { tabs, active_tab_id, .. } = parent_node {
                // Collect all visible siblings in this Tab
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

        // If not found or parent not a Tab, fallback
        (Vec::new(), leaf_id.to_owned())
    }

    /// Prepare for presentation by fixing up tabs, etc.
    pub fn prepare_for_presentation(&mut self) {
        TabManager::prepare_for_presentation(&mut self.tree.root);
    }
}
