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

    /// Attempt to resize a leaf node or tab node by adjusting its ratio.
    /// This only works if the node has exactly one sibling (i.e., two total siblings).
    pub fn resize_window(
        &mut self,
        window_id: &str,
        new_ratio: f32,
    ) -> bool {
        // Pass 1: find path (immutable).
        let path = match self.find_path_to_leaf(window_id) {
            Some(path) => path,
            None => return false,
        };

        // Pass 2: use path (mutable).
        let node_ref = Self::get_node_mut(&mut self.root, &path);
        node_ref.set_ratio(new_ratio);

        if path.is_empty() {
            // No parent to adjust (this node is the root).
            return true;
        }

        // The parent path is everything except the last index.
        let (parent_path, leaf_idx_slice) = path.split_at(path.len() - 1);
        let leaf_index = leaf_idx_slice[0];
        let parent_ref = Self::get_node_mut(&mut self.root, parent_path);

        match parent_ref {
            // If parent is a Split with exactly two children, adjust sibling ratio
            DockNode::Split { children, .. } if children.len() == 2 => {
                let sibling_idx = if leaf_index == 0 { 1 } else { 0 };
                children[sibling_idx].set_ratio(1.0 - new_ratio);
            }
            // If parent is a Tab with exactly two tabs, adjust sibling ratio
            DockNode::Tab { tabs, .. } if tabs.len() == 2 => {
                let sibling_idx = if leaf_index == 0 { 1 } else { 0 };
                tabs[sibling_idx].set_ratio(1.0 - new_ratio);
            }
            // Otherwise, no sibling adjustment
            _ => {}
        }

        true
    }

    pub fn get_node_by_id(
        &self,
        identifier: &str,
    ) -> Option<&DockNode> {
        let path = match self.find_path_to_leaf(identifier) {
            Some(path) => path,
            None => return None,
        };

        let node_ref = &Self::get_node(&self.root, &path);

        Some(node_ref)
    }

    pub fn get_node_by_id_mut(
        &mut self,
        identifier: &str,
    ) -> Option<&mut DockNode> {
        let path = match self.find_path_to_leaf(identifier) {
            Some(path) => path,
            None => return None,
        };

        let node_ref = Self::get_node_mut(&mut self.root, &path);

        Some(node_ref)
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
        // 1) Find path to leaf node.
        let path = match self.find_path_to_leaf(leaf_id) {
            Some(path) => path,
            None => return false,
        };

        // 2) If the path is empty, then the root *is* the leaf—no parent to set.
        if path.is_empty() {
            return false;
        }

        // 3) Split path into parent path + leaf index.
        let (parent_path, _) = path.split_at(path.len() - 1);

        // 4) Grab a mutable reference to the parent node.
        let parent_node = Self::get_node_mut(&mut self.root, parent_path);

        // 5) If the parent node is a Tab, set its active_tab_id and return true.
        if let DockNode::Tab { active_tab_id, .. } = parent_node {
            *active_tab_id = leaf_id.to_owned();
            true
        } else {
            false
        }
    }

    /// Return a path of indices that leads to the leaf node matching `window_id`.
    /// Example of a path: [2, 0] means: in root.children[2].children[0], or root.tabs[2].tabs[0].
    pub fn find_path_to_leaf(
        &self,
        window_id: &str,
    ) -> Option<Vec<usize>> {
        self.root.find_path_to_leaf(window_id)
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
                    // Check if there's a leaf with the same ID and it’s visible.
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
