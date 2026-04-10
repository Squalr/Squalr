use crate::{
    app_context::AppContext,
    models::docking::{drag_drop::dock_tab_drop_target::DockTabDropTarget, hierarchy::types::dock_tab_insertion_direction::DockTabInsertionDirection},
    ui::widgets::{controls::button::Button, docking::dock_root_view_data::DockRootViewData},
};
use eframe::egui::{Align, Align2, CursorIcon, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, Pos2, Rect, pos2, vec2};
use std::{rc::Rc, sync::Arc};

#[derive(Clone)]
pub struct DockedWindowFooterView {
    app_context: Arc<AppContext>,
    dock_view_data: Arc<DockRootViewData>,
    identifier: Rc<String>,
    height: f32,
}

impl DockedWindowFooterView {
    pub fn new(
        app_context: Arc<AppContext>,
        dock_view_data: Arc<DockRootViewData>,
        identifier: Rc<String>,
    ) -> Self {
        Self {
            app_context,
            dock_view_data,
            identifier,
            height: 24.0,
        }
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }
}

impl Widget for DockedWindowFooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, self.height), Sense::empty());
        let theme = &self.app_context.theme;
        let pointer_position = user_interface.ctx().pointer_interact_pos();
        let (sibling_ids, active_tab_id, active_dragged_window_identifier, is_drag_drop_active) = match self.app_context.docking_manager.read() {
            Ok(docking_manager_guard) => (
                docking_manager_guard.get_sibling_tab_ids(&self.identifier, true),
                docking_manager_guard.get_active_tab(&self.identifier),
                docking_manager_guard
                    .active_dragged_window_id()
                    .map(str::to_string),
                docking_manager_guard.is_drag_drop_active(),
            ),
            Err(error) => {
                log::error!("Failed to acquire docking manager lock: {}", error);
                return response;
            }
        };
        let windows = match self.dock_view_data.windows.read() {
            Ok(windows) => windows,
            Err(error) => {
                log::error!("Failed to acquire windows lock: {}", error);
                return response;
            }
        };

        // Background.
        user_interface
            .painter()
            .rect_filled(available_size_rect, CornerRadius::ZERO, theme.background_primary);

        let builder = UiBuilder::new()
            .max_rect(available_size_rect)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);
        let mut selected_tab_id = None;
        let mut toggled_maximized_tab_id = None;
        let mut drag_start_request = None;
        let mut hovered_tab_drop_target = None;

        for sibling_id in sibling_ids {
            let mut button = Button::new_from_theme(theme)
                .background_color(theme.background_control_secondary)
                .border_color(theme.submenu_border)
                .border_width(1.0);

            if sibling_id == active_tab_id {
                button.backgorund_color = theme.background_control_primary;
                button.border_color = theme.background_control_primary_light;
            }

            if active_dragged_window_identifier.as_deref() == Some(sibling_id.as_str()) {
                button.backgorund_color = theme.selected_background;
                button.border_color = theme.selected_border;
            }

            let response = child_user_interface
                .add_sized(
                    vec2(128.0, available_size_rect.height()),
                    button
                        .corner_radius(CornerRadius::ZERO)
                        .sense(Sense::click_and_drag()),
                )
                .on_hover_cursor(CursorIcon::Grab);

            if response.rect.is_positive() {
                for window in windows.iter() {
                    if window.get_identifier() == sibling_id {
                        child_user_interface.painter().text(
                            response.rect.center(),
                            Align2::CENTER_CENTER,
                            window.get_title(),
                            theme.font_library.font_noto_sans.font_header.clone(),
                            theme.foreground,
                        );

                        break;
                    }
                }
            }

            if response.clicked() {
                selected_tab_id = Some(sibling_id.clone());
            }

            if response.double_clicked() {
                selected_tab_id = Some(sibling_id.clone());
                toggled_maximized_tab_id = Some(sibling_id.clone());
            }

            if response.drag_started() {
                let pointer_press_origin = child_user_interface
                    .input(|input_state| input_state.pointer.press_origin())
                    .or_else(|| response.interact_pointer_pos());

                if let Some(pointer_press_origin) = pointer_press_origin {
                    drag_start_request = Some((sibling_id.clone(), pointer_press_origin));
                }

                child_user_interface.ctx().request_repaint();
            }

            if response.dragged() {
                child_user_interface.ctx().set_cursor_icon(CursorIcon::Grabbing);
                child_user_interface.ctx().request_repaint();
            }

            if is_drag_drop_active {
                let resolved_tab_drop_target =
                    resolve_tab_drop_target(active_dragged_window_identifier.as_deref(), &sibling_id, response.rect, pointer_position);

                if let Some(resolved_tab_drop_target) = resolved_tab_drop_target {
                    paint_tab_drop_preview(&child_user_interface, theme, response.rect, resolved_tab_drop_target.tab_insertion_direction);
                    hovered_tab_drop_target = Some(resolved_tab_drop_target);
                }
            }
        }

        if let Some((dragged_tab_identifier, pointer_press_origin)) = drag_start_request {
            if let Ok(mut docking_manager) = self.app_context.docking_manager.write() {
                docking_manager.begin_drag(&dragged_tab_identifier, pointer_press_origin);
            }
        }

        if let Some(hovered_tab_drop_target) = hovered_tab_drop_target {
            if let Ok(mut docking_manager) = self.app_context.docking_manager.write() {
                docking_manager.set_hovered_tab_drop_target(hovered_tab_drop_target);
            }
        }

        if let Some(selected_tab_id) = selected_tab_id {
            if let Ok(mut docking_manager) = self.app_context.docking_manager.write() {
                docking_manager.select_tab_by_window_id(&selected_tab_id);
            }
        }

        if let Some(toggled_maximized_tab_id) = toggled_maximized_tab_id {
            self.dock_view_data
                .toggle_maximized_window_identifier(&toggled_maximized_tab_id);
        }

        response
    }
}

