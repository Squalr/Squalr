use crate::app_context::AppContext;
use crate::ui::widgets::controls::state_layer::StateLayer;
use eframe::egui::{Align, Area, Frame, Id, Key, Layout, Order, Response, Sense, Ui, Widget};
use epaint::{Color32, CornerRadius, Margin, Rect, Stroke, TextureHandle, Vec2, pos2, vec2};
use std::{borrow::Cow, sync::Arc};

/// A combo box that allows arbitrary custom content (ie not a normalized dropdown entry list).
pub struct ComboBoxView<'lifetime, F: FnOnce(&mut Ui, &mut bool)> {
    app_context: Arc<AppContext>,
    label: Cow<'lifetime, str>,
    menu_id: &'lifetime str,
    icon: Option<TextureHandle>,
    add_contents: F,
    disabled: bool,
    width: f32,
    height: f32,
    icon_padding_left: f32,
    icon_size: f32,
    label_spacing: f32,
    show_dropdown_arrow: bool,
    divider_width: f32,
    border_width: f32,
    corner_radius: u8,
}

impl<'lifetime, F: FnOnce(&mut Ui, &mut bool)> ComboBoxView<'lifetime, F> {
    pub fn new(
        app_context: Arc<AppContext>,
        label: impl Into<Cow<'lifetime, str>>,
        menu_id: &'lifetime str,
        icon: Option<TextureHandle>,
        add_contents: F,
    ) -> Self {
        Self {
            app_context,
            label: label.into(),
            menu_id,
            icon,
            add_contents,
            disabled: false,
            width: 192.0,
            height: 28.0,
            icon_padding_left: 8.0,
            icon_size: 16.0,
            label_spacing: 8.0,
            show_dropdown_arrow: true,
            divider_width: 1.0,
            border_width: 1.0,
            corner_radius: 0,
        }
    }

    pub fn close(
        &self,
        user_interface: &mut Ui,
    ) {
        let popup_id = Id::new(("combo_popup", self.menu_id, user_interface.id().value()));

        user_interface.memory_mut(|memory| {
            memory.data.insert_temp(popup_id, false);
        });
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.width = width;
        self
    }

    pub fn disabled(
        mut self,
        disabled: bool,
    ) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn height(
        mut self,
        height: f32,
    ) -> Self {
        self.height = height;
        self
    }

    pub fn show_dropdown_arrow(
        mut self,
        show_dropdown_arrow: bool,
    ) -> Self {
        self.show_dropdown_arrow = show_dropdown_arrow;
        self
    }
}

