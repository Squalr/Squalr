use crate::models::docking::hierarchy::types::dock_tab_insertion_direction::DockTabInsertionDirection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockTabDropTarget {
    pub target_window_identifier: String,
    pub tab_insertion_direction: DockTabInsertionDirection,
}
