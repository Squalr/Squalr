use crate::RedockTarget;
use crate::models::docking::hierarchy::types::dock_reparent_direction::DockReparentDirection;
use slint_mvvm::view_data_converter::ViewDataConverter;

pub struct DocktargetConverter {}

impl DocktargetConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ViewDataConverter<DockReparentDirection, RedockTarget> for DocktargetConverter {
    fn convert_collection(
        &self,
        docked_window_nodes: &Vec<DockReparentDirection>,
    ) -> Vec<RedockTarget> {
        return docked_window_nodes
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect();
    }

    fn convert_to_view_data(
        &self,
        reparent_direction: &DockReparentDirection,
    ) -> RedockTarget {
        match reparent_direction {
            DockReparentDirection::Bottom => RedockTarget::Down,
            DockReparentDirection::Top => RedockTarget::Up,
            DockReparentDirection::Left => RedockTarget::Left,
            DockReparentDirection::Right => RedockTarget::Right,
            DockReparentDirection::Tab => RedockTarget::Center,
        }
    }

    fn convert_from_view_data(
        &self,
        redock_target: &RedockTarget,
    ) -> DockReparentDirection {
        match redock_target {
            RedockTarget::Down => DockReparentDirection::Bottom,
            RedockTarget::Up => DockReparentDirection::Top,
            RedockTarget::Left => DockReparentDirection::Left,
            RedockTarget::Right => DockReparentDirection::Right,
            RedockTarget::Center => DockReparentDirection::Tab,
        }
    }
}
