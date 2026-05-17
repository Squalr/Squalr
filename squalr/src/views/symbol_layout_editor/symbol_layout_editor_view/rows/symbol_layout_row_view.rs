use super::row_icon_button::render_row_icon_button;
use crate::{app_context::AppContext, ui::widgets::controls::state_layer::StateLayer};
use eframe::egui::{Align, Align2, Layout, RichText, Sense, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicLayoutKind;
use std::sync::Arc;

use super::super::SymbolLayoutEditorView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) enum SymbolLayoutRowAction {
    Select,
    Open,
    Rename,
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct SymbolLayoutRowView<'view> {
    app_context: Arc<AppContext>,
    layout_id: &'view str,
    layout_kind: SymbolicLayoutKind,
    field_count: usize,
    usage_count: usize,
    is_selected: bool,
}

impl<'view> SymbolLayoutRowView<'view> {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn new(
        app_context: Arc<AppContext>,
        layout_id: &'view str,
        layout_kind: SymbolicLayoutKind,
        field_count: usize,
        usage_count: usize,
        is_selected: bool,
    ) -> Self {
        Self {
            app_context,
            layout_id,
            layout_kind,
            field_count,
            usage_count,
            is_selected,
        }
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn show(
        self,
        user_interface: &mut Ui,
    ) -> Option<SymbolLayoutRowAction> {
        let theme = &self.app_context.theme;
        let (row_rect, row_response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), SymbolLayoutEditorView::LIST_ROW_HEIGHT), Sense::click());
        let mut row_action = None;

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
            has_focus: false,
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        user_interface.painter().text(
            pos2(row_rect.min.x + 8.0, row_rect.center().y),
            Align2::LEFT_CENTER,
            self.layout_id,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(row_rect)
                .layout(Layout::right_to_left(Align::Center)),
        );
        row_user_interface.set_clip_rect(row_rect);

        let rename_response = render_row_icon_button(
            &self.app_context,
            &mut row_user_interface,
            &theme.icon_library.icon_handle_common_edit,
            "Rename this symbol layout.",
            false,
            SymbolLayoutEditorView::ICON_BUTTON_WIDTH,
            SymbolLayoutEditorView::FIELD_ROW_HEIGHT,
        );
        if rename_response.clicked() {
            row_action = Some(SymbolLayoutRowAction::Rename);
        }

        row_user_interface.add_space(SymbolLayoutEditorView::FIELD_INPUT_SPACING);
        let entry_count_label = if self.layout_kind.is_union() { "variants" } else { "fields" };
        row_user_interface.label(
            RichText::new(format!(
                "{} | {} {} | {} uses",
                self.layout_kind.label(),
                self.field_count,
                entry_count_label,
                self.usage_count
            ))
            .color(if self.is_selected { theme.foreground } else { theme.foreground_preview }),
        );

        if row_response.double_clicked() && row_action.is_none() {
            row_action = Some(SymbolLayoutRowAction::Open);
        } else if row_response.clicked() && row_action.is_none() {
            row_action = Some(SymbolLayoutRowAction::Select);
        }

        row_action
    }
}
