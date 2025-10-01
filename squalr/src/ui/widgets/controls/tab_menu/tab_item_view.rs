use crate::app_context::AppContext;
use crate::ui::widgets::controls::state_layer::StateLayer;
use eframe::egui::{Response, Sense, Ui, Widget};
use epaint::{CornerRadius, pos2, vec2};
use std::rc::Rc;

pub struct TabItemView<'a> {
    app_context: Rc<AppContext>,
    header: &'a String,
    height: f32,
    horizontal_padding: f32,
}

impl<'a> TabItemView<'a> {
    pub fn new(
        app_context: Rc<AppContext>,
        header: &'a String,
        height: f32,
        horizontal_padding: f32,
    ) -> Self {
        Self {
            app_context,
            header,
            height,
            horizontal_padding,
        }
    }
}

impl<'a> Widget for TabItemView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        // Measure header text and compute padded size.
        let theme = &self.app_context.theme;
        let font_id = theme.font_library.font_noto_sans.font_header.clone();
        let text_color = theme.foreground;
        let header_galley = user_interface
            .ctx()
            .fonts(|fonts| fonts.layout_no_wrap(self.header.clone(), font_id.clone(), text_color));
        let text_size = header_galley.size();
        let style = user_interface.style().clone();
        let padding_vertical = style.spacing.button_padding.y.max(4.0);
        let padding_horizontal = self.horizontal_padding.max(style.spacing.button_padding.x);
        let desired = vec2(
            text_size.x + 2.0 * padding_horizontal,
            (self.height.max(0.0)).max(text_size.y + 2.0 * padding_vertical),
        );

        let (available_size_rectangle, response) = user_interface.allocate_exact_size(desired, Sense::click());

        // Compose the StateLayer (hover/press/focus) like the Button impl.
        StateLayer {
            bounds_min: available_size_rectangle.min,
            bounds_max: available_size_rectangle.max,
            enabled: true,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius { nw: 4, ne: 4, sw: 4, se: 4 },
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_primary_dark,
            border_color_focused: theme.background_control_primary_dark,
        }
        .ui(user_interface);

        // Header label centered vertically.
        let text_pos = pos2(
            available_size_rectangle.min.x + padding_horizontal,
            available_size_rectangle.center().y - text_size.y * 0.5,
        );

        user_interface
            .painter()
            .galley(text_pos, header_galley, text_color);

        response
    }
}
