use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        geometry::safe_clamp_f32,
        widgets::controls::{checkbox::Checkbox, state_layer::StateLayer},
    },
    views::project_explorer::project_hierarchy::view_data::project_hierarchy_frame_action::ProjectHierarchyFrameAction,
};
use eframe::egui::{Align2, Color32, FontId, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use std::{path::PathBuf, sync::Arc};

pub struct ProjectItemEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    project_item_path: &'lifetime PathBuf,
    display_name: &'lifetime str,
    preview_path: &'lifetime str,
    preview_value: &'lifetime str,
    is_activated: bool,
    depth: usize,
    icon: Option<TextureHandle>,
    is_selected: bool,
    is_cut: bool,
    is_directory: bool,
    has_children: bool,
    is_expanded: bool,
    project_hierarchy_frame_action: &'lifetime mut ProjectHierarchyFrameAction,
}

pub struct ProjectItemEntryViewResponse {
    pub row_response: Response,
    pub should_request_rename: bool,
    pub should_request_value_edit: bool,
}

impl<'lifetime> ProjectItemEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_item_path: &'lifetime PathBuf,
        display_name: &'lifetime str,
        preview_path: &'lifetime str,
        preview_value: &'lifetime str,
        is_activated: bool,
        depth: usize,
        icon: Option<TextureHandle>,
        is_selected: bool,
        is_cut: bool,
        is_directory: bool,
        has_children: bool,
        is_expanded: bool,
        project_hierarchy_frame_action: &'lifetime mut ProjectHierarchyFrameAction,
    ) -> Self {
        Self {
            app_context,
            project_item_path,
            display_name,
            preview_path,
            preview_value,
            is_activated,
            depth,
            icon,
            is_selected,
            is_cut,
            is_directory,
            has_children,
            is_expanded,
            project_hierarchy_frame_action,
        }
    }
}

impl<'lifetime> ProjectItemEntryView<'lifetime> {
    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> ProjectItemEntryViewResponse {
        let theme = &self.app_context.theme;
        let row_foreground = if self.is_cut { theme.foreground_preview } else { theme.foreground };
        let row_foreground_preview = if self.is_cut {
            theme.foreground_preview.gamma_multiply(0.85)
        } else {
            theme.foreground_preview
        };
        let row_icon_tint = if self.is_cut { theme.foreground_preview } else { Color32::WHITE };
        let icon_size = vec2(16.0, 16.0);
        let expand_arrow_size = vec2(10.0, 10.0);
        let row_left_padding = 8.0;
        let tree_level_indent = 18.0;
        let text_left_padding = 4.0;
        let checkbox_size = vec2(18.0, 18.0);
        let right_preview_padding = 8.0;
        let row_height = 28.0;
        let (allocated_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_size().x, row_height), Sense::click_and_drag());

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
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        let checkbox_pos_x =
            allocated_size_rectangle.min.x + row_left_padding + self.depth as f32 * tree_level_indent + expand_arrow_size.x + text_left_padding;
        let checkbox_rect = Rect::from_min_size(pos2(checkbox_pos_x, allocated_size_rectangle.center().y - checkbox_size.y * 0.5), checkbox_size);
        let is_activated = self.is_activated;
        let checkbox_response = user_interface.place(checkbox_rect, Checkbox::new_from_theme(theme).with_check_state_bool(is_activated));

        if checkbox_response.clicked() {
            *self.project_hierarchy_frame_action = ProjectHierarchyFrameAction::SetProjectItemActivation(self.project_item_path.clone(), !is_activated);
        }

        let indentation = self.depth as f32 * tree_level_indent;
        let arrow_center = pos2(
            allocated_size_rectangle.min.x + row_left_padding + indentation + expand_arrow_size.x * 0.5,
            allocated_size_rectangle.center().y,
        );
        let arrow_hit_box_size = vec2(14.0, 14.0);
        let arrow_hit_box_rect = Rect::from_center_size(arrow_center, arrow_hit_box_size);
        let arrow_click_response = user_interface.interact(
            arrow_hit_box_rect,
            user_interface.make_persistent_id(("project_hierarchy_arrow", self.project_item_path)),
            Sense::click(),
        );

        if self.is_directory && self.has_children {
            let expand_icon = if self.is_expanded {
                &theme.icon_library.icon_handle_navigation_down_arrow_small
            } else {
                &theme.icon_library.icon_handle_navigation_right_arrow_small
            };

            IconDraw::draw_sized(user_interface, arrow_center, expand_arrow_size, expand_icon);
            if arrow_click_response.clicked() {
                *self.project_hierarchy_frame_action = ProjectHierarchyFrameAction::ToggleDirectoryExpansion(self.project_item_path.clone());
            }
        }

