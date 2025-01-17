use crate::models::docking::builder::dock_builder::DockBuilder;
use crate::models::docking::hierarchy::types::dock_split_direction::DockSplitDirection;

#[derive(Debug)]
pub enum DockBuilderKind {
    Split {
        direction: DockSplitDirection,
        children: Vec<(DockBuilder, f32)>,
    },
    Tab {
        tabs: Vec<DockBuilder>,
        active_tab_id: String,
    },
    Window {
        window_identifier: String,
    },
}
