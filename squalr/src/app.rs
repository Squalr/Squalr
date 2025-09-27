use crate::ui::{controls::button::Button, main_window::title_bar_view::TitleBar, theme::Theme};
use eframe::egui::{CentralPanel, Context, Frame, UiBuilder, Visuals};
use epaint::{Pos2, Rect, Rgba};

pub struct SqualrGui {
    counter: i32,
    theme: Theme,
}

impl SqualrGui {
    pub fn new(context: &Context) -> Self {
        Self {
            counter: 0,
            theme: Theme::new(context),
        }
    }
}

impl eframe::App for SqualrGui {
    fn clear_color(
        &self,
        _visuals: &Visuals,
    ) -> [f32; 4] {
        Rgba::TRANSPARENT.to_array()
    }

    fn update(
        &mut self,
        context: &Context,
        _frame: &mut eframe::Frame,
    ) {
        let title_bar = TitleBar {
            title: "Squalr".to_string(),
            height: 32.0,
        };
        let button = Button {
            // text: "Click Me",
            tooltip_text: "Tooltip.",
            ..Button::new_from_theme(&self.theme)
        };
        let panel_frame = Frame::new()
            .fill(context.style().visuals.window_fill())
            .corner_radius(10)
            .stroke(context.style().visuals.widgets.noninteractive.fg_stroke)
            .outer_margin(1.0);

        CentralPanel::default()
            .frame(panel_frame)
            .show(context, move |user_interface| {
                let app_rect = user_interface.max_rect();

                // Reserve a rect at the top for the title bar
                let title_bar_rect = Rect {
                    min: app_rect.min,
                    max: Pos2 {
                        x: app_rect.max.x,
                        y: app_rect.min.y + title_bar.height,
                    },
                };

                // Draw the title bar (yours handles dragging + buttons)
                let mut title_ui = user_interface.new_child(UiBuilder::new().max_rect(title_bar_rect));

                title_bar.draw(&mut title_ui, context, &self.theme);

                // Content area below the title bar
                let content_rect = {
                    let mut rect = app_rect;
                    rect.min.y = title_bar_rect.max.y;
                    rect
                };

                // Paint a background for the content area
                user_interface
                    .painter()
                    .rect_filled(content_rect, 0.0, self.theme.background_control);

                let mut content_ui = user_interface.new_child(UiBuilder::new().max_rect(content_rect));

                if content_ui.add_sized([128.0, 64.0], button).clicked() {
                    self.counter += 1;
                };

                content_ui.label(format!("Counter: {}", self.counter));
            });
    }
}