impl<'lifetime, F: FnOnce(&mut Ui, &mut bool)> Widget for ComboBoxView<'lifetime, F> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let font_id = theme.font_library.font_noto_sans.font_normal.clone();
        let text_color = if self.disabled { theme.foreground_preview } else { theme.foreground };
        let icon_tint = if self.disabled { theme.foreground_preview } else { Color32::WHITE };
        let down_arrow = &theme.icon_library.icon_handle_navigation_down_arrow_small;
        let desired_size = vec2(self.width, self.height);
        let sense = if self.disabled { Sense::hover() } else { Sense::click() };
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(desired_size, sense);

        // Precompute positions.
        let icon_size_vec = vec2(self.icon_size, self.icon_size);
        let icon_y = allocated_size_rectangle.center().y - icon_size_vec.y * 0.5;
        let right_side_width = if self.show_dropdown_arrow {
            self.icon_size + self.icon_padding_left * 2.0 + self.divider_width
        } else {
            0.0
        };

        // Left-side icon.
        let has_label = !self.label.is_empty();
        let should_center_icon_only = self.icon.is_some() && !has_label && !self.show_dropdown_arrow;
        let left_icon_pos = if should_center_icon_only {
            pos2(allocated_size_rectangle.center().x - icon_size_vec.x * 0.5, icon_y)
        } else {
            pos2(allocated_size_rectangle.min.x + self.icon_padding_left, icon_y)
        };

        // Text label.
        let galley = user_interface
            .ctx()
            .fonts_mut(|fonts| fonts.layout_no_wrap(self.label.to_string(), font_id.clone(), text_color));
        let base_x = allocated_size_rectangle.min.x + self.icon_padding_left;
        let text_pos = pos2(
            if self.icon.is_some() {
                base_x + icon_size_vec.x + self.label_spacing
            } else {
                base_x
            },
            allocated_size_rectangle.center().y - galley.size().y * 0.5,
        );
        let divider_x = allocated_size_rectangle.max.x - right_side_width;
        let label_clip_rectangle = Rect::from_min_max(
            pos2(text_pos.x, allocated_size_rectangle.min.y + self.border_width),
            pos2((divider_x - self.label_spacing).max(text_pos.x), allocated_size_rectangle.max.y),
        );

        // Draw base background.
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::same(self.corner_radius), theme.background_control);

        // State overlay (hover/press).
        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
            enabled: !self.disabled,
            pressed: !self.disabled && response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::same(self.corner_radius),
            border_width: self.border_width,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.submenu_border,
            border_color_focused: theme.focused_border,
        }
        .ui(user_interface);

        // Draw left icon.
        if let Some(icon) = &self.icon {
            user_interface.painter().image(
                icon.id(),
                Rect::from_min_size(left_icon_pos, icon_size_vec),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                icon_tint,
            );
        }
        // Draw text next to icon.
        if has_label {
            user_interface
                .painter()
                .with_clip_rect(label_clip_rectangle)
                .galley(text_pos, galley, text_color);
        }

        if self.show_dropdown_arrow {
            // Divider bar before right arrow.
            let divider_rectangle = Rect::from_min_max(
                pos2(divider_x, allocated_size_rectangle.min.y + self.border_width),
                pos2(divider_x + self.divider_width, allocated_size_rectangle.max.y),
            );

            user_interface
                .painter()
                .rect_filled(divider_rectangle, 0.0, theme.submenu_border);

            // Draw right arrow.
            let right_arrow_pos = pos2(allocated_size_rectangle.max.x - self.icon_size - self.icon_padding_left, icon_y);

            user_interface.painter().image(
                down_arrow.id(),
                Rect::from_min_size(right_arrow_pos, icon_size_vec),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                icon_tint,
            );
        }

        // Popup logic.
        let popup_id = Id::new(("combo_popup", self.menu_id, user_interface.id().value()));
        let mut open = user_interface.memory(|memory| memory.data.get_temp::<bool>(popup_id).unwrap_or(false));

        if response.clicked() && !self.disabled {
            open = !open;
        }

        if self.disabled {
            open = false;
        }

        if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
            open = false;
        }

        user_interface.memory_mut(|memory| memory.data.insert_temp(popup_id, open));

        if !open {
            return response;
        }

        // Draw popup content.
        let popup_pos = pos2(allocated_size_rectangle.min.x, allocated_size_rectangle.max.y + 2.0);
        let popup_id_area = Id::new(("combo_popup_area", self.menu_id, user_interface.id().value()));
        let mut should_close = false;

        let area_response = Area::new(popup_id_area)
            .order(Order::Foreground)
            .fixed_pos(popup_pos)
            .show(user_interface.ctx(), |popup_user_interface| {
                Frame::new()
                    .fill(theme.background_primary)
                    .stroke(Stroke::new(self.border_width, theme.submenu_border))
                    .inner_margin(0)
                    .outer_margin(0)
                    .corner_radius(self.corner_radius)
                    .show(popup_user_interface, |popup_user_interface| {
                        popup_user_interface.spacing_mut().menu_margin = Margin::ZERO;
                        popup_user_interface.spacing_mut().window_margin = Margin::ZERO;
                        popup_user_interface.spacing_mut().menu_spacing = 0.0;
                        popup_user_interface.spacing_mut().item_spacing = Vec2::ZERO;
                        popup_user_interface.set_min_width(self.width);
                        popup_user_interface.set_max_width(self.width);
                        popup_user_interface.with_layout(Layout::top_down(Align::Min), |inner_user_interface| {
                            (self.add_contents)(inner_user_interface, &mut should_close);
                        });
                    });
            });

        let popup_rectangle = area_response.response.rect;

        let clicked_outside = user_interface.input(|input_state| {
            if !input_state.pointer.any_click() {
                return false;
            }

            let click_position = input_state
                .pointer
                .interact_pos()
                .unwrap_or(allocated_size_rectangle.center());
            let outside_header = !allocated_size_rectangle.contains(click_position);
            let outside_popup = !popup_rectangle.contains(click_position);

            outside_header && outside_popup
        });

        // Close popup when clicking outside.
        if should_close || clicked_outside {
            user_interface.memory_mut(|memory| memory.data.insert_temp(popup_id, false));
        }

        response
    }
}
