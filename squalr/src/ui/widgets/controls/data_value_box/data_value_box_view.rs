use crate::ui::widgets::controls::state_layer::StateLayer;
use crate::{app_context::AppContext, ui::widgets::controls::data_value_box::data_value_box_convert_item_view::DataValueBoxConvertItemView};
use eframe::egui::{Align, Area, Frame, Id, Key, Layout, Order, Response, Sense, TextEdit, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, Margin, Rect, Stroke, StrokeKind, Vec2, pos2, vec2};
use squalr_engine_api::{
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat},
    },
};
use std::sync::Arc;

pub struct DataValueBoxView<'lifetime> {
    app_context: Arc<AppContext>,
    anonymous_value_string: &'lifetime mut AnonymousValueString,
    validation_data_type: &'lifetime DataTypeRef,
    display_values: Option<&'lifetime [AnonymousValueString]>,
    is_read_only: bool,
    is_value_owned: bool,
    preview_text: &'lifetime str,
    id: &'lifetime str,
    allow_read_only_interpretation: bool,
    use_preview_foreground: bool,
    width: f32,
    height: f32,
    icon_padding: f32,
    icon_size: f32,
    border_width: f32,
    divider_width: f32,
    corner_radius: u8,
}

impl<'lifetime> DataValueBoxView<'lifetime> {
    const MIN_POPUP_WIDTH: f32 = 212.0;
    const COMMIT_ON_ENTER_ID_SALT: &'static str = "data_value_box_commit_on_enter";

    pub fn new(
        app_context: Arc<AppContext>,
        anonymous_value_string: &'lifetime mut AnonymousValueString,
        validation_data_type: &'lifetime DataTypeRef,
        is_read_only: bool,
        is_value_owned: bool,
        preview_text: &'lifetime str,
        id: &'lifetime str,
    ) -> Self {
        Self {
            app_context,
            anonymous_value_string,
            validation_data_type,
            display_values: None,
            is_read_only,
            is_value_owned,
            preview_text,
            id,
            allow_read_only_interpretation: false,
            use_preview_foreground: false,
            width: 212.0,
            height: 28.0,

            // Themed layout defaults
            icon_padding: 8.0,
            icon_size: 16.0,
            border_width: 1.0,
            divider_width: 1.0,
            corner_radius: 0,
        }
    }

    pub fn border_width(
        mut self,
        border_width: f32,
    ) -> Self {
        self.border_width = border_width;
        self
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.width = width;
        self
    }

    pub fn allow_read_only_interpretation(
        mut self,
        allow_read_only_interpretation: bool,
    ) -> Self {
        self.allow_read_only_interpretation = allow_read_only_interpretation;
        self
    }

    pub fn display_values(
        mut self,
        display_values: &'lifetime [AnonymousValueString],
    ) -> Self {
        self.display_values = Some(display_values);
        self
    }

    pub fn use_preview_foreground(
        mut self,
        use_preview_foreground: bool,
    ) -> Self {
        self.use_preview_foreground = use_preview_foreground;
        self
    }

    pub fn height(
        mut self,
        height: f32,
    ) -> Self {
        self.height = height;
        self
    }

    fn commit_on_enter_id(id: &str) -> Id {
        Id::new((Self::COMMIT_ON_ENTER_ID_SALT, id))
    }

    pub fn consume_commit_on_enter(
        user_interface: &mut Ui,
        id: &str,
    ) -> bool {
        let commit_on_enter_id = Self::commit_on_enter_id(id);
        let did_commit_on_enter = user_interface.memory(|memory| {
            memory
                .data
                .get_temp::<bool>(commit_on_enter_id)
                .unwrap_or(false)
        });

        if did_commit_on_enter {
            user_interface.memory_mut(|memory| memory.data.insert_temp(commit_on_enter_id, false));
        }

        did_commit_on_enter
    }
}