fn resolve_tab_drop_target(
    dragged_window_identifier: Option<&str>,
    target_window_identifier: &str,
    target_tab_rect: Rect,
    pointer_position: Option<Pos2>,
) -> Option<DockTabDropTarget> {
    let dragged_window_identifier = dragged_window_identifier?;
    let pointer_position = pointer_position?;

    if dragged_window_identifier == target_window_identifier || !target_tab_rect.contains(pointer_position) {
        return None;
    }

    let tab_insertion_direction = if pointer_position.x < target_tab_rect.center().x {
        DockTabInsertionDirection::BeforeTarget
    } else {
        DockTabInsertionDirection::AfterTarget
    };

    Some(DockTabDropTarget {
        target_window_identifier: target_window_identifier.to_string(),
        tab_insertion_direction,
    })
}

fn paint_tab_drop_preview(
    user_interface: &Ui,
    theme: &crate::ui::theme::Theme,
    target_tab_rect: Rect,
    tab_insertion_direction: DockTabInsertionDirection,
) {
    let preview_rect = build_tab_drop_preview_rect(target_tab_rect, tab_insertion_direction);
    let preview_fill = Color32::from_rgba_unmultiplied(theme.selected_border.r(), theme.selected_border.g(), theme.selected_border.b(), 0xD8);

    user_interface
        .painter()
        .rect_filled(preview_rect, CornerRadius::same(3), preview_fill);
}

fn build_tab_drop_preview_rect(
    target_tab_rect: Rect,
    tab_insertion_direction: DockTabInsertionDirection,
) -> Rect {
    let preview_width = (target_tab_rect.width() * 0.07).clamp(6.0, 10.0);

    match tab_insertion_direction {
        DockTabInsertionDirection::BeforeTarget => Rect::from_min_max(target_tab_rect.min, pos2(target_tab_rect.min.x + preview_width, target_tab_rect.max.y)),
        DockTabInsertionDirection::AfterTarget => Rect::from_min_max(pos2(target_tab_rect.max.x - preview_width, target_tab_rect.min.y), target_tab_rect.max),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_tab_drop_preview_rect, resolve_tab_drop_target};
    use crate::models::docking::hierarchy::types::dock_tab_insertion_direction::DockTabInsertionDirection;
    use epaint::{Rect, pos2};

    #[test]
    fn left_half_of_tab_resolves_to_before_target() {
        let target_tab_rect = Rect::from_min_max(pos2(100.0, 40.0), pos2(228.0, 64.0));
        let resolved_tab_drop_target =
            resolve_tab_drop_target(Some("dragged"), "target", target_tab_rect, Some(pos2(120.0, 52.0))).expect("expected tab drop target");

        assert_eq!(resolved_tab_drop_target.target_window_identifier, "target".to_string());
        assert_eq!(resolved_tab_drop_target.tab_insertion_direction, DockTabInsertionDirection::BeforeTarget,);
    }

    #[test]
    fn right_half_of_tab_resolves_to_after_target() {
        let target_tab_rect = Rect::from_min_max(pos2(100.0, 40.0), pos2(228.0, 64.0));
        let resolved_tab_drop_target =
            resolve_tab_drop_target(Some("dragged"), "target", target_tab_rect, Some(pos2(220.0, 52.0))).expect("expected tab drop target");

        assert_eq!(resolved_tab_drop_target.tab_insertion_direction, DockTabInsertionDirection::AfterTarget,);
    }

    #[test]
    fn preview_rect_hugs_requested_tab_edge() {
        let target_tab_rect = Rect::from_min_max(pos2(100.0, 40.0), pos2(228.0, 64.0));
        let left_preview_rect = build_tab_drop_preview_rect(target_tab_rect, DockTabInsertionDirection::BeforeTarget);
        let right_preview_rect = build_tab_drop_preview_rect(target_tab_rect, DockTabInsertionDirection::AfterTarget);

        assert_eq!(left_preview_rect.min.x, target_tab_rect.min.x);
        assert_eq!(right_preview_rect.max.x, target_tab_rect.max.x);
        assert!(left_preview_rect.max.x <= target_tab_rect.center().x);
        assert!(right_preview_rect.min.x >= target_tab_rect.center().x);
    }
}
