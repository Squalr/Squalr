use super::row_icon_button::render_row_icon_button_at;
use crate::app_context::AppContext;
use crate::ui::text::text_fitting::{measure_text_width, truncate_text_to_width};
use crate::ui::widgets::controls::state_layer::StateLayer;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::SymbolLayoutEditorViewData;
use eframe::egui::{Align2, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::symbol_layouts::symbol_layout_draft_ops::SymbolLayoutUnassignedRowContext;
use std::sync::Arc;

use super::super::{SymbolLayoutEditorView, SymbolLayoutUnassignedRowAction};

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct SymbolLayoutUnassignedRowView<'view> {
    app_context: Arc<AppContext>,
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    layout_id: Option<&'view str>,
    row_context: &'view SymbolLayoutUnassignedRowContext,
    can_show_context_menu: bool,
    can_define_field: bool,
    is_selected: bool,
}

impl<'view> SymbolLayoutUnassignedRowView<'view> {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn new(
        app_context: Arc<AppContext>,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        layout_id: Option<&'view str>,
        row_context: &'view SymbolLayoutUnassignedRowContext,
        can_show_context_menu: bool,
        can_define_field: bool,
        is_selected: bool,
    ) -> Self {
        Self {
            app_context,
            symbol_layout_editor_view_data,
            layout_id,
            row_context,
            can_show_context_menu,
            can_define_field,
            is_selected,
        }
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn show(
        self,
        user_interface: &mut Ui,
    ) -> Option<SymbolLayoutUnassignedRowAction> {
        if self.row_context.size_in_bytes == 0 {
            return None;
        }

        let theme = &self.app_context.theme;
        let can_move_up = self.row_context.move_up_field.is_some() || self.row_context.move_up_unassigned_span.is_some();
        let can_move_down = self.row_context.move_down_field.is_some() || self.row_context.move_down_unassigned_span.is_some();
        let row_sense = if self.can_show_context_menu || can_move_up || can_move_down {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (row_rect, row_response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), SymbolLayoutEditorView::FIELD_ROW_HEIGHT), row_sense);
        let mut pending_unassigned_row_action = None;

        if self.is_selected {
            user_interface
                .painter()
                .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface
                .painter()
                .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
        }

        StateLayer {
            bounds_min: row_rect.min,
            bounds_max: row_rect.max,
            enabled: true,
            pressed: row_response.is_pointer_button_down_on(),
            has_hover: row_response.hovered(),
            has_focus: row_response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        let button_area_width = SymbolLayoutEditorView::ICON_BUTTON_WIDTH * 2.0;
        let button_area_left = (row_rect.max.x - button_area_width).max(row_rect.min.x);
        let mut button_min_x = button_area_left;
        let mut render_next_button = |icon_handle: &TextureHandle, tooltip_text: &str, is_disabled: bool| -> Response {
            let button_rect = Rect::from_min_size(
                pos2(button_min_x, row_rect.min.y),
                vec2(SymbolLayoutEditorView::ICON_BUTTON_WIDTH, SymbolLayoutEditorView::FIELD_ROW_HEIGHT),
            );
            button_min_x += SymbolLayoutEditorView::ICON_BUTTON_WIDTH;

            render_row_icon_button_at(&self.app_context, user_interface, button_rect, icon_handle, tooltip_text, is_disabled)
        };

        let move_up_response = render_next_button(
            &theme.icon_library.icon_handle_navigation_up_arrow_small,
            "Move this unassigned span up.",
            !can_move_up,
        );
        if move_up_response.clicked() {
            pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::MoveUp);
        }

        let move_down_response = render_next_button(
            &theme.icon_library.icon_handle_navigation_down_arrow_small,
            "Move this unassigned span down.",
            !can_move_down,
        );
        if move_down_response.clicked() {
            pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::MoveDown);
        }

        if self.can_show_context_menu && row_response.secondary_clicked() {
            let position = row_response
                .interact_pointer_pos()
                .unwrap_or_else(|| pos2(row_rect.left(), row_rect.center().y));
            SymbolLayoutEditorViewData::show_unassigned_context_menu_for_layout(
                self.symbol_layout_editor_view_data.clone(),
                self.layout_id.map(str::to_string),
                self.row_context.offset_in_bytes,
                self.row_context.size_in_bytes,
                position,
                self.row_context.merge_above_span.clone(),
                self.row_context.merge_below_span.clone(),
            );
            if pending_unassigned_row_action.is_none() {
                pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::SelectSpan);
            }
        } else if self.can_show_context_menu && row_response.clicked() && pending_unassigned_row_action.is_none() {
            pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::SelectSpan);
        } else if row_response.clicked() && pending_unassigned_row_action.is_none() {
            SymbolLayoutEditorViewData::hide_unassigned_context_menu(self.symbol_layout_editor_view_data.clone());
            SymbolLayoutEditorViewData::hide_field_context_menu(self.symbol_layout_editor_view_data.clone());
        }

        let left_text = format!("UNASSIGNED[{}]", self.row_context.size_in_bytes);
        let right_text = format!("0x{:X}", self.row_context.offset_in_bytes);
        let label_position = pos2(row_rect.min.x + SymbolLayoutEditorView::FIELD_ROW_LEFT_PADDING, row_rect.center().y);
        let right_text_x = button_area_left - SymbolLayoutEditorView::FIELD_ROW_LEFT_PADDING;
        let left_max_width = (right_text_x - label_position.x).max(0.0);
        let left_color = if self.can_define_field { theme.foreground } else { theme.foreground_preview };
        let left_text = truncate_text_to_width(
            user_interface,
            &left_text,
            &theme.font_library.font_noto_sans.font_normal,
            left_color,
            left_max_width,
        );
        let left_width = measure_text_width(user_interface, &left_text, &theme.font_library.font_noto_sans.font_normal, left_color);
        let right_max_width = (right_text_x - label_position.x - left_width - SymbolLayoutEditorView::FIELD_ROW_PREVIEW_GAP).max(0.0);
        let right_text = truncate_text_to_width(
            user_interface,
            &right_text,
            &theme.font_library.font_noto_sans.font_small,
            theme.foreground_preview,
            right_max_width,
        );

        user_interface.painter().text(
            label_position,
            Align2::LEFT_CENTER,
            left_text,
            theme.font_library.font_noto_sans.font_normal.clone(),
            left_color,
        );
        if !right_text.is_empty() {
            user_interface.painter().text(
                pos2(right_text_x, row_rect.center().y),
                Align2::RIGHT_CENTER,
                right_text,
                theme.font_library.font_noto_sans.font_small.clone(),
                theme.foreground_preview,
            );
        }

        pending_unassigned_row_action
    }
}
