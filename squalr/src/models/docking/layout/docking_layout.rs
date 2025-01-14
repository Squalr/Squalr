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
    pub fn walk_with_layout<F>(
        &self,
        node: &DockNode,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        visitor: &mut F,
    ) where
        F: FnMut(&DockNode, (f32, f32, f32, f32)),
    {
        visitor(node, (x, y, w, h));

        match node {
            DockNode::Split { direction, children, .. } => {
                let visible_children: Vec<&DockNode> = children.iter().filter(|c| c.is_visible()).collect();
                if visible_children.is_empty() {
                    return;
                }

                let total_ratio: f32 = visible_children.iter().map(|c| c.get_ratio()).sum();
                let mut offset = 0.0;
                let child_count = visible_children.len();

                for child in visible_children {
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

                    self.walk_with_layout(child, cx, cy, cw, ch, visitor);

                    match direction {
                        DockSplitDirection::Horizontal => offset += cw,
                        DockSplitDirection::Vertical => offset += ch,
                    }
                }
            }
            // Each visible tab gets the entire rectangle.
            DockNode::Tab { tabs, .. } => {
                for tab_child in tabs {
                    if tab_child.is_visible() {
                        self.walk_with_layout(tab_child, x, y, w, h, visitor);
                    }
                }
            }
            DockNode::Leaf { .. } => {}
        }
    }

    /// Finds the bounding rectangle of the specified leaf ID. Returns `None` if not found or not visible.
    pub fn find_window_rect(
        &self,
        tree: &DockTree,
        leaf_id: &str,
    ) -> Option<(f32, f32, f32, f32)> {
        let mut found = None;
        self.walk_with_layout(&tree.root, 0.0, 0.0, self.available_width, self.available_height, &mut |node, (x, y, w, h)| {
            if let DockNode::Leaf {
                window_identifier, is_visible, ..
            } = node
            {
                if *is_visible && window_identifier == leaf_id {
                    found = Some((x, y, w, h));
                }
            }
        });
        found
    }
}
