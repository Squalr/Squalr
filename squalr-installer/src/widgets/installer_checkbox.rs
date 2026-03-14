use crate::theme::InstallerTheme;
use eframe::egui::{Align2, CornerRadius, Response, Sense, Stroke, StrokeKind, Ui, Widget, vec2};
use epaint::pos2;

pub(crate) struct InstallerCheckbox<'lifetime> {
    installer_theme: InstallerTheme,
    label: &'lifetime str,
    is_checked: &'lifetime mut bool,
}

impl<'lifetime> InstallerCheckbox<'lifetime> {
    const CHECKBOX_SIDE_LENGTH: f32 = 18.0;
    const ROW_HEIGHT: f32 = 22.0;

    pub(crate) fn new(
        installer_theme: InstallerTheme,
        label: &'lifetime str,
        is_checked: &'lifetime mut bool,
    ) -> Self {
        Self {
            installer_theme,
            label,
            is_checked,
        }
    }
}

impl Widget for InstallerCheckbox<'_> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (row_rectangle, mut response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::click());

        if response.clicked() {
            *self.is_checked = !*self.is_checked;
            response.mark_changed();
        }

        let checkbox_min_y = row_rectangle.center().y - Self::CHECKBOX_SIDE_LENGTH * 0.5;
        let checkbox_rectangle = eframe::egui::Rect::from_min_size(
            pos2(row_rectangle.left(), checkbox_min_y),
            vec2(Self::CHECKBOX_SIDE_LENGTH, Self::CHECKBOX_SIDE_LENGTH),
        );

        user_interface
            .painter()
            .rect_filled(checkbox_rectangle, CornerRadius::ZERO, self.installer_theme.color_background_control);
        user_interface.painter().rect_stroke(
            checkbox_rectangle,
            CornerRadius::ZERO,
            Stroke::new(1.0, self.installer_theme.color_border_panel),
            StrokeKind::Inside,
        );

        if response.hovered() {
            user_interface
                .painter()
                .rect_filled(checkbox_rectangle, CornerRadius::ZERO, self.installer_theme.color_hover_tint);
        }
        if response.is_pointer_button_down_on() {
            user_interface
                .painter()
                .rect_filled(checkbox_rectangle, CornerRadius::ZERO, self.installer_theme.color_pressed_tint);
        }

        if *self.is_checked {
            let check_start = pos2(checkbox_rectangle.left() + 4.0, checkbox_rectangle.center().y);
            let check_middle = pos2(checkbox_rectangle.left() + 7.5, checkbox_rectangle.bottom() - 4.5);
            let check_end = pos2(checkbox_rectangle.right() - 4.0, checkbox_rectangle.top() + 4.5);
            let check_stroke = Stroke::new(2.0, self.installer_theme.color_foreground);
            user_interface
                .painter()
                .line_segment([check_start, check_middle], check_stroke);
            user_interface
                .painter()
                .line_segment([check_middle, check_end], check_stroke);
        }

        let label_position = pos2(checkbox_rectangle.right() + 8.0, row_rectangle.center().y);
        user_interface.painter().text(
            label_position,
            Align2::LEFT_CENTER,
            self.label,
            self.installer_theme.fonts.font_normal.clone(),
            self.installer_theme.color_foreground,
        );

        response
    }
}
