use crate::ui::widgets::controls::state_layer::StateLayer;
use crate::{app_context::AppContext, ui::theme::Theme};
use eframe::egui::{Align, Area, Frame, Id, Key, Layout, Order, Response, Sense, Ui, Widget};
use epaint::{Color32, CornerRadius, Rect, TextureHandle, pos2, vec2};
use std::rc::Rc;

/// A combo box that allows arbitrary custom content (ie not a normalized dropdown entry list).
pub struct ComboBoxView<'lifetime, F: FnOnce(&mut Ui)> {
    app_context: Rc<AppContext>,
    label: &'lifetime str,
    icon: Option<TextureHandle>,
    add_contents: F,
    width: f32,
    height: f32,
    icon_padding: f32,
    icon_size: f32,
    label_spacing: f32,
    divider_width: f32,
    corner_radius: u8,
}

impl<'lifetime, F: FnOnce(&mut Ui)> ComboBoxView<'lifetime, F> {
    pub fn new_from_theme(
        theme: &Theme,
        app_context: Rc<AppContext>,
        label: &'lifetime str,
        icon: Option<TextureHandle>,
        add_contents: F,
    ) -> Self {
        Self {
            app_context,
            label,
            icon,
            add_contents,
            width: 192.0,
            height: 28.0,

            // Themed layout defaults
            icon_padding: 4.0,
            icon_size: 16.0,
            label_spacing: 8.0,
            divider_width: 1.0,
            corner_radius: 2,
        }
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.width = width;
        self
    }

    pub fn height(
        mut self,
        height: f32,
    ) -> Self {
        self.height = height;
        self
    }
}

impl<'lifetime, F: FnOnce(&mut Ui)> Widget for ComboBoxView<'lifetime, F> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let font_id = theme.font_library.font_noto_sans.font_normal.clone();
        let text_color = theme.foreground;
        let down_arrow = &theme.icon_library.icon_navigation_down_arrow_small;

        let desired_size = vec2(self.width, self.height);
        let (available_size_rectangle, response) = user_interface.allocate_exact_size(desired_size, Sense::click());

        // Precompute positions
        let icon_size_vec = vec2(self.icon_size, self.icon_size);
        let icon_y = available_size_rectangle.center().y - icon_size_vec.y * 0.5;

        // Left-side icon (new)
        let left_icon_pos = pos2(available_size_rectangle.min.x + self.icon_padding, icon_y);

        // Text label
        let galley = user_interface
            .ctx()
            .fonts(|fonts| fonts.layout_no_wrap(self.label.to_owned(), font_id.clone(), text_color));
        let text_pos = pos2(
            left_icon_pos.x + icon_size_vec.x + self.label_spacing,
            available_size_rectangle.center().y - galley.size().y * 0.5,
        );
        let border_width = 1.0;

        // Draw base background
        user_interface
            .painter()
            .rect_filled(available_size_rectangle, CornerRadius::same(self.corner_radius), theme.background_control);

        // State overlay (hover/press)
        StateLayer {
            bounds_min: available_size_rectangle.min,
            bounds_max: available_size_rectangle.max,
            enabled: true,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::same(self.corner_radius),
            border_width,
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
                Color32::WHITE,
            );
        }
        // Draw text next to icon
        user_interface.painter().galley(text_pos, galley, text_color);

        // Divider bar before right arrow
        let divider_x = available_size_rectangle.max.x - (self.icon_size + self.icon_padding * 2.0 + self.divider_width);
        let divider_rect = Rect::from_min_max(
            pos2(divider_x, available_size_rectangle.min.y + border_width),
            pos2(divider_x + self.divider_width, available_size_rectangle.max.y),
        );
        user_interface
            .painter()
            .rect_filled(divider_rect, 0.0, theme.submenu_border);

        // Draw right arrow
        let right_arrow_pos = pos2(available_size_rectangle.max.x - self.icon_size - self.icon_padding, icon_y);
        user_interface.painter().image(
            down_arrow.id(),
            Rect::from_min_size(right_arrow_pos, icon_size_vec),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        // Popup logic
        let popup_id = Id::new(("combo_popup", user_interface.id().value(), self.label));
        let mut open = user_interface.memory(|memory| memory.data.get_temp::<bool>(popup_id).unwrap_or(false));

        if response.clicked() {
            open = !open;
        }
        if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
            open = false;
        }
        user_interface.memory_mut(|memory| memory.data.insert_temp(popup_id, open));

        if !open {
            return response;
        }

        // Draw popup content
        let popup_pos = pos2(available_size_rectangle.min.x, available_size_rectangle.max.y + 2.0);
        let popup_id_area = Id::new(("combo_popup_area", user_interface.id().value(), self.label));
        let mut popup_rectangle: Option<Rect> = None;

        Area::new(popup_id_area)
            .order(Order::Foreground)
            .fixed_pos(popup_pos)
            .show(user_interface.ctx(), |popup_ui| {
                Frame::popup(user_interface.style())
                    .fill(theme.background_primary)
                    .show(popup_ui, |popup_ui| {
                        popup_ui.set_min_width(self.width);
                        popup_ui.with_layout(Layout::top_down(Align::Min), |inner_ui| {
                            (self.add_contents)(inner_ui);
                        });
                        popup_rectangle = Some(popup_ui.min_rect());
                    });
            });

        // Close popup when clicking outside.
        if user_interface.input(|input_state| {
            if !input_state.pointer.any_click() {
                return false;
            }

            let pos = input_state
                .pointer
                .interact_pos()
                .unwrap_or(available_size_rectangle.center());
            let outside_header = !available_size_rectangle.contains(pos);
            let outside_popup = popup_rectangle.map_or(true, |popup_rectangle| !popup_rectangle.contains(pos));

            outside_header && outside_popup
        }) {
            user_interface.memory_mut(|memory| memory.data.insert_temp(popup_id, false));
        }

        response
    }
}
