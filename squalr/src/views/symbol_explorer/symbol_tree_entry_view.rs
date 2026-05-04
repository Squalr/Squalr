use crate::{
    app_context::AppContext,
    ui::{converters::data_type_to_icon_converter::DataTypeToIconConverter, draw::icon_draw::IconDraw, widgets::controls::state_layer::StateLayer},
    views::symbol_explorer::view_data::symbol_tree_entry::{SymbolTreeEntry, SymbolTreeEntryKind},
};
use eframe::egui::{Align2, Color32, FontId, Rect, Response, Sense, Ui, Widget, pos2, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use std::sync::Arc;

pub struct SymbolTreeEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    symbol_tree_entry: &'lifetime SymbolTreeEntry,
    size_preview_text: &'lifetime str,
    size_tooltip_text: &'lifetime str,
    preview_value: &'lifetime str,
    is_selected: bool,
}

pub struct SymbolTreeEntryViewResponse {
    pub row_response: Response,
    pub did_click_row: bool,
    pub did_click_expand_arrow: bool,
}

impl<'lifetime> SymbolTreeEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        symbol_tree_entry: &'lifetime SymbolTreeEntry,
        size_preview_text: &'lifetime str,
        size_tooltip_text: &'lifetime str,
        preview_value: &'lifetime str,
        is_selected: bool,
    ) -> Self {
        Self {
            app_context,
            symbol_tree_entry,
            size_preview_text,
            size_tooltip_text,
            preview_value,
            is_selected,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> SymbolTreeEntryViewResponse {
        let theme = &self.app_context.theme;
        let row_left_padding = 8.0;
        let tree_level_indent = 18.0;
        let text_left_padding = 4.0;
        let expand_arrow_size = vec2(10.0, 10.0);
        let data_type_icon_size = vec2(16.0, 16.0);
        let data_type_icon_gap = 6.0;
        let right_preview_padding = 8.0;
        let row_height = 28.0;
        let (allocated_size_rectangle, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, row_height), Sense::click());

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

        let indentation = self.symbol_tree_entry.get_depth() as f32 * tree_level_indent;
        let arrow_center = pos2(
            allocated_size_rectangle.min.x + row_left_padding + indentation + expand_arrow_size.x * 0.5,
            allocated_size_rectangle.center().y,
        );
        let arrow_response = if self.symbol_tree_entry.can_expand() {
            let arrow_hit_box_rect = Rect::from_center_size(arrow_center, vec2(14.0, 14.0));
            let arrow_response = user_interface.interact(
                arrow_hit_box_rect,
                user_interface.make_persistent_id(("symbol_tree_entry_arrow", self.symbol_tree_entry.get_node_key())),
                Sense::click(),
            );
            let expand_icon = if self.symbol_tree_entry.is_expanded() {
                &theme.icon_library.icon_handle_navigation_down_arrow_small
            } else {
                &theme.icon_library.icon_handle_navigation_right_arrow_small
            };

            IconDraw::draw_sized(user_interface, arrow_center, expand_arrow_size, expand_icon);
            Some(arrow_response)
        } else {
            None
        };

        let data_type_icon_center = pos2(
            allocated_size_rectangle.min.x + row_left_padding + indentation + expand_arrow_size.x + text_left_padding + data_type_icon_size.x * 0.5,
            allocated_size_rectangle.center().y,
        );
        let data_type_icon = DataTypeToIconConverter::convert_data_type_to_icon(&self.symbol_tree_entry.get_display_type_id(), &theme.icon_library);
        IconDraw::draw_sized(user_interface, data_type_icon_center, data_type_icon_size, &data_type_icon);

        let text_position_x = data_type_icon_center.x + data_type_icon_size.x * 0.5 + data_type_icon_gap;
        let text_position = pos2(text_position_x, allocated_size_rectangle.center().y);
        let preview_position = pos2(allocated_size_rectangle.max.x - right_preview_padding, allocated_size_rectangle.center().y);
        let display_name_font = theme.font_library.font_noto_sans.font_normal.clone();
        let size_preview_font = theme.font_library.font_noto_sans.font_small.clone();
        let preview_value_font = theme.font_library.font_noto_sans.font_small.clone();
        let max_preview_text_width = (allocated_size_rectangle.max.x - text_position.x - 24.0).max(0.0);
        let preview_value_text = Self::truncate_text_to_width(
            user_interface,
            self.preview_value,
            &preview_value_font,
            theme.foreground_preview,
            max_preview_text_width,
        );
        let preview_value_width = Self::measure_text_width(user_interface, &preview_value_text, &preview_value_font, theme.foreground_preview);
        let left_text_max_x = preview_position.x - preview_value_width - 12.0;
        let max_left_text_width = (left_text_max_x - text_position.x).max(0.0);
        let display_name_width = Self::measure_text_width(user_interface, self.symbol_tree_entry.get_display_name(), &display_name_font, theme.foreground);

        let display_name_text = if self.size_preview_text.is_empty() || display_name_width >= max_left_text_width {
            Self::truncate_text_to_width(
                user_interface,
                self.symbol_tree_entry.get_display_name(),
                &display_name_font,
                theme.foreground,
                max_left_text_width,
            )
        } else {
            self.symbol_tree_entry.get_display_name().to_string()
        };
        let display_name_text_width = Self::measure_text_width(user_interface, &display_name_text, &display_name_font, theme.foreground);
        user_interface.painter().text(
            text_position,
            Align2::LEFT_CENTER,
            display_name_text,
            display_name_font.clone(),
            theme.foreground,
        );

        if !self.size_preview_text.is_empty() && display_name_text_width < max_left_text_width {
            let size_preview_gap = 10.0;
            let size_preview_position = pos2(
                text_position.x + display_name_text_width + size_preview_gap,
                allocated_size_rectangle.center().y,
            );
            let max_size_preview_width = (max_left_text_width - display_name_text_width - size_preview_gap).max(0.0);
            let size_preview_text = Self::truncate_text_to_width(
                user_interface,
                self.size_preview_text,
                &size_preview_font,
                theme.foreground_preview,
                max_size_preview_width,
            );

            if !size_preview_text.is_empty() {
                user_interface.painter().text(
                    size_preview_position,
                    Align2::LEFT_CENTER,
                    size_preview_text,
                    size_preview_font,
                    theme.foreground_preview,
                );
            }
        }

        user_interface.painter().text(
            preview_position,
            Align2::RIGHT_CENTER,
            preview_value_text,
            preview_value_font,
            theme.foreground_preview,
        );

        let did_click_expand_arrow = arrow_response
            .as_ref()
            .is_some_and(|arrow_response| arrow_response.clicked());
        let did_click_row = row_response.clicked() && !did_click_expand_arrow;
        let base_hover_text = match self.symbol_tree_entry.get_kind() {
            SymbolTreeEntryKind::ModuleSpace { .. } => {
                format!("{}\n{}", self.symbol_tree_entry.get_full_path(), self.symbol_tree_entry.get_display_type_id())
            }
            _ => format!(
                "{}\n{}\n{}",
                self.symbol_tree_entry.get_full_path(),
                self.symbol_tree_entry.get_display_type_id(),
                self.symbol_tree_entry.get_locator()
            ),
        };
        let hover_text = if self.size_tooltip_text.is_empty() {
            base_hover_text
        } else {
            format!("{}\nSize: {}", base_hover_text, self.size_tooltip_text)
        };

        SymbolTreeEntryViewResponse {
            row_response: row_response.on_hover_text(hover_text),
            did_click_row,
            did_click_expand_arrow,
        }
    }

    fn measure_text_width(
        user_interface: &mut Ui,
        text: &str,
        font_id: &FontId,
        text_color: Color32,
    ) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        user_interface.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.to_string(), font_id.clone(), text_color)
                .size()
                .x
        })
    }

    fn truncate_text_to_width(
        user_interface: &mut Ui,
        text: &str,
        font_id: &FontId,
        text_color: Color32,
        max_text_width: f32,
    ) -> String {
        if text.is_empty() || max_text_width <= 0.0 {
            return String::new();
        }

        let text_width = Self::measure_text_width(user_interface, text, font_id, text_color);
        if text_width <= max_text_width {
            return text.to_string();
        }

        let ellipsis = "...";
        let ellipsis_width = Self::measure_text_width(user_interface, ellipsis, font_id, text_color);
        if ellipsis_width > max_text_width {
            return String::new();
        }

        let mut truncated_text = text.to_string();
        while !truncated_text.is_empty() {
            truncated_text.pop();
            let candidate_text = format!("{}{}", truncated_text, ellipsis);
            let candidate_width = Self::measure_text_width(user_interface, &candidate_text, font_id, text_color);

            if candidate_width <= max_text_width {
                return candidate_text;
            }
        }

        String::new()
    }
}

impl<'lifetime> Widget for SymbolTreeEntryView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        self.show(user_interface).row_response
    }
}
