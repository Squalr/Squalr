use eframe::egui::{self, Color32, CornerRadius, Label, Margin, Response, RichText, Stroke, Ui};

#[derive(Default)]
pub struct Button<'a> {
    pub enabled: bool,
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
    pub click_sound: Option<&'a str>, // file path or handle
}

impl<'a> Button<'a> {
    pub fn draw(
        &self,
        user_interface: &mut Ui,
    ) -> Response {
        // Wrap in a Frame to handle border/rounding
        let frame = egui::Frame::new()
            .fill(self.background_color)
            .stroke(Stroke::new(self.border_width, self.border_color))
            .corner_radius(CornerRadius::same(self.corner_radius))
            .inner_margin(Margin::same(self.margin));

        let mut response = user_interface
            .add_enabled_ui(self.enabled, |user_interface| {
                frame
                    .show(user_interface, |user_interface: &mut Ui| {
                        user_interface.centered_and_justified(|user_interface| {
                            user_interface.add(Label::new(RichText::new(self.text).color(self.foreground_color)).selectable(false));
                        });
                    })
                    .response
            })
            .inner;

        // Tooltip
        if !self.tooltip_text.is_empty() {
            response = response.on_hover_text(self.tooltip_text);
        }

        // Apply hover/pressed tints manually
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

        // Sound hook
        if response.clicked() {
            if let Some(sound) = self.click_sound {
                // hook into your audio system here
                println!("Play sound: {}", sound);
            }
        }

        response
    }
}
