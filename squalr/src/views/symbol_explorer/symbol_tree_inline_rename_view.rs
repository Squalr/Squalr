use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::state_layer::StateLayer},
    views::symbol_explorer::view_data::symbol_tree_entry::SymbolTreeEntry,
};
use eframe::egui::{
    Id, Key, Rect, Response, Sense, TextEdit, Ui, UiBuilder, Widget, pos2,
    text::{CCursor, CCursorRange},
    vec2,
};
use epaint::{CornerRadius, Stroke, StrokeKind};
use std::sync::Arc;

pub struct SymbolTreeInlineRenameView<'lifetime> {
    app_context: Arc<AppContext>,
    symbol_key: &'lifetime str,
    symbol_tree_entry: &'lifetime SymbolTreeEntry,
    rename_text: &'lifetime mut String,
    should_highlight_text: &'lifetime mut bool,
    is_selected: bool,
}

pub struct SymbolTreeInlineRenameViewResponse {
    pub row_response: Response,
    pub should_commit: bool,
    pub should_cancel: bool,
}

impl<'lifetime> SymbolTreeInlineRenameView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        symbol_key: &'lifetime str,
        symbol_tree_entry: &'lifetime SymbolTreeEntry,
        rename_text: &'lifetime mut String,
        should_highlight_text: &'lifetime mut bool,
        is_selected: bool,
    ) -> Self {
        Self {
            app_context,
            symbol_key,
            symbol_tree_entry,
            rename_text,
            should_highlight_text,
            is_selected,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> SymbolTreeInlineRenameViewResponse {
        let theme = &self.app_context.theme;
        let row_left_padding = 8.0;
        let tree_level_indent = 18.0;
        let text_left_padding = 4.0;
        let expand_arrow_size = vec2(10.0, 10.0);
        let right_preview_padding = 8.0;
        let row_height = 28.0;
        let (allocated_size_rectangle, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, row_height), Sense::hover());

        if self.is_selected {
            user_interface
                .painter()
                .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.selected_background);

            user_interface.painter().rect_stroke(
                allocated_size_rectangle,
                CornerRadius::ZERO,
                Stroke::new(1.0, theme.selected_border),
                StrokeKind::Inside,
            );
        }

        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
            enabled: true,
            pressed: false,
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

        let indentation = self.symbol_tree_entry.get_depth() as f32 * tree_level_indent;
        let arrow_center = pos2(
            allocated_size_rectangle.min.x + row_left_padding + indentation + expand_arrow_size.x * 0.5,
            allocated_size_rectangle.center().y,
        );

        if self.symbol_tree_entry.can_expand() {
            let expand_icon = if self.symbol_tree_entry.is_expanded() {
                &theme.icon_library.icon_handle_navigation_down_arrow_small
            } else {
                &theme.icon_library.icon_handle_navigation_right_arrow_small
            };

            IconDraw::draw_sized(user_interface, arrow_center, expand_arrow_size, expand_icon);
        }

        let text_rect = Rect::from_min_max(
            pos2(
                allocated_size_rectangle.min.x + row_left_padding + indentation + expand_arrow_size.x + text_left_padding,
                allocated_size_rectangle.min.y + 3.0,
            ),
            pos2(allocated_size_rectangle.max.x - right_preview_padding, allocated_size_rectangle.max.y - 3.0),
        );
        let text_edit_id = Id::new(("symbol_explorer_inline_rename_editor", self.symbol_key));
        let mut text_edit_user_interface = user_interface.new_child(UiBuilder::new().max_rect(text_rect));
        text_edit_user_interface.set_clip_rect(text_rect);
        let mut output = TextEdit::singleline(self.rename_text)
            .id_salt(text_edit_id)
            .font(theme.font_library.font_noto_sans.font_normal.clone())
            .background_color(theme.background_control)
            .text_color(theme.foreground)
            .desired_width(text_rect.width())
            .show(&mut text_edit_user_interface);
        let text_edit_response = output.response.clone();

        if *self.should_highlight_text {
            let rename_text_length = self.rename_text.chars().count();

            text_edit_response.request_focus();
            output
                .state
                .cursor
                .set_char_range(Some(CCursorRange::two(CCursor::new(0), CCursor::new(rename_text_length))));
            output.state.store(user_interface.ctx(), text_edit_response.id);
            *self.should_highlight_text = false;
        }

        SymbolTreeInlineRenameViewResponse {
            row_response,
            should_commit: text_edit_response.lost_focus() && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)),
            should_cancel: (text_edit_response.has_focus() && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)))
                || (text_edit_response.lost_focus()
                    && user_interface.input(|input_state| input_state.pointer.any_click())
                    && !user_interface.input(|input_state| input_state.key_pressed(Key::Enter))),
        }
    }
}
