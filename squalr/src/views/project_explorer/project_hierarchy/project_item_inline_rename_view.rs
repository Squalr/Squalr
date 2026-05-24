use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{checkbox::Checkbox, state_layer::StateLayer},
    },
};
use eframe::egui::{
    Id, Key, Rect, Response, Sense, TextEdit, TextureHandle, Ui, UiBuilder, Widget, pos2,
    text::{CCursor, CCursorRange},
    vec2,
};
use epaint::{CornerRadius, Stroke, StrokeKind};
use std::{path::PathBuf, sync::Arc};

pub struct ProjectItemInlineRenameView<'lifetime> {
    app_context: Arc<AppContext>,
    project_item_path: &'lifetime PathBuf,
    rename_text: &'lifetime mut String,
    should_highlight_text: &'lifetime mut bool,
    is_activated: bool,
    depth: usize,
    icon: Option<TextureHandle>,
    is_selected: bool,
    is_directory: bool,
    has_children: bool,
    is_expanded: bool,
}

pub struct ProjectItemInlineRenameViewResponse {
    pub row_response: Response,
    pub should_commit: bool,
    pub should_cancel: bool,
}

impl<'lifetime> ProjectItemInlineRenameView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_item_path: &'lifetime PathBuf,
        rename_text: &'lifetime mut String,
        should_highlight_text: &'lifetime mut bool,
        is_activated: bool,
        depth: usize,
        icon: Option<TextureHandle>,
        is_selected: bool,
        is_directory: bool,
        has_children: bool,
        is_expanded: bool,
    ) -> Self {
        Self {
            app_context,
            project_item_path,
            rename_text,
            should_highlight_text,
            is_activated,
            depth,
            icon,
            is_selected,
            is_directory,
            has_children,
            is_expanded,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> ProjectItemInlineRenameViewResponse {
        let theme = &self.app_context.theme;
        let icon_size = vec2(16.0, 16.0);
        let expand_arrow_size = vec2(10.0, 10.0);
        let row_left_padding = 8.0;
        let tree_level_indent = 18.0;
        let text_left_padding = 4.0;
        let checkbox_size = vec2(18.0, 18.0);
        let right_padding = 8.0;
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

        let checkbox_pos_x =
            allocated_size_rectangle.min.x + row_left_padding + self.depth as f32 * tree_level_indent + expand_arrow_size.x + text_left_padding;
        let checkbox_rect = Rect::from_min_size(pos2(checkbox_pos_x, allocated_size_rectangle.center().y - checkbox_size.y * 0.5), checkbox_size);
        let mut checkbox_user_interface = user_interface.new_child(UiBuilder::new().max_rect(checkbox_rect));
        checkbox_user_interface.add_enabled(false, Checkbox::new_from_theme(theme).with_check_state_bool(self.is_activated));

        let indentation = self.depth as f32 * tree_level_indent;
        let arrow_center = pos2(
            allocated_size_rectangle.min.x + row_left_padding + indentation + expand_arrow_size.x * 0.5,
            allocated_size_rectangle.center().y,
        );

        if self.is_directory && self.has_children {
            let expand_icon = if self.is_expanded {
                &theme.icon_library.icon_handle_navigation_down_arrow_small
            } else {
                &theme.icon_library.icon_handle_navigation_right_arrow_small
            };

            IconDraw::draw_sized(user_interface, arrow_center, expand_arrow_size, expand_icon);
        }

        let icon_pos_x = checkbox_rect.max.x + text_left_padding;
        let icon_pos_y = allocated_size_rectangle.center().y - icon_size.y * 0.5;
        let icon_rect = Rect::from_min_size(pos2(icon_pos_x, icon_pos_y), icon_size);

        if let Some(icon) = &self.icon {
            IconDraw::draw_sized(user_interface, icon_rect.center(), icon_size, icon);
        }

        let text_rect = Rect::from_min_max(
            pos2(icon_rect.max.x + text_left_padding, allocated_size_rectangle.min.y + 3.0),
            pos2(allocated_size_rectangle.max.x - right_padding, allocated_size_rectangle.max.y - 3.0),
        );
        let text_edit_id = Id::new(("project_hierarchy_inline_rename_editor", self.project_item_path));
        let mut text_edit_user_interface = user_interface.new_child(UiBuilder::new().max_rect(text_rect));
        text_edit_user_interface.set_clip_rect(text_rect);
        let mut output = TextEdit::singleline(self.rename_text)
            .id_salt(text_edit_id)
            .font(theme.font_library.font_ubuntu_mono_bold.font_normal.clone())
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

        ProjectItemInlineRenameViewResponse {
            row_response,
            should_commit: text_edit_response.lost_focus() && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)),
            should_cancel: text_edit_response.has_focus() && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)),
        }
    }
}
