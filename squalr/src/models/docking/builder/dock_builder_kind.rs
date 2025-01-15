use crate::models::docking::builder::dock_builder::DockBuilder;
use crate::models::docking::dock_split_direction::DockSplitDirection;

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
    Leaf {
        window_identifier: String,
    },
}
