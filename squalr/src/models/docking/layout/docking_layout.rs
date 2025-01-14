use crate::models::docking::builder::dock_builder::DockBuilder;
use crate::models::docking::layout::dock_node::DockNode;

#[derive(Debug)]
pub struct DockingLayout {
    pub root: DockNode,
    pub available_width: f32,
    pub available_height: f32,
}

impl DockingLayout {
    /// Create a new layout from a builder directly.
    pub fn from_root(root: DockNode) -> Self {
        Self {
            root: root,
            available_width: 0.0,
            available_height: 0.0,
        }
    }

    /// Create a new layout from a builder directly.
    pub fn from_builder(builder: DockBuilder) -> Self {
        Self {
            root: builder.build(),
            available_width: 0.0,
            available_height: 0.0,
        }
    }

    /// Set the layout's available width and height.
    pub fn set_available_size(
        &mut self,
        width: f32,
        height: f32,
    ) {
        self.available_width = width;
        self.available_height = height;
    }

    /// Set the layout's available width.
    pub fn set_available_width(
        &mut self,
        width: f32,
    ) {
        self.available_width = width;
    }

    /// Set the layout's available height.
    pub fn set_available_height(
        &mut self,
        height: f32,
    ) {
        self.available_height = height;
    }

    pub fn find_node_by_id(
        &self,
        window_id: &str,
    ) -> Option<(&DockNode, Vec<usize>)> {
        let path = self.root.find_path_to_leaf(window_id)?;
        let node = Self::get_node(&self.root, &path);
        Some((node, path))
    }

    pub fn find_node_by_id_mut(
        &mut self,
        window_id: &str,
    ) -> Option<(&mut DockNode, Vec<usize>)> {
        let path = self.root.find_path_to_leaf(window_id)?;
        let node = Self::get_node_mut(&mut self.root, &path);
        Some((node, path))
    }

    pub fn get_node_by_id(
        &self,
        identifier: &str,
    ) -> Option<&DockNode> {
        self.find_node_by_id(identifier).map(|(node, _)| node)
    }

    pub fn get_node_by_id_mut(
        &mut self,
        identifier: &str,
    ) -> Option<&mut DockNode> {
        self.find_node_by_id_mut(identifier).map(|(node, _)| node)
    }

    pub fn get_root(&self) -> &DockNode {
        &self.root
    }

    pub fn set_root(
        &mut self,
        root: &DockNode,
    ) {
        self.root = root.clone();
    }

    pub fn get_all_leaves(&self) -> Vec<String> {
        let mut leaves = Vec::new();
        self.root.collect_leaves(&mut leaves);
        leaves
    }

    /// Attempt to resize a leaf node or tab node by adjusting its ratio.
    /// This only works if the node has exactly one sibling (i.e., two total siblings).
    pub fn resize_window(
        &mut self,
        window_id: &str,
        new_ratio: f32,
    ) -> bool {
        // Single pass: get the node + path
        let Some((node_ref, path)) = self.find_node_by_id_mut(window_id) else {
            return false;
        };

        // Set our new ratio
        node_ref.set_ratio(new_ratio);

        // If the path is empty, this node is root => no sibling to adjust.
        if path.is_empty() {
            return true;
        }

        // The parent path is everything except the last index.
        let (parent_path, leaf_idx_slice) = path.split_at(path.len() - 1);
        let leaf_index = leaf_idx_slice[0];

        let parent_ref = Self::get_node_mut(&mut self.root, parent_path);

        // If parent is a Split/Tab with exactly two children, adjust sibling ratio:
        match parent_ref {
            DockNode::Split { children, .. } if children.len() == 2 => {
                let sibling_idx = if leaf_index == 0 { 1 } else { 0 };
                children[sibling_idx].set_ratio(1.0 - new_ratio);
            }
            DockNode::Tab { tabs, .. } if tabs.len() == 2 => {
                let sibling_idx = if leaf_index == 0 { 1 } else { 0 };
                tabs[sibling_idx].set_ratio(1.0 - new_ratio);
            }
            _ => {}
        }

        true
    }

