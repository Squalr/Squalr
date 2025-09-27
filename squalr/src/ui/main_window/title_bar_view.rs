use crate::ui::{controls::button::Button, theme::Theme};
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{Context, Id, RichText, Sense, Ui};
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
        ctx: &Context,
        theme: &Theme,
    ) {
        let rect = user_interface.max_rect();
        user_interface
            .painter()
            .rect_filled(rect, CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 }, theme.background_primary);

        // Interactions
        let resp = user_interface.interact(rect, Id::new("titlebar"), Sense::click_and_drag());
        if resp.drag_started() {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if resp.double_clicked() {
            let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
            ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }

        user_interface.horizontal(|user_interface| {
            // Left side: app icon + title.
            user_interface.add_space(4.0);
            user_interface.label(RichText::new("🟦").size(16.0));
            user_interface.add_space(6.0);
            user_interface.label(RichText::new(&self.title).color(theme.foreground));

            user_interface.add_space(user_interface.available_width() - 3.0 * 36.0);

            // Right side: window controls.
            self.draw_buttons(user_interface, ctx, theme);
        });
    }

    fn draw_buttons(
        &self,
        user_interface: &mut Ui,
        ctx: &Context,
        theme: &Theme,
    ) {
        // Close
        let close = Button {
            text: "✕",
            tooltip_text: "Close",
            ..Default::default()
        };
        if close.draw(user_interface).clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        // Maximize
        let maximize = Button {
            text: "🗖",
            tooltip_text: "Maximize",
            ..Default::default()
        };
        if maximize.draw(user_interface).clicked() {
            let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
            ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }

        // Minimize
        let minimize = Button {
            text: "🗕",
            tooltip_text: "Minimize",
            ..Default::default()
        };
        if minimize.draw(user_interface).clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        }
    }
}
