use crate::{
    app_context::AppContext,
    models::docking::{drag_drop::dock_tab_drop_target::DockTabDropTarget, hierarchy::types::dock_tab_insertion_direction::DockTabInsertionDirection},
    ui::{
        draw::icon_draw::IconDraw,
        geometry::safe_clamp_f32,
        widgets::{
            controls::button::Button,
            docking::{
                dock_root_view_data::DockRootViewData,
                dock_tab_attention_state::{DockTabAttentionKind, DockTabAttentionState},
            },
        },
    },
};
use eframe::egui::{Align, CursorIcon, Layout, Response, Sense, Ui, UiBuilder, Widget};
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
    const ARROW_BUTTON_WIDTH: f32 = 24.0;
    const TAB_HORIZONTAL_PADDING: f32 = 10.0;
    const TAB_MIN_WIDTH: f32 = 96.0;
    const TAB_MAX_WIDTH: f32 = 160.0;
    const TAB_SCROLL_STEP: f32 = 128.0;
    const ATTENTION_PULSE_PERIOD_SECS: f32 = 2.6;

    fn lerp_color(
        from: Color32,
        to: Color32,
        factor: f32,
    ) -> Color32 {
        let clamped_factor = safe_clamp_f32(factor, 0.0, 1.0);
        let lerp_channel = |from_channel: u8, to_channel: u8| -> u8 {
            let from_channel = from_channel as f32;
            let to_channel = to_channel as f32;

            safe_clamp_f32((from_channel + (to_channel - from_channel) * clamped_factor).round(), 0.0, 255.0) as u8
        };

        Color32::from_rgba_unmultiplied(
            lerp_channel(from.r(), to.r()),
            lerp_channel(from.g(), to.g()),
            lerp_channel(from.b(), to.b()),
            lerp_channel(from.a(), to.a()),
        )
    }

    fn resolve_attention_colors(
        theme: &crate::ui::theme::Theme,
        attention_state: &DockTabAttentionState,
        base_background_color: Color32,
        base_border_color: Color32,
    ) -> (Color32, Color32) {
        let (target_background_color, target_border_color) = match attention_state.get_attention_kind() {
            DockTabAttentionKind::Warning => (theme.background_control_warning, theme.background_control_warning_dark),
            DockTabAttentionKind::Danger => (theme.background_control_danger, theme.background_control_danger_dark),
        };
        let pulse_elapsed_secs = attention_state.get_requested_at().elapsed().as_secs_f32();
        let pulse_cycle = ((pulse_elapsed_secs / Self::ATTENTION_PULSE_PERIOD_SECS) * std::f32::consts::TAU).sin();
        let pulse_factor = 0.14 + ((pulse_cycle * 0.5) + 0.5) * 0.26;
        let border_factor = (pulse_factor + 0.14).min(0.6);

        (
            Self::lerp_color(base_background_color, target_background_color, pulse_factor),
            Self::lerp_color(base_border_color, target_border_color, border_factor),
        )
    }

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

    fn build_tab_group_key(sibling_ids: &[String]) -> String {
        sibling_ids.join("|")
    }

    fn resolve_window_title<'window>(
        windows: &'window [Box<dyn crate::ui::widgets::docking::dockable_window::DockableWindow>],
        sibling_id: &str,
    ) -> Option<&'window str> {
        windows
            .iter()
            .find(|window| window.get_identifier() == sibling_id)
            .map(|window| window.get_title())
    }

    fn measure_tab_width(
        user_interface: &Ui,
        theme: &crate::ui::theme::Theme,
        title: &str,
    ) -> f32 {
        let title_galley = user_interface
            .ctx()
            .fonts_mut(|fonts| fonts.layout_no_wrap(title.to_string(), theme.font_library.font_noto_sans.font_header.clone(), theme.foreground));

        safe_clamp_f32(
            title_galley.size().x + Self::TAB_HORIZONTAL_PADDING * 2.0,
            Self::TAB_MIN_WIDTH,
            Self::TAB_MAX_WIDTH,
        )
    }

    fn clamp_tab_strip_scroll_offset(
        requested_scroll_offset: f32,
        total_tab_width: f32,
        viewport_width: f32,
    ) -> f32 {
        let max_scroll_offset = (total_tab_width - viewport_width).max(0.0);

        safe_clamp_f32(requested_scroll_offset, 0.0, max_scroll_offset)
    }

    fn scroll_offset_for_visible_tab(
        scroll_offset: f32,
        tab_rect: Rect,
        viewport_width: f32,
        total_tab_width: f32,
    ) -> f32 {
        let mut next_scroll_offset = scroll_offset;

        if tab_rect.min.x < scroll_offset {
            next_scroll_offset = tab_rect.min.x;
        } else if tab_rect.max.x > scroll_offset + viewport_width {
            next_scroll_offset = tab_rect.max.x - viewport_width;
        }

        Self::clamp_tab_strip_scroll_offset(next_scroll_offset, total_tab_width, viewport_width)
    }

    fn draw_tab_label(
        user_interface: &Ui,
        theme: &crate::ui::theme::Theme,
        tab_rect: Rect,
        title: &str,
        clip_rect: Rect,
    ) {
        let title_galley = user_interface
            .ctx()
            .fonts_mut(|fonts| fonts.layout_no_wrap(title.to_string(), theme.font_library.font_noto_sans.font_header.clone(), theme.foreground));
        let text_position = pos2(
            tab_rect.center().x - title_galley.size().x * 0.5,
            tab_rect.center().y - title_galley.size().y * 0.5,
        );
        let effective_clip_rect = clip_rect.intersect(tab_rect.shrink2(vec2(Self::TAB_HORIZONTAL_PADDING, 0.0)));

        if !effective_clip_rect.is_positive() {
            return;
        }

        user_interface
            .painter()
            .with_clip_rect(effective_clip_rect)
            .galley(text_position, title_galley, theme.foreground);
    }

    fn render_scroll_button(
        &self,
        user_interface: &mut Ui,
        button_rect: Rect,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.put(
            button_rect,
            Button::new_from_theme(theme)
                .background_color(theme.background_control_secondary)
                .border_color(theme.submenu_border)
                .border_width(1.0)
                .with_tooltip_text(tooltip_text)
                .disabled(is_disabled),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_rect,
            icon_handle,
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        button_response
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
        let tab_titles = sibling_ids
            .iter()
            .map(|sibling_id| {
                (
                    sibling_id.clone(),
                    Self::resolve_window_title(windows.as_slice(), sibling_id)
                        .unwrap_or(sibling_id.as_str())
                        .to_string(),
                )
            })
            .collect::<Vec<_>>();
        let tab_widths = tab_titles
            .iter()
            .map(|(_, title)| Self::measure_tab_width(user_interface, theme, title))
            .collect::<Vec<_>>();
        let total_tab_width = tab_widths.iter().sum::<f32>();
        let tab_group_key = Self::build_tab_group_key(&sibling_ids);
        let is_overflowing = total_tab_width > available_size_rect.width();
        let viewport_rect = Rect::from_min_max(
            pos2(
                available_size_rect.min.x + if is_overflowing { Self::ARROW_BUTTON_WIDTH } else { 0.0 },
                available_size_rect.min.y,
            ),
            pos2(
                available_size_rect.max.x - if is_overflowing { Self::ARROW_BUTTON_WIDTH } else { 0.0 },
                available_size_rect.max.y,
            ),
        );
        let viewport_width = viewport_rect.width().max(1.0);
        let mut scroll_offset =
            Self::clamp_tab_strip_scroll_offset(self.dock_view_data.get_tab_strip_scroll_offset(&tab_group_key), total_tab_width, viewport_width);
        let active_tab_index = sibling_ids
            .iter()
            .position(|sibling_id| sibling_id == active_tab_id.as_str());

        if let Some(active_tab_index) = active_tab_index {
            let tab_start_x = tab_widths.iter().take(active_tab_index).sum::<f32>();
            let active_tab_rect = Rect::from_min_size(
                pos2(tab_start_x, available_size_rect.min.y),
                vec2(tab_widths[active_tab_index], available_size_rect.height()),
            );
            scroll_offset = Self::scroll_offset_for_visible_tab(scroll_offset, active_tab_rect, viewport_width, total_tab_width);
        }

        let max_scroll_offset = (total_tab_width - viewport_width).max(0.0);

        // Background.
        user_interface
            .painter()
            .rect_filled(available_size_rect, CornerRadius::ZERO, theme.background_primary);

        let builder = UiBuilder::new()
            .max_rect(viewport_rect)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);
        child_user_interface.set_clip_rect(viewport_rect);
        let mut selected_tab_id = None;
        let mut toggled_maximized_tab_id = None;
        let mut drag_start_request = None;
        let mut hovered_tab_drop_target = None;
        let mut selected_tab_index = None;

        let content_rect = Rect::from_min_size(
            pos2(viewport_rect.min.x - scroll_offset, viewport_rect.min.y),
            vec2(total_tab_width.max(viewport_rect.width()), available_size_rect.height()),
        );
        let mut tab_strip_user_interface = child_user_interface.new_child(
            UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        tab_strip_user_interface.set_clip_rect(viewport_rect);

        for (tab_index, ((sibling_id, title), tab_width)) in tab_titles.iter().zip(tab_widths.iter()).enumerate() {
            let mut button = Button::new_from_theme(theme)
                .background_color(theme.background_control_secondary)
                .border_color(theme.submenu_border)
                .border_width(1.0);

            if sibling_id.as_str() == active_tab_id {
                button.backgorund_color = theme.background_control_primary;
                button.border_color = theme.background_control_primary_light;
            }

            if active_dragged_window_identifier.as_deref() == Some(sibling_id.as_str()) {
                button.backgorund_color = theme.selected_background;
                button.border_color = theme.selected_border;
            }

            let mut tab_attention_state = self.dock_view_data.get_tab_attention_state(sibling_id);
            if sibling_id.as_str() == active_tab_id
                && tab_attention_state
                    .as_ref()
                    .is_some_and(|tab_attention_state| !tab_attention_state.get_force_when_visible())
            {
                self.dock_view_data.clear_tab_attention(sibling_id);
                tab_attention_state = None;
            }
            if let Some(tab_attention_state) = tab_attention_state.as_ref() {
                let (attention_background_color, attention_border_color) =
                    Self::resolve_attention_colors(theme, tab_attention_state, button.backgorund_color, button.border_color);
                button.backgorund_color = attention_background_color;
                button.border_color = attention_border_color;
                tab_strip_user_interface.ctx().request_repaint();
            }

            let response = tab_strip_user_interface
                .add_sized(
                    vec2(*tab_width, available_size_rect.height()),
                    button
                        .corner_radius(CornerRadius::ZERO)
                        .sense(Sense::click_and_drag()),
                )
                .on_hover_cursor(CursorIcon::Grab);

            if response.rect.is_positive() {
                Self::draw_tab_label(&tab_strip_user_interface, theme, response.rect, title, viewport_rect);
            }

            if response.clicked() {
                selected_tab_id = Some(sibling_id.clone());
                selected_tab_index = Some(tab_index);
                self.dock_view_data.clear_tab_attention(sibling_id);
            }

            if response.double_clicked() {
                selected_tab_id = Some(sibling_id.clone());
                selected_tab_index = Some(tab_index);
                toggled_maximized_tab_id = Some(sibling_id.clone());
                self.dock_view_data.clear_tab_attention(sibling_id);
            }

            if response.drag_started() {
                let pointer_press_origin = tab_strip_user_interface
                    .input(|input_state| input_state.pointer.press_origin())
                    .or_else(|| response.interact_pointer_pos());

                if let Some(pointer_press_origin) = pointer_press_origin {
                    drag_start_request = Some((sibling_id.clone(), pointer_press_origin));
                }

                tab_strip_user_interface.ctx().request_repaint();
            }

            if response.dragged() {
                tab_strip_user_interface
                    .ctx()
                    .set_cursor_icon(CursorIcon::Grabbing);
                tab_strip_user_interface.ctx().request_repaint();
            }

            if is_drag_drop_active {
                let resolved_tab_drop_target = resolve_tab_drop_target(
                    active_dragged_window_identifier.as_deref(),
                    sibling_id.as_str(),
                    response.rect,
                    pointer_position,
                );

                if let Some(resolved_tab_drop_target) = resolved_tab_drop_target {
                    paint_tab_drop_preview(
                        &tab_strip_user_interface,
                        theme,
                        response.rect,
                        resolved_tab_drop_target.tab_insertion_direction,
                    );
                    hovered_tab_drop_target = Some(resolved_tab_drop_target);
                }
            }
        }

        if is_overflowing {
            let left_button_rect = Rect::from_min_size(available_size_rect.min, vec2(Self::ARROW_BUTTON_WIDTH, available_size_rect.height()));
            let right_button_rect = Rect::from_min_size(
                pos2(available_size_rect.max.x - Self::ARROW_BUTTON_WIDTH, available_size_rect.min.y),
                vec2(Self::ARROW_BUTTON_WIDTH, available_size_rect.height()),
            );
            let can_scroll_left = scroll_offset > 0.0;
            let can_scroll_right = scroll_offset < max_scroll_offset;
            let scroll_left_response = self.render_scroll_button(
                user_interface,
                left_button_rect,
                &theme.icon_library.icon_handle_navigation_left_arrow_small,
                "Scroll tabs left.",
                !can_scroll_left,
            );
            let scroll_right_response = self.render_scroll_button(
                user_interface,
                right_button_rect,
                &theme.icon_library.icon_handle_navigation_right_arrow_small,
                "Scroll tabs right.",
                !can_scroll_right,
            );

            if scroll_left_response.clicked() {
                scroll_offset = Self::clamp_tab_strip_scroll_offset(scroll_offset - Self::TAB_SCROLL_STEP, total_tab_width, viewport_width);
            }

            if scroll_right_response.clicked() {
                scroll_offset = Self::clamp_tab_strip_scroll_offset(scroll_offset + Self::TAB_SCROLL_STEP, total_tab_width, viewport_width);
            }
        }

        let final_active_tab_id = selected_tab_id.as_deref().unwrap_or(active_tab_id.as_str());
        let final_active_tab_index = selected_tab_index.or_else(|| {
            sibling_ids
                .iter()
                .position(|sibling_id| sibling_id == final_active_tab_id)
        });

        if let Some(final_active_tab_index) = final_active_tab_index {
            let tab_start_x = tab_widths.iter().take(final_active_tab_index).sum::<f32>();
            let tab_rect = Rect::from_min_size(
                pos2(tab_start_x, available_size_rect.min.y),
                vec2(tab_widths[final_active_tab_index], available_size_rect.height()),
            );
            scroll_offset = Self::scroll_offset_for_visible_tab(scroll_offset, tab_rect, viewport_width, total_tab_width);
        }

        let clamped_scroll_offset = Self::clamp_tab_strip_scroll_offset(scroll_offset, total_tab_width, viewport_width);
        self.dock_view_data
            .set_tab_strip_scroll_offset(tab_group_key, clamped_scroll_offset);

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
    let preview_width = safe_clamp_f32(target_tab_rect.width() * 0.07, 6.0, 10.0);

    match tab_insertion_direction {
        DockTabInsertionDirection::BeforeTarget => Rect::from_min_max(target_tab_rect.min, pos2(target_tab_rect.min.x + preview_width, target_tab_rect.max.y)),
        DockTabInsertionDirection::AfterTarget => Rect::from_min_max(pos2(target_tab_rect.max.x - preview_width, target_tab_rect.min.y), target_tab_rect.max),
    }
}

#[cfg(test)]
mod tests {
    use super::{DockedWindowFooterView, build_tab_drop_preview_rect, resolve_tab_drop_target};
    use crate::models::docking::hierarchy::types::dock_tab_insertion_direction::DockTabInsertionDirection;
    use epaint::{Rect, pos2, vec2};

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

    #[test]
    fn clamp_tab_strip_scroll_offset_stays_in_bounds() {
        assert_eq!(DockedWindowFooterView::clamp_tab_strip_scroll_offset(-10.0, 480.0, 200.0), 0.0);
        assert_eq!(DockedWindowFooterView::clamp_tab_strip_scroll_offset(400.0, 480.0, 200.0), 280.0);
        assert_eq!(DockedWindowFooterView::clamp_tab_strip_scroll_offset(120.0, 480.0, 200.0), 120.0);
    }

    #[test]
    fn scroll_offset_for_visible_tab_keeps_hidden_tab_in_view() {
        let tab_rect = Rect::from_min_size(pos2(320.0, 0.0), vec2(96.0, 24.0));
        let next_scroll_offset = DockedWindowFooterView::scroll_offset_for_visible_tab(0.0, tab_rect, 200.0, 480.0);

        assert_eq!(next_scroll_offset, 216.0);
    }
}
