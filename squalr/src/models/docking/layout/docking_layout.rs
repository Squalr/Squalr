use crate::models::docking::dock_node::DockNode;
use crate::models::docking::dock_split_direction::DockSplitDirection;
use crate::models::docking::dock_tree::DockTree;

#[derive(Debug)]
pub struct DockingLayout {
    pub available_width: f32,
    pub available_height: f32,
}

impl DockingLayout {
    pub fn new() -> Self {
        Self {
            available_width: 0.0,
            available_height: 0.0,
        }
    }

    pub fn set_available_size(
        &mut self,
        width: f32,
        height: f32,
    ) {
        self.available_width = width;
        self.available_height = height;
    }

    pub fn set_available_width(
        &mut self,
        width: f32,
    ) {
        self.available_width = width;
    }

    pub fn set_available_height(
        &mut self,
        height: f32,
    ) {
        self.available_height = height;
    }

    /// Compute bounding rectangles for every visible node. The `visitor` receives `(node, (x, y, w, h))`.
    fn walk_with_layout_and_path<F>(
        &self,
        node: &DockNode,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        path: &mut Vec<usize>,
        visitor: &mut F,
    ) where
        F: FnMut(&DockNode, &[usize], (f32, f32, f32, f32)),
    {
        // Call the visitor on the current node
        visitor(node, path, (x, y, w, h));

        match node {
            DockNode::Split { direction, children, .. } => {
                let visible_children: Vec<&DockNode> = children.iter().filter(|c| c.is_visible()).collect();
                if visible_children.is_empty() {
                    return;
                }

                let total_ratio: f32 = visible_children.iter().map(|c| c.get_ratio()).sum();
                let mut offset = 0.0;
                let child_count = visible_children.len();

                for (original_idx, child) in children.iter().enumerate() {
                    if !child.is_visible() {
                        continue;
                    }

                    let child_ratio = if total_ratio > 0.0 {
                        child.get_ratio() / total_ratio
                    } else {
                        1.0 / child_count as f32
                    };

                    let (cw, ch) = match direction {
                        DockSplitDirection::Horizontal => (w * child_ratio, h),
                        DockSplitDirection::Vertical => (w, h * child_ratio),
                    };
                    let (cx, cy) = match direction {
                        DockSplitDirection::Horizontal => (x + offset, y),
                        DockSplitDirection::Vertical => (x, y + offset),
                    };

                    // Push this child's index
                    path.push(original_idx);
                    self.walk_with_layout_and_path(child, cx, cy, cw, ch, path, visitor);
                    path.pop();

                    // Advance offset
                    match direction {
                        DockSplitDirection::Horizontal => offset += cw,
                        DockSplitDirection::Vertical => offset += ch,
                    }
                }
            }
            DockNode::Tab { tabs, .. } => {
                let visible_children: Vec<&DockNode> = tabs.iter().filter(|c| c.is_visible()).collect();
                if visible_children.is_empty() {
                    return;
                }

                // All visible tabs receive the same (x, y, w, h).
                for (original_idx, tab_child) in tabs.iter().enumerate() {
                    if !tab_child.is_visible() {
                        continue;
                    }

                    path.push(original_idx);
                    self.walk_with_layout_and_path(tab_child, x, y, w, h, path, visitor);
                    path.pop();
                }
            }
            // No children, just a leaf
            DockNode::Leaf { .. } => {}
        }
    }

    /// Finds the bounding rectangle of the specified leaf ID. Returns None if not found or not visible.
    pub fn find_window_rect(
        &self,
        tree: &DockTree,
        leaf_id: &str,
    ) -> Option<(f32, f32, f32, f32)> {
        let path = tree.find_leaf_path(leaf_id)?;
        self.find_node_rect(tree, &path)
    }

    /// Find the bounding rectangle of a node at the given path.
    pub fn find_node_rect(
        &self,
        tree: &DockTree,
        path: &[usize],
    ) -> Option<(f32, f32, f32, f32)> {
        let mut found = None;
        let mut path_stack = Vec::new();
        self.walk_with_layout_and_path(
            &tree.root,
            0.0,
            0.0,
            self.available_width,
            self.available_height,
            &mut path_stack,
            &mut |_node, current_path, rect| {
                if current_path == path {
                    found = Some(rect);
                }
            },
        );

        found
    }
}
