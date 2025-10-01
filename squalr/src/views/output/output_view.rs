use crate::app_context::AppContext;
use eframe::egui::{Align, Label, Layout, Response, RichText, ScrollArea, Ui, Widget};
use epaint::Color32;
use log::Level;
use std::rc::Rc;

#[derive(Clone)]
pub struct OutputView {
    app_context: Rc<AppContext>,
}

impl OutputView {
    pub fn new(app_context: Rc<AppContext>) -> Self {
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
            .engine_execution_context
            .get_logger()
            .get_log_history();

        let theme = &self.app_context.theme;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                if let Ok(log_history) = log_history.try_read() {
                    // Wrap messages inside a vertical scroll area.
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .stick_to_bottom(true)
                        .show(user_interface, |user_interface| {
                            for log_message in log_history.iter() {
                                let color = match log_message.level {
                                    Level::Error => Color32::RED,
                                    Level::Warn => Color32::YELLOW,
                                    Level::Info => theme.foreground,
                                    Level::Debug => Color32::LIGHT_BLUE,
                                    Level::Trace => Color32::GREEN,
                                };

                                user_interface.add(Label::new(RichText::new(&log_message.message).color(color)));
                            }
                        });
                }
            })
            .response;

        response
    }
}
