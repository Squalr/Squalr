use crate::models::docking::drag_drop::dock_tab_drop_target::DockTabDropTarget;
use epaint::Pos2;

#[derive(Clone, Debug, PartialEq)]
pub struct DockDragState {
    pub source_window_identifier: String,
    pointer_press_origin: Pos2,
    current_pointer_position: Option<Pos2>,
    has_crossed_activation_distance: bool,
    hovered_tab_drop_target: Option<DockTabDropTarget>,
}

impl DockDragState {
    pub const ACTIVATION_DISTANCE_PX: f32 = 24.0;

    pub fn new(
        source_window_identifier: String,
        pointer_press_origin: Pos2,
    ) -> Self {
        Self {
            source_window_identifier,
            pointer_press_origin,
            current_pointer_position: Some(pointer_press_origin),
            has_crossed_activation_distance: false,
            hovered_tab_drop_target: None,
        }
    }

    pub fn update_pointer_position(
        &mut self,
        current_pointer_position: Option<Pos2>,
    ) {
        self.current_pointer_position = current_pointer_position;

        if self.has_crossed_activation_distance {
            return;
        }

        if let Some(current_pointer_position) = current_pointer_position {
            self.has_crossed_activation_distance = self.pointer_press_origin.distance(current_pointer_position) >= Self::ACTIVATION_DISTANCE_PX;
        }
    }

    pub fn current_pointer_position(&self) -> Option<Pos2> {
        self.current_pointer_position
    }

    pub fn clear_hovered_tab_drop_target(&mut self) {
        self.hovered_tab_drop_target = None;
    }

    pub fn set_hovered_tab_drop_target(
        &mut self,
        hovered_tab_drop_target: DockTabDropTarget,
    ) {
        self.hovered_tab_drop_target = Some(hovered_tab_drop_target);
    }

    pub fn hovered_tab_drop_target(&self) -> Option<&DockTabDropTarget> {
        self.hovered_tab_drop_target.as_ref()
    }

    pub fn is_drop_overlay_visible(&self) -> bool {
        self.has_crossed_activation_distance
    }
}

#[cfg(test)]
mod tests {
    use super::DockDragState;
    use epaint::pos2;

    #[test]
    fn drag_overlay_only_activates_after_threshold() {
        let mut dock_drag_state = DockDragState::new("source".to_string(), pos2(10.0, 10.0));

        dock_drag_state.update_pointer_position(Some(pos2(20.0, 20.0)));

        assert!(!dock_drag_state.is_drop_overlay_visible());

        dock_drag_state.update_pointer_position(Some(pos2(40.0, 10.0)));

        assert!(dock_drag_state.is_drop_overlay_visible());
    }

    #[test]
    fn drag_overlay_stays_active_after_threshold_is_crossed() {
        let mut dock_drag_state = DockDragState::new("source".to_string(), pos2(10.0, 10.0));

        dock_drag_state.update_pointer_position(Some(pos2(40.0, 10.0)));
        dock_drag_state.update_pointer_position(Some(pos2(18.0, 10.0)));

        assert!(dock_drag_state.is_drop_overlay_visible());
    }
}
