use crate::app_context::AppContext;
use eframe::egui::{Align, Layout, Response, RichText, ScrollArea, Ui, UiBuilder, Widget};
use epaint::Vec2;
use log::Level;
use std::sync::Arc;

#[derive(Clone)]
pub struct OutputView {
    app_context: Arc<AppContext>,
}

impl OutputView {
    pub const WINDOW_ID: &'static str = "window_output";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self { app_context }
    }
}

impl Widget for OutputView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        // Process any new logs for display.
        let log_history = self
            .app_context
            .engine_unprivileged_state
            .get_logger()
            .get_log_history();

        let theme = &self.app_context.theme;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                if let Ok(log_history) = log_history.read() {
                    let outer_rectangle = user_interface.available_rect_before_wrap();
                    let inset_amount = Vec2::new(8.0, 4.0);
                    let inner_rectangle = outer_rectangle.shrink2(inset_amount);
                    let builder = UiBuilder::new()
                        .max_rect(inner_rectangle)
                        .layout(Layout::top_down(Align::Min));
                    let mut inner_user_interface = user_interface.new_child(builder);

                    ScrollArea::vertical()
                        .id_salt("output")
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(&mut inner_user_interface, |inner_user_interface| {
                            for log_message in log_history.iter() {
                                let color = match log_message.level {
                                    Level::Error => theme.background_control_danger,
                                    Level::Warn => theme.background_control_warning,
                                    Level::Info => theme.foreground,
                                    Level::Debug => theme.background_control_info,
                                    Level::Trace => theme.background_control_success,
                                };

                                inner_user_interface.label(
                                    RichText::new(&log_message.message)
                                        .color(color)
                                        .font(theme.font_library.font_noto_sans.font_normal.clone()),
                                );
                            }
                        });
                }
            })
            .response;

        response
    }
}
