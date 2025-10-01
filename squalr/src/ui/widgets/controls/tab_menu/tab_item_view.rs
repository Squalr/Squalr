use crate::app_context::AppContext;
use crate::ui::widgets::controls::state_layer::StateLayer;
use eframe::egui::{Response, Sense, Ui, Widget};
use epaint::{CornerRadius, Stroke, StrokeKind, pos2, vec2};
use std::rc::Rc;

pub struct TabItemView<'a> {
    app_context: Rc<AppContext>,
    header: &'a String,
    min_width: f32,
    height: f32,
    horizontal_padding: f32,
    is_selected: bool,
}

impl<'a> TabItemView<'a> {
    pub fn new(
        app_context: Rc<AppContext>,
        header: &'a String,
        min_width: f32,
        height: f32,
        horizontal_padding: f32,
        is_selected: bool,
    ) -> Self {
        Self {
            app_context,
            header,
            min_width,
            height,
            horizontal_padding,
            is_selected,
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
            self.min_width.max(text_size.x + 2.0 * padding_horizontal),
            (self.height.max(0.0)).max(text_size.y + 2.0 * padding_vertical),
        );
        let (available_size_rectangle, response) = user_interface.allocate_exact_size(desired, Sense::click());
        let corner_radius = CornerRadius { nw: 4, ne: 4, sw: 0, se: 0 };

        user_interface.painter().rect(
            available_size_rectangle,
            corner_radius,
            match self.is_selected {
                true => theme.background_control_primary,
                false => theme.background_control_secondary,
            },
            Stroke {
                width: 1.0,
                color: match self.is_selected {
                    true => theme.background_control_primary_dark,
                    false => theme.background_control_secondary_dark,
                },
            },
            StrokeKind::Inside,
        );

        // Compose the StateLayer (hover/press/focus) like the Button impl.
        StateLayer {
            bounds_min: available_size_rectangle.min,
            bounds_max: available_size_rectangle.max,
            enabled: true,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_primary_dark,
            border_color_focused: theme.background_control_primary_dark,
        }
        .ui(user_interface);

        // Header label centered vertically.
        let text_pos = pos2(
            available_size_rectangle.center().x - text_size.x * 0.5,
            available_size_rectangle.center().y - text_size.y * 0.5,
        );

        user_interface
            .painter()
            .galley(text_pos, header_galley, text_color);

        response
    }
}
