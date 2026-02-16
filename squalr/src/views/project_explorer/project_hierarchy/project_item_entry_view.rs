use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{checkbox::Checkbox, state_layer::StateLayer},
    },
    views::project_explorer::project_hierarchy::view_data::project_hierarchy_frame_action::ProjectHierarchyFrameAction,
};
use eframe::egui::{Align2, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use std::{path::PathBuf, sync::Arc};

pub struct ProjectItemEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    project_item_path: &'lifetime PathBuf,
    display_name: &'lifetime str,
    preview_value: &'lifetime str,
    is_activated: bool,
    depth: usize,
    icon: Option<TextureHandle>,
    is_selected: bool,
    is_directory: bool,
    has_children: bool,
    is_expanded: bool,
    project_hierarchy_frame_action: &'lifetime mut ProjectHierarchyFrameAction,
}

impl<'lifetime> ProjectItemEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        project_item_path: &'lifetime PathBuf,
        display_name: &'lifetime str,
        preview_value: &'lifetime str,
        is_activated: bool,
        depth: usize,
        icon: Option<TextureHandle>,
        is_selected: bool,
        is_directory: bool,
        has_children: bool,
        is_expanded: bool,
        project_hierarchy_frame_action: &'lifetime mut ProjectHierarchyFrameAction,
    ) -> Self {
        Self {
            app_context,
            project_item_path,
            display_name,
            preview_value,
            is_activated,
            depth,
            icon,
            is_selected,
            is_directory,
            has_children,
            is_expanded,
            project_hierarchy_frame_action,
        }
    }
}

impl<'lifetime> Widget for ProjectItemEntryView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
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

        if let Some(icon) = &self.icon {
            IconDraw::draw_sized(user_interface, icon_rect.center(), icon_size, icon);
        }

        user_interface.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            self.display_name,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            preview_pos,
            Align2::RIGHT_CENTER,
            self.preview_value,
            theme.font_library.font_noto_sans.font_small.clone(),
            theme.foreground_preview,
        );

        response
    }
}
