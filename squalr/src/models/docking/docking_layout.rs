use crate::models::docking::dock_builder::DockBuilder;
use crate::models::docking::dock_split_direction::DockSplitDirection;
use crate::models::docking::dockable_window_settings::DockableWindowSettings;
use crate::models::docking::docked_window_node::DockedWindowNode;

pub struct DockingLayout {
    root: DockedWindowNode,
    available_width: f32,
    available_height: f32,
}

impl DockingLayout {
    pub fn new() -> Self {
        Self {
            root: DockedWindowNode::default(),
            available_width: 0.0,
            available_height: 0.0,
        }
    }

    pub fn default() -> Self {
        // Create the default three-panel layout
        Self::from_builder(
            DockBuilder::new("root")
                .direction(DockSplitDirection::Horizontal)
                .split(
                    0.7,
                    DockBuilder::new("vsplit_1")
                        .direction(DockSplitDirection::Vertical)
                        .split(
                            0.5,
                            DockBuilder::new("hsplit_1")
                                .direction(DockSplitDirection::Horizontal)
                                .split(0.2, DockBuilder::new("process-selector"))
                                .split(0.3, DockBuilder::new("project-explorer"))
                                .split(0.5, DockBuilder::new("settings")),
                        )
                        .split(0.5, DockBuilder::new("output")),
                )
                .split(
                    0.3,
                    DockBuilder::new("vsplit_2")
                        .direction(DockSplitDirection::Vertical)
                        .split(0.6, DockBuilder::new("scan-results"))
                        .split(0.4, DockBuilder::new("property-viewer")),
                ),
        )
    }

    pub fn from_settings() -> Self {
        let docking_layout = Self::default();

        docking_layout
    }

    pub fn from_builder(builder: DockBuilder) -> Self {
        Self {
            root: builder.build(),
            available_width: 0.0,
            available_height: 0.0,
        }
    }

    pub fn set_available_width(
        &mut self,
        available_width: f32,
    ) {
        self.available_width = available_width;
    }

    pub fn set_available_height(
        &mut self,
        available_height: f32,
    ) {
        self.available_height = available_height;
    }

    pub fn set_available_size(
        &mut self,
        available_width: f32,
        available_height: f32,
    ) {
        self.available_width = available_width;
        self.available_height = available_height;
    }

    pub fn resize_window(
        &mut self,
        window_id: &str,
        new_ratio: f32,
    ) -> bool {
        if let Some((parent, index)) = Self::find_parent_and_index(&mut self.root, window_id) {
            // Only adjust the sibling ratio (assumes two panes)
            if parent.children.len() == 2 {
                let sibling_index = 1 - index;
                parent.children[index].ratio = new_ratio;
                parent.children[sibling_index].ratio = 1.0 - new_ratio;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_all_nodes(&self) -> Vec<DockedWindowNode> {
        let mut nodes = Vec::new();
        Self::collect_nodes(&self.root, &mut nodes);
        nodes
    }

    fn collect_nodes<'a>(
        node: &'a DockedWindowNode,
        nodes: &mut Vec<DockedWindowNode>,
    ) {
        // Add current node
        nodes.push(node.clone());

        // Recursively collect child nodes
        for child in &node.children {
            Self::collect_nodes(child, nodes);
        }
    }

    fn find_parent_and_index<'a>(
        root: &'a mut DockedWindowNode,
        window_id: &str,
    ) -> Option<(&'a mut DockedWindowNode, usize)> {
        for (index, child) in root.children.iter().enumerate() {
            if child.window_identifier == window_id {
                return Some((root, index));
            }
        }

        for child in &mut root.children {
            if let Some(found) = Self::find_parent_and_index(child, window_id) {
                return Some(found);
            }
        }
        None
    }

    pub fn calculate_window_rect(
        &self,
        window_id: &str,
    ) -> Option<(f32, f32, f32, f32)> {
        Self::find_window_rect(&self.root, window_id, 0.0, 0.0, self.available_width, self.available_height)
    }

    pub fn save(&self) {
        let settings = DockableWindowSettings::get_instance();
        settings.set_dock_layout_settings(self.root.clone());
    }

    fn find_window_rect(
        node: &DockedWindowNode,
        target_id: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        // If this node is invisible, skip it entirely (including children).
        if !node.is_visible {
            return None;
        }

        // If this is the target window, return the current rect.
        if node.window_identifier == target_id {
            return Some((x, y, width, height));
        }

        // Collect *only visible* children.
        let visible_children: Vec<_> = node.children.iter().filter(|child| child.is_visible).collect();

        // If no visible children remain, there is no sub-layout to traverse.
        if visible_children.is_empty() {
            return None;
        }

        // Compute total ratio of visible children so we can re-normalize.
        let total_ratio: f32 = visible_children.iter().map(|child| child.ratio).sum();

        let mut current_offset = 0.0;
        let visible_children_len = visible_children.len();

        for child in visible_children {
            // Re-normalize ratio to ensure only visible children share space.
            let child_ratio = if total_ratio > 0.0 {
                child.ratio / total_ratio
            } else {
                // Fallback to equal distribution if total_ratio == 0.0
                1.0 / visible_children_len as f32
            };

            // Calculate child size based on *effective* ratio.
            let (child_width, child_height) = match node.direction {
                DockSplitDirection::Horizontal => (width * child_ratio, height),
                DockSplitDirection::Vertical => (width, height * child_ratio),
            };

            // Calculate child position.
            let (child_x, child_y) = match node.direction {
                DockSplitDirection::Horizontal => (x + current_offset, y),
                DockSplitDirection::Vertical => (x, y + current_offset),
            };

            // Recursively search this child.
            if let Some(rect) = Self::find_window_rect(child, target_id, child_x, child_y, child_width, child_height) {
                return Some(rect);
            }

            // Update offset for the next visible child.
            match node.direction {
                DockSplitDirection::Horizontal => current_offset += child_width,
                DockSplitDirection::Vertical => current_offset += child_height,
            }
        }

        None
    }
}
