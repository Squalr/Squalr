use crate::models::docking::drag_drop::dock_drop_zone::{DockDragOverlay, DockDropZone};
use crate::{app_context::AppContext, ui::widgets::docking::dock_root_view_data::DockRootViewData};
use eframe::egui::{CursorIcon, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, Rect, pos2, vec2};
use std::sync::Arc;

#[derive(Clone)]
pub struct DockRootView {
    app_context: Arc<AppContext>,
    dock_view_data: Arc<DockRootViewData>,
}

impl DockRootView {
    pub fn new(
        app_context: Arc<AppContext>,
        dock_view_data: Arc<DockRootViewData>,
    ) -> Self {
        Self { app_context, dock_view_data }
    }
}

impl Widget for DockRootView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rect, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());
        let theme = &self.app_context.theme;
        let docking_manager = &self.app_context.docking_manager;
        let pointer_position = user_interface.ctx().pointer_interact_pos();
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
            .rect_filled(available_size_rect, CornerRadius::ZERO, theme.background_panel);

        let mut dock_drag_overlay = None;
        let is_drag_release_frame = user_interface
            .ctx()
            .input(|input_state| input_state.pointer.primary_released());

        if let Ok(mut docking_manager) = docking_manager.try_write() {
            docking_manager.prepare_for_presentation();
            docking_manager
                .get_main_window_layout_mut()
                .set_available_size(available_size_rect.width(), available_size_rect.height());
            docking_manager.update_drag_pointer_position(pointer_position);

            if docking_manager.active_dragged_window_id().is_some() {
                user_interface.ctx().request_repaint();
            }

            if is_drag_release_frame {
                docking_manager.finish_drag(available_size_rect);
            } else {
                dock_drag_overlay = docking_manager.get_drag_overlay(available_size_rect);
            }
        }

        for window in windows.iter() {
            let window_identifier = window.get_identifier();
            let active_tab_id = match docking_manager.read() {
                Ok(docking_manager) => docking_manager.get_active_tab(&window_identifier),
                Err(_) => String::new(),
            };

            // We only need to render the active tab in windows that share the same space.
            if active_tab_id != window_identifier {
                continue;
            }

            let window_rect = {
                if let Ok(docking_manager) = docking_manager.read() {
                    docking_manager
                        .find_window_rect(window_identifier)
                        .map(|(x, y, w, h)| {
                            let offset = available_size_rect.min;
                            Rect::from_min_size(pos2(offset.x + x as f32, offset.y + y as f32), vec2(w as f32, h as f32))
                        })
                } else {
                    None
                }
            };

            if let Some(window_rect) = window_rect {
                let builder = UiBuilder::new().max_rect(window_rect);
                let mut child_user_interface = user_interface.new_child(builder);

                window.ui(&mut child_user_interface);
            }
        }

        if let Some(dock_drag_overlay) = dock_drag_overlay {
            paint_drag_overlay(user_interface, theme, &dock_drag_overlay);

            if dock_drag_overlay.hovered_drop_target.is_some() {
                user_interface.ctx().set_cursor_icon(CursorIcon::Move);
            } else {
                user_interface.ctx().set_cursor_icon(CursorIcon::NoDrop);
            }
        }

        response
    }
}

fn paint_drag_overlay(
    user_interface: &Ui,
    theme: &crate::ui::theme::Theme,
    dock_drag_overlay: &DockDragOverlay,
) {
    for dock_drop_zone in dock_drag_overlay.drop_zones.iter() {
        if dock_drop_zone.is_hovered {
            paint_drop_preview(user_interface, theme, dock_drop_zone);
        }
    }

    for dock_drop_zone in dock_drag_overlay.drop_zones.iter() {
        paint_drop_zone_button(user_interface, theme, dock_drop_zone);
    }
}

fn paint_drop_preview(
    user_interface: &Ui,
    theme: &crate::ui::theme::Theme,
    dock_drop_zone: &DockDropZone,
) {
    let preview_fill = Color32::from_rgba_unmultiplied(theme.selected_border.r(), theme.selected_border.g(), theme.selected_border.b(), 0x52);
    let preview_sheen = Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x24);
    let preview_sheen_rect = Rect::from_min_max(
        dock_drop_zone.preview_rect.min,
        pos2(dock_drop_zone.preview_rect.max.x, dock_drop_zone.preview_rect.center().y),
    )
    .shrink2(vec2(4.0, 4.0));

    user_interface
        .painter()
        .rect_filled(dock_drop_zone.preview_rect, CornerRadius::same(8), preview_fill);
    user_interface.painter().rect_stroke(
        dock_drop_zone.preview_rect,
        CornerRadius::same(8),
        Stroke::new(1.5, theme.selected_border),
        StrokeKind::Outside,
    );
    user_interface
        .painter()
        .rect_filled(preview_sheen_rect, CornerRadius::same(6), preview_sheen);
}

fn paint_drop_zone_button(
    user_interface: &Ui,
    theme: &crate::ui::theme::Theme,
    dock_drop_zone: &DockDropZone,
) {
    let button_fill = if dock_drop_zone.is_hovered {
        Color32::from_rgba_unmultiplied(theme.selected_border.r(), theme.selected_border.g(), theme.selected_border.b(), 0xC0)
    } else {
        Color32::from_rgba_unmultiplied(theme.selected_border.r(), theme.selected_border.g(), theme.selected_border.b(), 0x84)
    };
    let button_sheen = if dock_drop_zone.is_hovered {
        Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x30)
    } else {
        Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x18)
    };
    let button_sheen_rect = Rect::from_min_max(
        dock_drop_zone.button_rect.min,
        pos2(dock_drop_zone.button_rect.max.x, dock_drop_zone.button_rect.center().y),
    )
    .shrink2(vec2(2.0, 2.0));

    user_interface
        .painter()
        .rect_filled(dock_drop_zone.button_rect, CornerRadius::same(6), button_fill);
    user_interface.painter().rect_stroke(
        dock_drop_zone.button_rect,
        CornerRadius::same(6),
        Stroke::new(1.0, theme.selected_border),
        StrokeKind::Outside,
    );
    user_interface
        .painter()
        .rect_filled(button_sheen_rect, CornerRadius::same(5), button_sheen);
}