        if response.clicked() && !checkbox_response.clicked() && !arrow_click_response.clicked() {
            let input_modifiers = user_interface.input(|input_state| input_state.modifiers);
            let additive_selection = input_modifiers.command || input_modifiers.ctrl;
            let range_selection = input_modifiers.shift;

            *self.project_hierarchy_frame_action = ProjectHierarchyFrameAction::SelectProjectItem {
                project_item_path: self.project_item_path.clone(),
                additive_selection,
                range_selection,
            };
        }

        let icon_pos_x = checkbox_rect.max.x + text_left_padding;
        let icon_pos_y = allocated_size_rectangle.center().y - icon_size.y * 0.5;
        let icon_rect = Rect::from_min_size(pos2(icon_pos_x, icon_pos_y), icon_size);
        let text_pos = pos2(icon_rect.max.x + text_left_padding, allocated_size_rectangle.center().y);
        let preview_pos = pos2(allocated_size_rectangle.max.x - right_preview_padding, allocated_size_rectangle.center().y);
        let display_name_font = theme.font_library.font_noto_sans.font_normal.clone();
        let preview_path_font = theme.font_library.font_noto_sans.font_small.clone();
        let preview_value_font = theme.font_library.font_noto_sans.font_small.clone();
        let preview_value_width = Self::measure_text_width(user_interface, self.preview_value, &preview_value_font, row_foreground_preview);
        let left_text_max_x = preview_pos.x - preview_value_width - 12.0;
        let max_left_text_width = (left_text_max_x - text_pos.x).max(0.0);
        let value_hit_box_left = safe_clamp_f32(left_text_max_x, text_pos.x, allocated_size_rectangle.max.x);
        let name_hit_box_rect = Rect::from_min_max(
            pos2(text_pos.x, allocated_size_rectangle.min.y),
            pos2(value_hit_box_left, allocated_size_rectangle.max.y),
        );
        let value_hit_box_rect = Rect::from_min_max(pos2(value_hit_box_left, allocated_size_rectangle.min.y), allocated_size_rectangle.max);

        if let Some(icon) = &self.icon {
            IconDraw::draw_sized_tinted(user_interface, icon_rect.center(), icon_size, icon, row_icon_tint);
        }

        let full_display_name_width = Self::measure_text_width(user_interface, self.display_name, &display_name_font, row_foreground);
        let display_name_text = if self.preview_path.is_empty() || full_display_name_width >= max_left_text_width {
            Self::truncate_text_to_width(user_interface, self.display_name, &display_name_font, row_foreground, max_left_text_width)
        } else {
            self.display_name.to_string()
        };
        let display_name_width = Self::measure_text_width(user_interface, &display_name_text, &display_name_font, row_foreground);

        user_interface
            .painter()
            .text(text_pos, Align2::LEFT_CENTER, display_name_text, display_name_font.clone(), row_foreground);

        if !self.preview_path.is_empty() && display_name_width < max_left_text_width {
            let preview_path_gap = 10.0;
            let preview_path_pos = pos2(text_pos.x + display_name_width + preview_path_gap, allocated_size_rectangle.center().y);
            let max_preview_path_width = (max_left_text_width - display_name_width - preview_path_gap).max(0.0);
            let preview_path_text = Self::truncate_text_to_width(
                user_interface,
                self.preview_path,
                &preview_path_font,
                row_foreground_preview,
                max_preview_path_width,
            );

            if !preview_path_text.is_empty() {
                user_interface.painter().text(
                    preview_path_pos,
                    Align2::LEFT_CENTER,
                    preview_path_text,
                    preview_path_font,
                    row_foreground_preview,
                );
            }
        }

        user_interface.painter().text(
            preview_pos,
            Align2::RIGHT_CENTER,
            self.preview_value,
            preview_value_font,
            row_foreground_preview,
        );

        let click_position = response
            .interact_pointer_pos()
            .unwrap_or_else(|| allocated_size_rectangle.center());
        let should_request_value_edit = response.double_clicked()
            && !checkbox_response.clicked()
            && !arrow_click_response.clicked()
            && !self.is_directory
            && !self.preview_value.is_empty()
            && value_hit_box_rect.contains(click_position);
        let should_request_rename = response.double_clicked()
            && !checkbox_response.clicked()
            && !arrow_click_response.clicked()
            && name_hit_box_rect.contains(click_position)
            && !should_request_value_edit;
        let row_response = if self.preview_path.is_empty() {
            response
        } else {
            response.on_hover_text(format!("{}: {}", self.display_name, self.preview_path))
        };

        ProjectItemEntryViewResponse {
            row_response,
            should_request_rename,
            should_request_value_edit,
        }
    }
}

impl<'lifetime> ProjectItemEntryView<'lifetime> {
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
