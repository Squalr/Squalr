use crate::models::docking::builder::dock_builder::DockBuilder;
use crate::models::docking::hierarchy::types::dock_split_direction::DockSplitDirection;

/// Represents the types of dockable nodes that can be created by the dock builder.
#[derive(Debug)]
pub enum DockBuilderKind {
    /// Builds a split node type, which is a container with multiple children split horizontally or vertically.
    Split {
        direction: DockSplitDirection,
        children: Vec<(DockBuilder, f32)>,
    },
    /// Builds a tab node type, which is a container with multiple children as tabs.
    Tab { tabs: Vec<DockBuilder>, active_tab_id: String },
    /// Builds a window node type, which is the final leaf node that displays content.
    Window { window_identifier: String },
}
