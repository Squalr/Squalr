use crate::ui::controls::button::Button;
use crate::ui::theme::Theme;
use eframe::egui::pos2;
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{self, Align, Context, Id, Layout, Rect, RichText, Sense, Ui};
use epaint::{Color32, CornerRadius};

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
        let full_rect = user_interface.max_rect();

        // Background.
        user_interface
            .painter()
            .rect_filled(full_rect, CornerRadius { nw: 4, ne: 4, sw: 0, se: 0 }, theme.background_primary);

        // We'll capture where the buttons end up:
        let mut buttons_rect: Option<Rect> = None;

        // Single row: left = icon/title, right = buttons.
        user_interface.allocate_ui_with_layout(user_interface.available_size(), Layout::left_to_right(Align::Center), |user_interface| {
            user_interface.add_space(8.0);

            // Left side: icon + title.
            let icon_size = theme.icon_library.icon_handle_app.size();
            let (_icon_id, icon_rect) = user_interface.allocate_space(egui::vec2(icon_size[0] as f32, icon_size[1] as f32));

            // Draw the icon into the allocated rect
            user_interface.painter().image(
                theme.icon_library.icon_handle_app.id(),
                Rect::from_min_size(icon_rect.min, egui::vec2(icon_size[0] as f32, icon_size[1] as f32)),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), // UV coords
                Color32::WHITE,
            );
            user_interface.label(RichText::new(&self.title).color(theme.foreground));

            // Fill remaining space so the next child sits at the far right.
            user_interface.allocate_space(egui::vec2(user_interface.available_width(), 0.0));

            // Right side: measure buttons rect by drawing them in a child with RTL layout.
            user_interface.allocate_ui_with_layout(user_interface.available_size(), Layout::right_to_left(Align::Center), |user_interface| {
                buttons_rect = Some(self.draw_buttons(user_interface, context, theme));
            });
        });

        // Compute draggable rect = everything left of the buttons
        let right_edge_of_drag = buttons_rect.map(|r| r.min.x).unwrap_or(full_rect.max.x);
        let drag_rect = Rect::from_min_max(full_rect.min, pos2(right_edge_of_drag, full_rect.max.y));

        let resp = user_interface.interact(drag_rect, Id::new("titlebar"), Sense::click_and_drag());

        if resp.drag_started() {
            context.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        if resp.double_clicked() {
            let is_max = context.input(|i| i.viewport().maximized.unwrap_or(false));

            context.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }
    }

    /// Draws the three window buttons and returns their UNION rect.
    fn draw_buttons(
        &self,
        user_interface: &mut Ui,
        context: &Context,
        theme: &Theme,
    ) -> Rect {
        let button_size = egui::vec2(36.0, 32.0);

        user_interface.set_height(button_size.y);
        user_interface.add_space(8.0);

        // Close.
        let result_close = user_interface.add_sized(
            button_size,
            Button {
                margin: 0,
                ..Button::new_from_theme(theme)
            },
        );

        let texture_size = theme.icon_library.icon_handle_close.size();
        let texture_rect = Rect::from_center_size(result_close.rect.center(), egui::vec2(texture_size[0] as f32, texture_size[1] as f32));

        user_interface.painter().image(
            theme.icon_library.icon_handle_close.id(),
            texture_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        if result_close.clicked() {
            context.send_viewport_cmd(ViewportCommand::Close);
        }

        // Maximize / Restore.
        let result_max = user_interface.add_sized(
            button_size,
            Button {
                margin: 0,
                ..Button::new_from_theme(theme)
            },
        );

        let texture_size = theme.icon_library.icon_handle_maximize.size();
        let texture_rect = Rect::from_center_size(result_max.rect.center(), egui::vec2(texture_size[0] as f32, texture_size[1] as f32));

        user_interface.painter().image(
            theme.icon_library.icon_handle_maximize.id(),
            texture_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        if result_max.clicked() {
            let is_max = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));

            context.send_viewport_cmd(ViewportCommand::Maximized(!is_max));
        }

        // Minimize.
        let result_min = user_interface.add_sized(
            button_size,
            Button {
                margin: 0,
                ..Button::new_from_theme(theme)
            },
        );

        let texture_size = theme.icon_library.icon_handle_minimize.size();
        let texture_rect = Rect::from_center_size(result_min.rect.center(), egui::vec2(texture_size[0] as f32, texture_size[1] as f32));

        user_interface.painter().image(
            theme.icon_library.icon_handle_minimize.id(),
            texture_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        if result_min.clicked() {
            context.send_viewport_cmd(ViewportCommand::Minimized(true));
        }

        // Return the bounding rect actually used by the buttons.
        result_close.rect.union(result_max.rect).union(result_min.rect)
    }
}
