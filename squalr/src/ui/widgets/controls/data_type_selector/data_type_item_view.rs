use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{check_state::CheckState, checkbox::Checkbox, state_layer::StateLayer},
    },
};
use eframe::egui::{Align2, Rect, Response, Sense, TextureHandle, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::CornerRadius;
use std::sync::Arc;

/// A generic context menu item.
pub struct DataTypeItemView<'lifetime> {
    app_context: Arc<AppContext>,
    label: &'lifetime str,
    icon: Option<TextureHandle>,
    combo_box_width: f32,
    check_state: Option<CheckState>,
    disabled: bool,
}

impl<'lifetime> DataTypeItemView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        label: &'lifetime str,
        icon: Option<TextureHandle>,
        width: f32,
    ) -> Self {
        Self {
            app_context: app_context,
            label,
            icon,
            combo_box_width: width,
            check_state: None,
            disabled: false,
        }
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.combo_box_width = width;
        self
    }

    pub fn with_check_state(
        mut self,
        check_state: CheckState,
    ) -> Self {
        self.check_state = Some(check_state);
        self
    }

    pub fn disabled(
        mut self,
        disabled: bool,
    ) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<'a> Widget for DataTypeItemView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let icon_size = vec2(16.0, 16.0);
        let left_padding = 8.0;
        let item_spacing = 8.0;
        let is_selected = matches!(self.check_state, Some(CheckState::True | CheckState::Mixed));

        // Whole clickable area includes indentation.
        let row_height = 36.0;
        let sense = if self.disabled { Sense::hover() } else { Sense::click_and_drag() };
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(self.combo_box_width, row_height), sense);

        if is_selected {
            user_interface
                .painter()
                .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.selected_background);
        }

        // Background and state overlay.
        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
            enabled: !self.disabled,
            pressed: !self.disabled && response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: if is_selected { 1.0 } else { 0.0 },
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: match is_selected {
                true => theme.selected_border,
                false => theme.background_control_secondary,
            },
            border_color_focused: match is_selected {
                true => theme.selected_border,
                false => theme.background_control_secondary,
            },
        }
        .ui(user_interface);

        let mut item_contents_user_interface = user_interface.new_child(UiBuilder::new().max_rect(allocated_size_rectangle));
        item_contents_user_interface.set_clip_rect(allocated_size_rectangle);

        // Draw icon and label inside layout.
        let mut current_pos_x = allocated_size_rectangle.min.x + left_padding;

        if let Some(check_state) = self.check_state {
            let checkbox_rect = Rect::from_min_size(
                pos2(current_pos_x, allocated_size_rectangle.center().y - Checkbox::HEIGHT * 0.5),
                vec2(Checkbox::WIDTH, Checkbox::HEIGHT),
            );
            item_contents_user_interface.put(
                checkbox_rect,
                Checkbox::new_from_theme(theme)
                    .with_check_state(check_state)
                    .disabled(true),
            );
            current_pos_x = checkbox_rect.max.x + item_spacing;
        }

        let text_pos = if let Some(icon) = &self.icon {
            let icon_rect = Rect::from_min_size(pos2(current_pos_x, allocated_size_rectangle.center().y - icon_size.y * 0.5), icon_size);
            IconDraw::draw_sized(&item_contents_user_interface, icon_rect.center(), icon_size, icon);
            current_pos_x = icon_rect.max.x + item_spacing;

            pos2(current_pos_x, allocated_size_rectangle.center().y)
        } else {
            pos2(current_pos_x, allocated_size_rectangle.center().y)
        };

        item_contents_user_interface.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            self.label,
            theme.font_library.font_noto_sans.font_normal.clone(),
            match self.disabled {
                true => theme.foreground_preview,
                false => theme.foreground,
            },
        );

        response
    }
}