impl<'lifetime> Widget for DataValueBoxView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let down_arrow = &theme.icon_library.icon_handle_navigation_down_arrow_small;
        let symbol_registry = SymbolRegistry::get_instance();
        let is_valid = symbol_registry.validate_value_string(&self.validation_data_type, &self.anonymous_value_string);
        let foreground_color = match self.use_preview_foreground {
            true => theme.foreground_preview,
            false => theme.foreground,
        };
        let binary_color = match self.use_preview_foreground {
            true => theme.binary_blue_preview,
            false => theme.binary_blue,
        };
        let hexadecimal_color = match self.use_preview_foreground {
            true => theme.hexadecimal_green_preview,
            false => theme.hexadecimal_green,
        };
        let text_color = match is_valid {
            true => match self.anonymous_value_string.get_anonymous_value_string_format() {
                AnonymousValueStringFormat::Bool => foreground_color,
                AnonymousValueStringFormat::String => foreground_color,
                AnonymousValueStringFormat::Binary => binary_color,
                AnonymousValueStringFormat::Decimal => foreground_color,
                AnonymousValueStringFormat::Hexadecimal => hexadecimal_color,
                AnonymousValueStringFormat::Address => hexadecimal_color,
                AnonymousValueStringFormat::DataTypeRef => foreground_color,
                AnonymousValueStringFormat::Enumeration => foreground_color,
            },
            false => theme.error_red,
        };

        let desired_size = vec2(self.width, self.height);
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(desired_size, Sense::hover());
        let icon_size_vec = vec2(self.icon_size, self.icon_size);

        // Divider bar before right arrow.
        let button_width = self.icon_size + self.icon_padding * 2.0;
        let divider_x = allocated_size_rectangle.max.x - (button_width + self.divider_width);

        // The button rectangle (full clickable dropdown-area background).
        let dropdown_background_rectangle = Rect::from_min_max(
            pos2(divider_x + self.divider_width, allocated_size_rectangle.min.y),
            pos2(allocated_size_rectangle.max.x, allocated_size_rectangle.max.y),
        );

        let button_response = user_interface.interact(
            dropdown_background_rectangle,
            user_interface.make_persistent_id(format!("{}_button", self.id)),
            if self.is_read_only && !self.allow_read_only_interpretation {
                Sense::hover()
            } else {
                Sense::click()
            },
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
            pressed: button_response.is_pointer_button_down_on(),
            has_hover: button_response.hovered(),
            has_focus: button_response.has_focus(),
            corner_radius: CornerRadius::same(self.corner_radius),
            border_width: self.border_width,
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

        let mut text_value = self
            .anonymous_value_string
            .get_anonymous_value_string()
            .to_string();
        let mut text_edit_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(text_edit_rectangle_inner)
                .layout(Layout::right_to_left(Align::Center)),
        );

        let font_id = if text_value.len() > 0 {
            theme.font_library.font_ubuntu_mono_bold.font_normal.clone()
        } else {
            theme.font_library.font_noto_sans.font_normal.clone()
        };
        let text_edit_id = Id::new(format!("{}_text_edit", self.id));
        let text_edit_response = text_edit_user_interface.add(
            TextEdit::singleline(&mut text_value)
                .id(text_edit_id)
                .vertical_align(eframe::egui::Align::Center)
                .font(font_id.clone())
                .text_color(text_color)
                .hint_text(self.preview_text)
                .interactive(!self.is_read_only)
                .frame(false),
        );

        if self.border_width > 0.0 {
            user_interface.painter().rect_stroke(
                allocated_size_rectangle,
                CornerRadius::same(self.corner_radius),
                Stroke::new(self.border_width, theme.submenu_border),
                StrokeKind::Inside,
            );
        }

        // Draw drop-down arrow.
        user_interface.painter().image(
            down_arrow.id(),
            Rect::from_min_size(right_arrow_pos, icon_size_vec),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        // If the user changed text, update the display value
        if text_edit_response.changed() {
            self.anonymous_value_string
                .set_anonymous_value_string(text_value);
        }

        let commit_on_enter_pressed = text_edit_response.lost_focus() && user_interface.input(|input_state| input_state.key_pressed(Key::Enter));

        if commit_on_enter_pressed {
            user_interface.memory_mut(|memory| memory.data.insert_temp(Self::commit_on_enter_id(self.id), true));
        }

        // Popup logic.
        let popup_id = Id::new(("data_value_box_popup", self.id, user_interface.id().value()));
        let mut open = user_interface.memory(|memory| memory.data.get_temp::<bool>(popup_id).unwrap_or(false));

        if button_response.clicked() && (!self.is_read_only || self.allow_read_only_interpretation) {
            open = !open;
        }

        if self.is_read_only && !self.allow_read_only_interpretation {
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
        let popup_id_area = Id::new(("data_value_box_popup_area", self.id, user_interface.id().value()));
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
                        popup_user_interface.set_min_width(Self::MIN_POPUP_WIDTH);
                        popup_user_interface.with_layout(Layout::top_down(Align::Min), |inner_user_interface| {
                            let anonymous_value_string_formats = symbol_registry.get_supported_anonymous_value_string_formats(&self.validation_data_type);

                            for anonymous_value_string_format in &anonymous_value_string_formats {
                                let target_display_value = self.display_values.and_then(|display_values| {
                                    display_values
                                        .iter()
                                        .find(|display_value| display_value.get_anonymous_value_string_format() == *anonymous_value_string_format)
                                });

                                if inner_user_interface
                                    .add(DataValueBoxConvertItemView::new(
                                        self.app_context.clone(),
                                        self.anonymous_value_string,
                                        anonymous_value_string_format,
                                        target_display_value,
                                        false,
                                        self.is_value_owned,
                                        self.width.max(Self::MIN_POPUP_WIDTH),
                                    ))
                                    .clicked()
                                {
                                    should_close = true;
                                }
                            }

                            if self.is_value_owned && !self.is_read_only {
                                inner_user_interface.separator();

                                for anonymous_value_string_format in &anonymous_value_string_formats {
                                    if inner_user_interface
                                        .add(DataValueBoxConvertItemView::new(
                                            self.app_context.clone(),
                                            self.anonymous_value_string,
                                            anonymous_value_string_format,
                                            None,
                                            true,
                                            self.is_value_owned,
                                            self.width,
                                        ))
                                        .clicked()
                                    {
                                        should_close = true;
                                    }
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
