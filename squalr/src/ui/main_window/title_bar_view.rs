use crate::ui::{controls::button::Button, theme::Theme};
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{self, Align, Context, Id, Layout, RichText, Sense, Ui};
use epaint::CornerRadius;

#[derive(Default)]
pub struct TitleBar {
    pub title: String,
    pub height: f32,
}

impl TitleBar {
    pub fn draw(
        &self,
        user_interface: &mut Ui,
        context: &Context,
        theme: &Theme,
    ) {
        let rect = user_interface.max_rect();
        let mut input_handled = false;

        user_interface
            .painter()
            .rect_filled(rect, CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 }, theme.background_primary);

        user_interface.with_layout(Layout::left_to_right(Align::Min), |user_interface| {
            // Left side: app icon + title.
            user_interface.label(RichText::new("<ICON HERE>").size(16.0)); // Font size, not actual size.
            user_interface.label(RichText::new(&self.title).color(theme.foreground));

            // Right side: window controls.
            user_interface.with_layout(Layout::right_to_left(Align::Min), |user_interface| {
                input_handled = self.draw_buttons(user_interface, context, theme);
            });
        });

        if input_handled {
            return;
        }

        let title_bar_interact = user_interface.interact(rect, Id::new("titlebar"), Sense::click_and_drag());

        if title_bar_interact.drag_started() {
            context.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        if title_bar_interact.double_clicked() {
            let is_max = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));

            context.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }
    }

    fn draw_buttons(
        &self,
        user_interface: &mut Ui,
        context: &Context,
        theme: &Theme,
    ) -> bool {
        let mut input_handled = false;
        let button_size = egui::vec2(28.0, 28.0);

        user_interface.set_height(button_size.x);
        user_interface.add_space(4.0);

        // Close.
        if user_interface
            .add_sized(
                button_size,
                Button {
                    text: "X",
                    margin: 0,
                    ..Button::new_from_theme(theme)
                },
            )
            .clicked()
        {
            context.send_viewport_cmd(ViewportCommand::Close);

            input_handled = true;
        }

        // Maximize.
        if user_interface
            .add_sized(
                button_size,
                Button {
                    text: "X",
                    margin: 0,
                    ..Button::new_from_theme(theme)
                },
            )
            .clicked()
        {
            let is_max = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));

            context.send_viewport_cmd(ViewportCommand::Maximized(!is_max));

            input_handled = true;
        }

        // Minimize.
        if user_interface
            .add_sized(
                button_size,
                Button {
                    text: "X",
                    margin: 0,
                    ..Button::new_from_theme(theme)
                },
            )
            .clicked()
        {
            context.send_viewport_cmd(ViewportCommand::Minimized(true));

            input_handled = true;
        }

        input_handled
    }
}