    /// Find the bounding rectangle of a given node by ID (assuming a Leaf’s `window_identifier`).
    pub fn calculate_window_rect(
        &self,
        window_id: &str,
    ) -> Option<(f32, f32, f32, f32)> {
        self.root
            .find_window_rect(window_id, 0.0, 0.0, self.available_width, self.available_height)
    }

    /// Select (activate) the tab containing the specified leaf by setting the tab node’s
    /// `active_tab_id` to the given `leaf_id`. Returns `true` if successful, otherwise `false`.
    pub fn select_tab_by_leaf_id(
        &mut self,
        leaf_id: &str,
    ) -> bool {
        let Some((_leaf_node, path)) = self.find_node_by_id_mut(leaf_id) else {
            return false;
        };

        if path.is_empty() {
            return false;
        }

        let (parent_path, _) = path.split_at(path.len() - 1);
        let parent_node = Self::get_node_mut(&mut self.root, parent_path);

        if let DockNode::Tab { active_tab_id, .. } = parent_node {
            *active_tab_id = leaf_id.to_owned();
            true
        } else {
            false
        }
    }

    /// Traverse the path and return a mutable reference to the node at that path.
    /// If the path is empty, that means we're referring to the node itself.
    pub fn get_node_mut<'a>(
        mut node: &'a mut DockNode,
        path: &[usize],
    ) -> &'a mut DockNode {
        for &index in path {
            match node {
                DockNode::Split { children, .. } => {
                    node = &mut children[index];
                }
                DockNode::Tab { tabs, .. } => {
                    node = &mut tabs[index];
                }
                DockNode::Leaf { .. } => {
                    log::error!("Path goes deeper than a leaf node!");
                }
            }
        }
        node
    }

    /// Traverse the path and return an immutable reference to the node at that path.
    /// If the path is empty, that means we're referring to the node itself.
    pub fn get_node<'a>(
        node: &'a DockNode,
        path: &[usize],
    ) -> &'a DockNode {
        let mut current = node;
        for &index in path {
            match current {
                DockNode::Split { children, .. } => {
                    current = &children[index];
                }
                DockNode::Tab { tabs, .. } => {
                    current = &tabs[index];
                }
                DockNode::Leaf { .. } => {
                    log::error!("Path goes deeper than a leaf node!");
                    return current;
                }
            }
        }
        current
    }

    /// Clean up the layout so it's ready for presentation. Ensures that, for example,
    /// if an active tab is hidden, we pick another visible tab to become active.
    pub fn prepare_for_presentation(&mut self) {
        Self::prepare_for_presentation_node(&mut self.root);
    }

    /// Recursively fix up the docking layout's internal structure so that it's valid for display.
    fn prepare_for_presentation_node(node: &mut DockNode) {
        match node {
            // For splits, we simply recurse into each child.
            DockNode::Split { children, .. } => {
                for child in children {
                    Self::prepare_for_presentation_node(child);
                }
            }

            // For tab nodes, make sure the active tab is valid; if not, pick another.
            DockNode::Tab { active_tab_id, tabs, .. } => {
                // First, recurse down to ensure all tabs are also prepared:
                for tab_child in tabs.iter_mut() {
                    Self::prepare_for_presentation_node(tab_child);
                }

                // Find the currently active tab, if any.
                let mut active_tab_is_valid = false;
                if !active_tab_id.is_empty() {
                    // Check if there's a leaf with the same ID that is visible.
                    if let Some(_) = tabs
                        .iter()
                        .position(|child| child.is_leaf_with_id(active_tab_id) && child.is_visible())
                    {
                        // The current active tab is valid if found + visible.
                        active_tab_is_valid = true;
                    }
                }

                // If the active tab is invalid/hidden, pick the first visible tab as active (greedy).
                if !active_tab_is_valid {
                    if let Some(new_active_identifier) = tabs
                        .iter()
                        .filter(|child| child.is_visible())
                        .find_map(|child| match child {
                            DockNode::Leaf { window_identifier, .. } => Some(window_identifier.clone()),
                            _ => None,
                        })
                    {
                        *active_tab_id = new_active_identifier;
                    } else {
                        // Otherwise, clear it if no visible leaves remain:
                        active_tab_id.clear();
                    }
                }
            }

            // Leaf nodes have nothing to fix up here.
            DockNode::Leaf { .. } => {}
        }
    }
}
