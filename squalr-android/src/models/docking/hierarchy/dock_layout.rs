use crate::models::docking::hierarchy::dock_node::DockNode;

/// Represents a layout area to be filled by an owned root `DockNode`.
#[derive(Debug)]
pub struct DockLayout {
    root_node: DockNode,
    available_width: f32,
    available_height: f32,
}

/// Implements methods to determine how much space each docked window takes up within the available space.
impl DockLayout {
    pub fn new(root_node: DockNode) -> Self {
        Self {
            root_node: root_node,
            available_width: 0.0,
            available_height: 0.0,
        }
    }

    /// Replace the entire root node in the root_node.
    pub fn set_root(
        &mut self,
        new_root: DockNode,
    ) {
        self.root_node = new_root;
    }

    /// Get the root node of the docking layout.
    pub fn get_root(&self) -> &DockNode {
        &self.root_node
    }

    /// Get the root node of the docking layout.
    pub fn get_root_mut(&mut self) -> &mut DockNode {
        &mut self.root_node
    }

    /// Sets the available width and height that this dock layout fills.
    pub fn set_available_size(
        &mut self,
        width: f32,
        height: f32,
    ) {
        self.available_width = width;
        self.available_height = height;
    }

    /// Gets the available width and height that this dock layout fills.
    pub fn get_available_size(&self) -> (f32, f32) {
        (self.available_width, self.available_height)
    }

    /// Sets the available width that this dock layout fills.
    pub fn set_available_width(
        &mut self,
        width: f32,
    ) {
        self.available_width = width;
    }

    /// Gets the available width that this dock layout fills.
    pub fn get_available_width(&self) -> f32 {
        self.available_width
    }

    /// Sets the available height that this dock layout fills.
    pub fn set_available_height(
        &mut self,
        height: f32,
    ) {
        self.available_height = height;
    }

    /// Gets the available height that this dock layout fills.
    pub fn get_available_height(&self) -> f32 {
        self.available_height
    }
}
