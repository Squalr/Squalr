use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::theme::Theme;
use crate::ui::widgets::controls::button::Button;
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{Align, Context, Id, Layout, Rect, Response, RichText, Sense, Ui, UiBuilder, pos2};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct MainTitleBarView {
    context: Context,
    theme: Rc<Theme>,
    corner_radius: CornerRadius,
    height: f32,
    title: String,
}

impl MainTitleBarView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        corner_radius: CornerRadius,
        height: f32,
        title: String,
    ) -> Self {
        Self {
            context,
            theme,
            corner_radius,
            height,
            title,
        }
    }
}

impl eframe::egui::Widget for MainTitleBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::empty());

        user_interface.painter().rect_filled(
            rect,
            CornerRadius {
                nw: self.corner_radius.nw,
                ne: self.corner_radius.ne,
                sw: 0,
                se: 0,
            },
            self.theme.background_primary,
        );

        // Create a child ui constrained to the title bar.
        let builder = UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);
        let mut buttons_rect: Option<Rect> = None;

        // Hard-clip to the titlebar.
        child_user_interface.set_clip_rect(rect);

        // Create the app icon / name.
        child_user_interface.add_space(8.0);

        let [texture_width, texture_height] = self.theme.icon_library.icon_handle_logo.size();
        let (_id, app_icon_rect) = child_user_interface.allocate_space(vec2(texture_width as f32, texture_height as f32));
        IconDraw::draw(&child_user_interface, app_icon_rect, &self.theme.icon_library.icon_handle_logo);

        child_user_interface.add_space(4.0);

        child_user_interface.label(
            RichText::new(&self.title)
                .font(self.theme.font_library.font_noto_sans.font_window_title.clone())
                .color(self.theme.foreground),
        );

        // Push the rest (buttons) to the far right within the same row.
        child_user_interface.add_space(child_user_interface.available_width());

        // Create the buttons right-to-left.
        child_user_interface.with_layout(Layout::right_to_left(Align::Center), |user_interface| {
            let button_size = vec2(36.0, 32.0);

            // Close.
            let close = user_interface.add_sized(button_size, Button::new_from_theme(&self.theme));
            IconDraw::draw(user_interface, close.rect, &self.theme.icon_library.icon_handle_close);

            if close.clicked() {
                self.context.send_viewport_cmd(ViewportCommand::Close);
            }

            // Maximize / Restore.
            let max = user_interface.add_sized(button_size, Button::new_from_theme(&self.theme));
            IconDraw::draw(user_interface, max.rect, &self.theme.icon_library.icon_handle_maximize);

            if max.clicked() {
                let is_max = self
                    .context
                    .input(|input_state| input_state.viewport().maximized.unwrap_or(false));
                self.context
                    .send_viewport_cmd(ViewportCommand::Maximized(!is_max));
            }

            // Minimize.
            let min = user_interface.add_sized(button_size, Button::new_from_theme(&self.theme));
            IconDraw::draw(user_interface, min.rect, &self.theme.icon_library.icon_handle_minimize);

            if min.clicked() {
                self.context.send_viewport_cmd(ViewportCommand::Minimized(true));
            }

            buttons_rect = Some(close.rect.union(max.rect).union(min.rect));
        });

        // Drag area = everything left of the buttons, inside the titlebar rect.
        let right_edge = buttons_rect.map(|r| r.min.x).unwrap_or(rect.max.x);
        let drag_rect = Rect::from_min_max(rect.min, pos2(right_edge, rect.max.y));
        let drag = user_interface.interact(drag_rect, Id::new("titlebar"), Sense::click_and_drag());

        if drag.drag_started() {
            self.context.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        if drag.double_clicked() {
            let is_max = self
                .context
                .input(|input_state| input_state.viewport().maximized.unwrap_or(false));
            self.context
                .send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }

        response
    }
}
