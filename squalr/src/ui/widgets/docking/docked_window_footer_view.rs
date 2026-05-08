use crate::{
    app_context::AppContext,
    models::docking::{drag_drop::dock_tab_drop_target::DockTabDropTarget, hierarchy::types::dock_tab_insertion_direction::DockTabInsertionDirection},
    ui::{
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

#[derive(Clone, Debug, PartialEq)]
struct DockedTabLayoutItem {
    sibling_id: String,
    title: String,
    tab_width: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct DockedTabLayoutRow {
    items: Vec<DockedTabLayoutItem>,
    row_width: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct DockedTabLayout {
    rows: Vec<DockedTabLayoutRow>,
}

#[derive(Clone)]
pub struct DockedWindowFooterView {
    app_context: Arc<AppContext>,
    dock_view_data: Arc<DockRootViewData>,
    identifier: Rc<String>,
}

impl DockedWindowFooterView {
    const TAB_ROW_HEIGHT: f32 = 24.0;
    const TAB_HORIZONTAL_PADDING: f32 = 10.0;
    const TAB_MIN_WIDTH: f32 = 96.0;
    const TAB_MAX_WIDTH: f32 = 160.0;
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
        }
    }

    pub fn get_height(
        &self,
        user_interface: &Ui,
        available_width: f32,
    ) -> f32 {
        let sibling_ids = match self.app_context.docking_manager.read() {
            Ok(docking_manager_guard) => docking_manager_guard.get_sibling_tab_ids(&self.identifier, true),
            Err(error) => {
                log::error!("Failed to acquire docking manager lock while resolving dock footer height: {}", error);
                Vec::new()
            }
        };
        let windows = match self.dock_view_data.windows.read() {
            Ok(windows) => windows,
            Err(error) => {
                log::error!("Failed to acquire windows lock while resolving dock footer height: {}", error);
                return Self::TAB_ROW_HEIGHT;
            }
        };
        let tab_layout = self.resolve_tab_layout(user_interface, &sibling_ids, windows.as_slice(), available_width);

        Self::height_for_row_count(tab_layout.rows.len())
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

    fn height_for_row_count(row_count: usize) -> f32 {
        Self::TAB_ROW_HEIGHT * row_count.max(1) as f32
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

    fn build_tab_layout_items(
        &self,
        user_interface: &Ui,
        sibling_ids: &[String],
        windows: &[Box<dyn crate::ui::widgets::docking::dockable_window::DockableWindow>],
    ) -> Vec<DockedTabLayoutItem> {
        let theme = &self.app_context.theme;

        sibling_ids
            .iter()
            .map(|sibling_id| {
                let title = Self::resolve_window_title(windows, sibling_id)
                    .unwrap_or(sibling_id.as_str())
                    .to_string();

                DockedTabLayoutItem {
                    sibling_id: sibling_id.clone(),
                    tab_width: Self::measure_tab_width(user_interface, theme, &title),
                    title,
                }
            })
            .collect()
    }

    fn resolve_minimum_row_count(
        tab_layout_items: &[DockedTabLayoutItem],
        available_width: f32,
    ) -> usize {
        if tab_layout_items.is_empty() {
            return 1;
        }

        let available_width = available_width.max(1.0);
        let mut row_count = 1;
        let mut current_row_width = 0.0;

        for tab_layout_item in tab_layout_items {
            if current_row_width > 0.0 && current_row_width + tab_layout_item.tab_width > available_width {
                row_count += 1;
                current_row_width = tab_layout_item.tab_width;
            } else {
                current_row_width += tab_layout_item.tab_width;
            }
        }

        row_count
    }

    fn row_width(tab_layout_items: &[DockedTabLayoutItem]) -> f32 {
        tab_layout_items
            .iter()
            .map(|tab_layout_item| tab_layout_item.tab_width)
            .sum()
    }

    fn resolve_balanced_tab_rows(
        tab_layout_items: &[DockedTabLayoutItem],
        available_width: f32,
    ) -> Vec<DockedTabLayoutRow> {
        if tab_layout_items.is_empty() {
            return Vec::new();
        }

        let available_width = available_width.max(1.0);
        let row_count = Self::resolve_minimum_row_count(tab_layout_items, available_width);
        let mut rows = Vec::with_capacity(row_count);
        let mut first_remaining_tab_index = 0;

        for row_number in 0..row_count {
            let remaining_tab_count = tab_layout_items.len().saturating_sub(first_remaining_tab_index);
            let remaining_row_count = row_count.saturating_sub(row_number).max(1);
            let maximum_row_tab_count = remaining_tab_count.saturating_sub(remaining_row_count.saturating_sub(1));
            let mut row_tab_count = remaining_tab_count
                .div_ceil(remaining_row_count)
                .min(maximum_row_tab_count)
                .max(1);

            while row_tab_count > 1
                && Self::row_width(&tab_layout_items[first_remaining_tab_index..first_remaining_tab_index + row_tab_count]) > available_width
            {
                row_tab_count -= 1;
            }

            let last_remaining_tab_index = first_remaining_tab_index + row_tab_count;
            let items = tab_layout_items[first_remaining_tab_index..last_remaining_tab_index].to_vec();
            let row_width = Self::row_width(&items);
            rows.push(DockedTabLayoutRow { items, row_width });
            first_remaining_tab_index = last_remaining_tab_index;
        }

        rows
    }

    fn resolve_tab_layout(
        &self,
        user_interface: &Ui,
        sibling_ids: &[String],
        windows: &[Box<dyn crate::ui::widgets::docking::dockable_window::DockableWindow>],
        available_width: f32,
    ) -> DockedTabLayout {
        let tab_layout_items = self.build_tab_layout_items(user_interface, sibling_ids, windows);
        let rows = Self::resolve_balanced_tab_rows(&tab_layout_items, available_width);

        DockedTabLayout { rows }
    }
}

impl Widget for DockedWindowFooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let footer_width = user_interface.available_size().x;
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
                let (_, response) = user_interface.allocate_exact_size(vec2(footer_width, Self::TAB_ROW_HEIGHT), Sense::empty());
                return response;
            }
        };
        let windows = match self.dock_view_data.windows.read() {
            Ok(windows) => windows,
            Err(error) => {
                log::error!("Failed to acquire windows lock: {}", error);
                let (_, response) = user_interface.allocate_exact_size(vec2(footer_width, Self::TAB_ROW_HEIGHT), Sense::empty());
                return response;
            }
        };
        let tab_layout = self.resolve_tab_layout(user_interface, &sibling_ids, windows.as_slice(), footer_width);
        let footer_height = Self::height_for_row_count(tab_layout.rows.len());
        let (available_size_rect, response) = user_interface.allocate_exact_size(vec2(footer_width, footer_height), Sense::empty());

        user_interface
            .painter()
            .rect_filled(available_size_rect, CornerRadius::ZERO, theme.background_primary);

        let mut selected_tab_id = None;
        let mut toggled_maximized_tab_id = None;
        let mut drag_start_request = None;
        let mut hovered_tab_drop_target = None;

        for (row_number, tab_layout_row) in tab_layout.rows.iter().enumerate() {
            let row_min = pos2(available_size_rect.min.x, available_size_rect.min.y + row_number as f32 * Self::TAB_ROW_HEIGHT);
            let row_rect = Rect::from_min_size(row_min, vec2(available_size_rect.width(), Self::TAB_ROW_HEIGHT));
            let row_content_rect = Rect::from_min_size(row_min, vec2(tab_layout_row.row_width.max(row_rect.width()), Self::TAB_ROW_HEIGHT));
            let mut tab_strip_user_interface = user_interface.new_child(
                UiBuilder::new()
                    .max_rect(row_content_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            tab_strip_user_interface.set_clip_rect(row_rect);

            for tab_layout_item in &tab_layout_row.items {
                let mut button = Button::new_from_theme(theme)
                    .background_color(theme.background_control_secondary)
                    .border_color(theme.submenu_border)
                    .border_width(1.0);

                if tab_layout_item.sibling_id.as_str() == active_tab_id {
                    button.backgorund_color = theme.background_control_primary;
                    button.border_color = theme.background_control_primary_light;
                }

                if active_dragged_window_identifier.as_deref() == Some(tab_layout_item.sibling_id.as_str()) {
                    button.backgorund_color = theme.selected_background;
                    button.border_color = theme.selected_border;
                }

                let mut tab_attention_state = self
                    .dock_view_data
                    .get_tab_attention_state(&tab_layout_item.sibling_id);
                if tab_layout_item.sibling_id.as_str() == active_tab_id
                    && tab_attention_state
                        .as_ref()
                        .is_some_and(|tab_attention_state| !tab_attention_state.get_force_when_visible())
                {
                    self.dock_view_data
                        .clear_tab_attention(&tab_layout_item.sibling_id);
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
                        vec2(tab_layout_item.tab_width, Self::TAB_ROW_HEIGHT),
                        button
                            .corner_radius(CornerRadius::ZERO)
                            .sense(Sense::click_and_drag()),
                    )
                    .on_hover_cursor(CursorIcon::Grab);

                if response.rect.is_positive() {
                    Self::draw_tab_label(&tab_strip_user_interface, theme, response.rect, &tab_layout_item.title, row_rect);
                }

                if response.clicked() {
                    selected_tab_id = Some(tab_layout_item.sibling_id.clone());
                    self.dock_view_data
                        .clear_tab_attention(&tab_layout_item.sibling_id);
                }

                if response.double_clicked() {
                    selected_tab_id = Some(tab_layout_item.sibling_id.clone());
                    toggled_maximized_tab_id = Some(tab_layout_item.sibling_id.clone());
                    self.dock_view_data
                        .clear_tab_attention(&tab_layout_item.sibling_id);
                }

                if response.drag_started() {
                    let pointer_press_origin = tab_strip_user_interface
                        .input(|input_state| input_state.pointer.press_origin())
                        .or_else(|| response.interact_pointer_pos());

                    if let Some(pointer_press_origin) = pointer_press_origin {
                        drag_start_request = Some((tab_layout_item.sibling_id.clone(), pointer_press_origin));
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
                        tab_layout_item.sibling_id.as_str(),
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
    let preview_width = safe_clamp_f32(target_tab_rect.width() * 0.07, 6.0, 10.0);

    match tab_insertion_direction {
        DockTabInsertionDirection::BeforeTarget => Rect::from_min_max(target_tab_rect.min, pos2(target_tab_rect.min.x + preview_width, target_tab_rect.max.y)),
        DockTabInsertionDirection::AfterTarget => Rect::from_min_max(pos2(target_tab_rect.max.x - preview_width, target_tab_rect.min.y), target_tab_rect.max),
    }
}

#[cfg(test)]
mod tests {
    use super::{DockedTabLayoutItem, DockedWindowFooterView, build_tab_drop_preview_rect, resolve_tab_drop_target};
    use crate::models::docking::hierarchy::types::dock_tab_insertion_direction::DockTabInsertionDirection;
    use epaint::{Rect, pos2};

    fn build_tab_layout_item(
        tab_index: usize,
        tab_width: f32,
    ) -> DockedTabLayoutItem {
        DockedTabLayoutItem {
            sibling_id: format!("tab_{tab_index}"),
            title: format!("Tab {tab_index}"),
            tab_width,
        }
    }

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
    fn balanced_tab_rows_split_four_tabs_two_and_two() {
        let tab_layout_items = (0..4)
            .map(|tab_index| build_tab_layout_item(tab_index, 100.0))
            .collect::<Vec<_>>();
        let tab_layout_rows = DockedWindowFooterView::resolve_balanced_tab_rows(&tab_layout_items, 300.0);

        let row_lengths = tab_layout_rows
            .iter()
            .map(|tab_layout_row| tab_layout_row.items.len())
            .collect::<Vec<_>>();
        assert_eq!(row_lengths, vec![2, 2]);
    }

    #[test]
    fn balanced_tab_rows_allow_two_and_one_for_three_tabs() {
        let tab_layout_items = (0..3)
            .map(|tab_index| build_tab_layout_item(tab_index, 100.0))
            .collect::<Vec<_>>();
        let tab_layout_rows = DockedWindowFooterView::resolve_balanced_tab_rows(&tab_layout_items, 200.0);

        let row_lengths = tab_layout_rows
            .iter()
            .map(|tab_layout_row| tab_layout_row.items.len())
            .collect::<Vec<_>>();
        assert_eq!(row_lengths, vec![2, 1]);
    }

    #[test]
    fn balanced_tab_rows_preserve_order() {
        let tab_layout_items = (0..5)
            .map(|tab_index| build_tab_layout_item(tab_index, 100.0))
            .collect::<Vec<_>>();
        let tab_layout_rows = DockedWindowFooterView::resolve_balanced_tab_rows(&tab_layout_items, 250.0);
        let ordered_tab_ids = tab_layout_rows
            .iter()
            .flat_map(|tab_layout_row| {
                tab_layout_row
                    .items
                    .iter()
                    .map(|tab_layout_item| tab_layout_item.sibling_id.as_str())
            })
            .collect::<Vec<_>>();

        assert_eq!(ordered_tab_ids, vec!["tab_0", "tab_1", "tab_2", "tab_3", "tab_4"]);
    }
}
