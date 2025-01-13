use crate::models::docking::dock_builder::DockBuilder;
use crate::models::docking::dock_node::DockNode;
use crate::models::docking::dock_split_direction::DockSplitDirection;

#[derive(Debug)]
pub struct DockingLayout {
    pub root: DockNode,
    pub available_width: f32,
    pub available_height: f32,
}

impl DockingLayout {
    /// Create a new empty layout.
    pub fn new() -> Self {
        Self {
            root: DockNode::default(),
            available_width: 0.0,
            available_height: 0.0,
        }
    }

    /// The default layout for Squalr.
    pub fn default() -> Self {
        let builder = DockBuilder::split_node(DockSplitDirection::Horizontal)
            .push_child(
                0.7,
                DockBuilder::split_node(DockSplitDirection::Vertical)
                    .push_child(
                        0.5,
                        DockBuilder::split_node(DockSplitDirection::Horizontal)
                            // Build a tab with two leaves: process-selector & project-explorer
                            .push_child(
                                0.5,
                                DockBuilder::tab_node("project-explorer")
                                    .push_tab(DockBuilder::leaf("process-selector"))
                                    .push_tab(DockBuilder::leaf("project-explorer")),
                            )
                            // And a leaf node for "settings" occupying the other 0.5
                            .push_child(0.5, DockBuilder::leaf("settings")),
                    )
                    .push_child(0.5, DockBuilder::leaf("output")),
            )
            .push_child(
                0.3,
                DockBuilder::split_node(DockSplitDirection::Vertical)
                    .push_child(0.6, DockBuilder::leaf("scan-results"))
                    .push_child(0.4, DockBuilder::leaf("property-viewer")),
            );

        Self {
            root: builder.build(),
            available_width: 0.0,
            available_height: 0.0,
        }
    }

