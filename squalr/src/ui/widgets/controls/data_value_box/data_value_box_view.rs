use crate::ui::widgets::controls::state_layer::StateLayer;
use crate::{app_context::AppContext, ui::widgets::controls::data_value_box::data_value_box_convert_item_view::DataValueBoxConvertItemView};
use eframe::egui::{Align, Area, Frame, Id, Key, Layout, Order, Response, Sense, TextEdit, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, Margin, Rect, Stroke, StrokeKind, Vec2, pos2, vec2};
use squalr_engine_api::{
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{anonymous_value::AnonymousValue, display_value::DisplayValue, display_value_type::DisplayValueType},
    },
};
use std::sync::Arc;

pub struct DataValueBoxView<'lifetime> {
    app_context: Arc<AppContext>,
    display_value: &'lifetime mut DisplayValue,
    validation_data_type: &'lifetime DataTypeRef,
    preview_text: &'lifetime str,
    width: f32,
    height: f32,
    icon_padding: f32,
    icon_size: f32,
    divider_width: f32,
    corner_radius: u8,
}

impl<'lifetime> DataValueBoxView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        display_value: &'lifetime mut DisplayValue,
        validation_data_type: &'lifetime DataTypeRef,
        preview_text: &'lifetime str,
    ) -> Self {
        Self {
            app_context,
            display_value,
            validation_data_type,
            preview_text,
            width: 192.0,
            height: 28.0,

            // Themed layout defaults
            icon_padding: 8.0,
            icon_size: 16.0,
            divider_width: 1.0,
            corner_radius: 0,
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

impl<'lifetime> Widget for DataValueBoxView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let font_id = theme.font_library.font_noto_sans.font_normal.clone();
        let down_arrow = &theme.icon_library.icon_handle_navigation_down_arrow_small;
        let anonymous_value = AnonymousValue::new(&self.display_value);
        let DATA_TYPE_REGISTRY = SymbolRegistry::new();
        let is_valid = DATA_TYPE_REGISTRY.validate_value(&self.validation_data_type, &anonymous_value);
        let text_color = match is_valid {
            true => match self.display_value.get_display_value_type() {
                DisplayValueType::Bool => theme.foreground,
                DisplayValueType::String => theme.foreground,
                DisplayValueType::Binary => theme.binary_blue,
                DisplayValueType::Decimal => theme.foreground,
                DisplayValueType::Hexadecimal => theme.hexadecimal_green,
                DisplayValueType::Address => theme.hexadecimal_green,
                DisplayValueType::DataTypeRef => theme.foreground,
                DisplayValueType::Enumeration => theme.foreground,
            },
            false => theme.error_red,
        };

        let desired_size = vec2(self.width, self.height);
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(desired_size, Sense::click());
        let icon_size_vec = vec2(self.icon_size, self.icon_size);
        let border_width = 1.0;

        // Divider bar before right arrow.
        let button_width = self.icon_size + self.icon_padding * 2.0;
        let divider_x = allocated_size_rectangle.max.x - (button_width + self.divider_width);

        // The button rectangle (full clickable dropdown-area background).
        let dropdown_background_rectangle = Rect::from_min_max(
            pos2(divider_x + self.divider_width, allocated_size_rectangle.min.y),
            pos2(allocated_size_rectangle.max.x, allocated_size_rectangle.max.y),
        );

        // Arrow position.
        let right_arrow_pos = pos2(
            allocated_size_rectangle.max.x - self.icon_padding - self.icon_size,
            allocated_size_rectangle.center().y - self.icon_size * 0.5,
        );

        user_interface
            .painter()
            .rect_filled(dropdown_background_rectangle, CornerRadius::same(self.corner_radius), theme.background_control);

        // State overlay (hover/press).
        StateLayer {
            bounds_min: dropdown_background_rectangle.min,
            bounds_max: dropdown_background_rectangle.max,
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

        // Define editable region (between left label and dropdown divider).
        let text_edit_rectangle = Rect::from_min_max(
            pos2(allocated_size_rectangle.min.x, allocated_size_rectangle.min.y),
            pos2(divider_x, allocated_size_rectangle.max.y),
        );

        let text_edit_rectangle_inner = text_edit_rectangle.shrink2(vec2(4.0, 4.0));

        let mut text_value = self.display_value.get_display_string().to_string();
        let mut text_edit_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(text_edit_rectangle_inner)
                .layout(Layout::right_to_left(Align::Center)),
        );

        let text_edit_response = text_edit_user_interface.add(
            TextEdit::singleline(&mut text_value)
                .vertical_align(eframe::egui::Align::Center)
                .font(font_id.clone())
                .text_color(text_color)
                .hint_text(self.preview_text)
                .frame(false),
        );

        user_interface.painter().rect_stroke(
            allocated_size_rectangle,
            CornerRadius::same(self.corner_radius),
            Stroke::new(border_width, theme.submenu_border),
            StrokeKind::Inside,
        );
        // Draw right arrow.
        user_interface.painter().image(
            down_arrow.id(),
            Rect::from_min_size(right_arrow_pos, icon_size_vec),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        // If the user changed text, update the display value
        if text_edit_response.changed() {
            self.display_value.set_display_string(text_value);
        }

        // Popup logic.
        let popup_id = Id::new(("data_value_box_popup", user_interface.id().value()));
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

        // Draw popup content.
        let popup_pos = pos2(allocated_size_rectangle.min.x, allocated_size_rectangle.max.y + 2.0);
        let popup_id_area = Id::new(("data_value_box_popup_area", user_interface.id().value()));
        let mut popup_rectangle: Option<Rect> = None;
        let mut should_close = false;

        Area::new(popup_id_area)
            .order(Order::Foreground)
            .fixed_pos(popup_pos)
            .show(user_interface.ctx(), |popup_user_interface| {
                Frame::popup(user_interface.style())
                    .fill(theme.background_primary)
                    .inner_margin(Margin::ZERO)
                    .corner_radius(self.corner_radius)
                    .show(popup_user_interface, |popup_user_interface| {
                        popup_user_interface.spacing_mut().menu_margin = Margin::ZERO;
                        popup_user_interface.spacing_mut().window_margin = Margin::ZERO;
                        popup_user_interface.spacing_mut().menu_spacing = 0.0;
                        popup_user_interface.spacing_mut().item_spacing = Vec2::ZERO;
                        popup_user_interface.set_min_width(self.width);
                        popup_user_interface.with_layout(Layout::top_down(Align::Min), |inner_user_interface| {
                            let display_value_types = DATA_TYPE_REGISTRY.get_supported_display_value_types(&self.validation_data_type);

                            for display_value_type in &display_value_types {
                                if inner_user_interface
                                    .add(DataValueBoxConvertItemView::new(
                                        self.app_context.clone(),
                                        self.display_value,
                                        display_value_type,
                                        true,
                                        self.width,
                                    ))
                                    .clicked()
                                {
                                    should_close = true;
                                }
                            }

                            for display_value_type in &display_value_types {
                                if inner_user_interface
                                    .add(DataValueBoxConvertItemView::new(
                                        self.app_context.clone(),
                                        self.display_value,
                                        display_value_type,
                                        false,
                                        self.width,
                                    ))
                                    .clicked()
                                {
                                    should_close = true;
                                }
                            }
                        });
                        popup_rectangle = Some(popup_user_interface.min_rect());
                    });
            });

        let clicked_outside = user_interface.input(|input_state| {
            if !input_state.pointer.any_click() {
                return false;
            }

            let click_position = input_state
                .pointer
                .interact_pos()
                .unwrap_or(allocated_size_rectangle.center());
            let outside_header = !allocated_size_rectangle.contains(click_position);
            let outside_popup = popup_rectangle.map_or(true, |popup_rectangle| !popup_rectangle.contains(click_position));

            outside_header && outside_popup
        });

        // Close popup when clicking outside.
        if should_close || clicked_outside {
            user_interface.memory_mut(|memory| memory.data.insert_temp(popup_id, false));
        }

        response
    }
}
