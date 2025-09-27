use eframe::egui::{self, Color32, CornerRadius, Label, Margin, Response, RichText, Stroke, Ui, Widget};

use crate::ui::theme::Theme;

#[derive(Default)]
pub struct Button<'a> {
    pub disabled: bool,
    pub text: &'a str,
    pub tooltip_text: &'a str,
    pub corner_radius: u8,
    pub border_width: f32,
    pub margin: i8,
    pub background_color: Color32,
    pub foreground_color: Color32,
    pub hover_color: Color32,
    pub pressed_color: Color32,
    pub border_color: Color32,
    pub click_sound: Option<&'a str>,
}

impl<'a> Button<'a> {
    pub fn new_from_theme(theme: &Theme) -> Button<'a> {
        let mut button = Button::default();

        button.corner_radius = 4;
        button.border_width = 2.0;
        button.margin = 4;
        button.background_color = theme.background_control_primary;
        button.foreground_color = theme.foreground;
        button.hover_color = theme.hover_tint;
        button.pressed_color = theme.pressed_tint;
        button.border_color = theme.background_control_primary_dark;

        button
    }
}

impl<'a> Widget for Button<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let button_frame = egui::Frame::new()
            .fill(self.background_color)
            .stroke(Stroke::new(self.border_width, self.border_color))
            .corner_radius(CornerRadius::same(self.corner_radius))
            .inner_margin(Margin::same(self.margin));

        let mut response = user_interface
            .add_enabled_ui(!self.disabled, |user_interface| {
                button_frame
                    .show(user_interface, |user_interface| {
                        user_interface.centered_and_justified(|user_interface| {
                            user_interface.add(Label::new(RichText::new(self.text).color(self.foreground_color)).selectable(false));
                        });
                    })
                    .response
            })
            .inner;

        if !self.tooltip_text.is_empty() {
            response = response.on_hover_text(self.tooltip_text);
        }

        if response.hovered() {
            user_interface
                .painter()
                .rect_filled(response.rect, CornerRadius::same(self.corner_radius), self.hover_color);
        }

        if response.is_pointer_button_down_on() {
            user_interface
                .painter()
                .rect_filled(response.rect, CornerRadius::same(self.corner_radius), self.pressed_color);
        }

        if response.clicked() {
            if let Some(sound) = self.click_sound {
                println!("JIRA: Play sound: {}", sound);
            }
        }

        response
    }
}