    /// Create a new layout from saved settings.
    pub fn from_settings() -> Self {
        let dock_layout = Self::default();

        dock_layout
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
        let path = match Self::find_path_to_leaf(&self.root, window_id) {
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
        let path = match Self::find_path_to_leaf(&self.root, identifier) {
            Some(path) => path,
            None => return None,
        };

        let node_ref = &Self::get_node(&self.root, &path);

        Some(node_ref)
    }

    pub fn get_all_leaves(&self) -> Vec<String> {
        let mut leaves = Vec::new();
        Self::collect_leaves(&self.root, &mut leaves);
        leaves
    }

    fn collect_leaves(
        node: &DockNode,
        leaves: &mut Vec<String>,
    ) {
        match node {
            DockNode::Leaf { window_identifier, .. } => {
                leaves.push(window_identifier.clone());
            }
            DockNode::Split { children, .. } => {
                for child in children {
                    Self::collect_leaves(child, leaves);
                }
            }
            DockNode::Tab { tabs, .. } => {
                for child in tabs {
                    Self::collect_leaves(child, leaves);
                }
            }
        }
    }

    /// Find the bounding rectangle of a given node by ID (assuming a Leaf’s `window_identifier`).
    pub fn calculate_window_rect(
        &self,
        window_id: &str,
    ) -> Option<(f32, f32, f32, f32)> {
        Self::find_window_rect(&self.root, window_id, 0.0, 0.0, self.available_width, self.available_height)
    }

    /// Recursively search for `window_id` in a node, returning `(x, y, w, h)`.
    fn find_window_rect(
        node: &DockNode,
        target_id: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        match node {
            DockNode::Leaf {
                window_identifier, is_visible, ..
            } => {
                if !is_visible {
                    return None;
                }
                if window_identifier == target_id {
                    return Some((x, y, width, height));
                }
                None
            }
            DockNode::Split {
                direction,
                is_visible,
                children,
                ..
            } => {
                if !is_visible {
                    return None;
                }
                // Collect only visible children for layout distribution.
                let visible_children: Vec<&DockNode> = children.iter().filter(|c| c.is_visible()).collect();

                if visible_children.is_empty() {
                    return None;
                }

                // Sum ratios for normalization
                let total_ratio: f32 = visible_children.iter().map(|c| c.get_ratio()).sum();
                let mut offset = 0.0;
                let visible_children_len = visible_children.len();

                for child in visible_children {
                    // Re-normalize ratio among visible children
                    let child_ratio = if total_ratio > 0.0 {
                        child.get_ratio() / total_ratio
                    } else {
                        1.0 / visible_children_len as f32
                    };

                    let (cw, ch) = match direction {
                        DockSplitDirection::Horizontal => (width * child_ratio, height),
                        DockSplitDirection::Vertical => (width, height * child_ratio),
                    };

                    let (cx, cy) = match direction {
                        DockSplitDirection::Horizontal => (x + offset, y),
                        DockSplitDirection::Vertical => (x, y + offset),
                    };

                    // Recurse
                    if let Some(rect) = Self::find_window_rect(child, target_id, cx, cy, cw, ch) {
                        return Some(rect);
                    }

                    match direction {
                        DockSplitDirection::Horizontal => offset += cw,
                        DockSplitDirection::Vertical => offset += ch,
                    }
                }
                None
            }
            DockNode::Tab { is_visible, tabs, .. } => {
                if !is_visible {
                    return None;
                }
                // If a Tab node is visible, only one “page” is typically visible at a time.
                // But for simplicity, let's just check them all logically:
                for child in tabs {
                    if let Some(rect) = Self::find_window_rect(child, target_id, x, y, width, height) {
                        return Some(rect);
                    }
                }
                None
            }
        }
    }

    /// Select (activate) the tab containing the specified leaf by setting the tab node’s
    /// `active_tab_id` to the given `leaf_id`. Returns `true` if successful, otherwise `false`.
    pub fn select_tab_by_leaf_id(
        &mut self,
        leaf_id: &str,
    ) -> bool {
        // 1) Find path to leaf node.
        let path = match Self::find_path_to_leaf(&self.root, leaf_id) {
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
        node: &DockNode,
        window_id: &str,
    ) -> Option<Vec<usize>> {
        match node {
            DockNode::Leaf { window_identifier, .. } => {
                if window_identifier == window_id {
                    // Found it! Return an empty path meaning "we are the node."
                    Some(vec![])
                } else {
                    None
                }
            }
            DockNode::Split { children, .. } => {
                // Try each child in turn
                for (i, child) in children.iter().enumerate() {
                    if let Some(mut path) = Self::find_path_to_leaf(child, window_id) {
                        // Found it in child i. Prepend `i` to the path.
                        path.insert(0, i);
                        return Some(path);
                    }
                }
                None
            }
            DockNode::Tab { tabs, .. } => {
                for (i, tab) in tabs.iter().enumerate() {
                    if let Some(mut path) = Self::find_path_to_leaf(tab, window_id) {
                        path.insert(0, i);
                        return Some(path);
                    }
                }
                None
            }
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
}

/// Helper methods on `DockNode`.
impl DockNode {
    /// Check if a node is visible.
    pub fn is_visible(&self) -> bool {
        match self {
            DockNode::Split { is_visible, .. } => *is_visible,
            DockNode::Tab { is_visible, .. } => *is_visible,
            DockNode::Leaf { is_visible, .. } => *is_visible,
        }
    }

    /// Get ratio of a node.
    pub fn get_ratio(&self) -> f32 {
        match self {
            DockNode::Split { ratio, .. } => *ratio,
            DockNode::Tab { ratio, .. } => *ratio,
            DockNode::Leaf { ratio, .. } => *ratio,
        }
    }

    /// Set ratio of a node.
    pub fn set_ratio(
        &mut self,
        new_ratio: f32,
    ) {
        match self {
            DockNode::Split { ratio, .. } => *ratio = new_ratio,
            DockNode::Tab { ratio, .. } => *ratio = new_ratio,
            DockNode::Leaf { ratio, .. } => *ratio = new_ratio,
        }
    }

    /// Check if this node is a leaf with a specific ID.
    pub fn is_leaf_with_id(
        &self,
        target_id: &str,
    ) -> bool {
        match self {
            DockNode::Leaf { window_identifier, .. } => window_identifier == target_id,
            _ => false,
        }
    }
}
