use crate::ui::controls::button::Button;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::theme::Theme;
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{Align, Context, Id, Layout, Rect, Response, RichText, Sense, Ui, UiBuilder, pos2};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct TitleBarView {
    pub context: Context,
    pub theme: Rc<Theme>,
    pub title: String,
    pub height: f32,
}

impl eframe::egui::Widget for TitleBarView {
    fn ui(
        self,
        ui: &mut Ui,
    ) -> Response {
        // Reserve exactly one row of height = title bar.
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), self.height), Sense::empty());

        // Paint background.
        ui.painter()
            .rect_filled(rect, CornerRadius { nw: 4, ne: 4, sw: 0, se: 0 }, self.theme.background_primary);

        // Create a child Ui constrained to *this* rect (no wrapping outside).
        let builder = UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center));
        let mut bar_ui = ui.new_child(builder);
        bar_ui.set_clip_rect(rect); // hard-clip to the titlebar

        let mut buttons_rect: Option<Rect> = None;

        // Left: app icon + title
        bar_ui.add_space(8.0);
        let [iw, ih] = self.theme.icon_library.icon_handle_app.size();
        let (_id, app_icon_rect) = bar_ui.allocate_space(vec2(iw as f32, ih as f32));
        IconDraw::draw(&bar_ui, app_icon_rect, &self.theme.icon_library.icon_handle_app);

        bar_ui.label(RichText::new(&self.title).color(self.theme.foreground));

        // Push the rest (buttons) to the far right within the same row.
        bar_ui.add_space(bar_ui.available_width());

        // Right: window buttons, right-to-left so Close ends up at the far right.
        bar_ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let button_size = vec2(36.0, 32.0);

            // Close
            let close = ui.add_sized(button_size, Button::new_from_theme(&self.theme));
            IconDraw::draw(ui, close.rect, &self.theme.icon_library.icon_handle_close);

            if close.clicked() {
                self.context.send_viewport_cmd(ViewportCommand::Close);
            }

            // Maximize / Restore
            let max = ui.add_sized(button_size, Button::new_from_theme(&self.theme));
            IconDraw::draw(ui, max.rect, &self.theme.icon_library.icon_handle_maximize);

            if max.clicked() {
                let is_max = self.context.input(|i| i.viewport().maximized.unwrap_or(false));
                self.context
                    .send_viewport_cmd(ViewportCommand::Maximized(!is_max));
            }

            // Minimize
            let min = ui.add_sized(button_size, Button::new_from_theme(&self.theme));
            IconDraw::draw(ui, min.rect, &self.theme.icon_library.icon_handle_minimize);

            if min.clicked() {
                self.context.send_viewport_cmd(ViewportCommand::Minimized(true));
            }

            buttons_rect = Some(close.rect.union(max.rect).union(min.rect));
        });

        // Drag area = everything left of the buttons, inside the titlebar rect.
        let right_edge = buttons_rect.map(|r| r.min.x).unwrap_or(rect.max.x);
        let drag_rect = Rect::from_min_max(rect.min, pos2(right_edge, rect.max.y));
        let drag = ui.interact(drag_rect, Id::new("titlebar"), Sense::click_and_drag());

        if drag.drag_started() {
            self.context.send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if drag.double_clicked() {
            let is_max = self.context.input(|i| i.viewport().maximized.unwrap_or(false));
            self.context
                .send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }

        response
    }
}
