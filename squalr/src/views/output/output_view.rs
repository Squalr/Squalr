use crate::app_context::AppContext;
use crossbeam_channel::Receiver;
use eframe::egui::{Align, Label, Layout, Response, RichText, ScrollArea, Sense, Ui, Widget};
use epaint::Color32;
use log::Level;
use smallvec::SmallVec;
use squalr_engine_api::structures::logging::log_event::LogEvent;
use std::{rc::Rc, sync::RwLock};

#[derive(Clone)]
pub struct OutputView {
    app_context: Rc<AppContext>,
    log_receiver: Rc<Option<Receiver<LogEvent>>>,
    log_messages: Rc<RwLock<SmallVec<[LogEvent; 2048]>>>,
}

impl OutputView {
    pub fn new(app_context: Rc<AppContext>) -> Self {
        let log_receiver = Rc::new(
            match app_context
                .engine_execution_context
                .get_logger()
                .subscribe_to_logs()
            {
                Ok(receiver) => Some(receiver),
                Err(error) => {
                    log::error!("Error subscribing to engine logs, output will be limited: {}", error);
                    None
                }
            },
        );
        let log_messages = Rc::new(RwLock::new(SmallVec::<[LogEvent; 2048]>::default()));

        Self {
            app_context,
            log_receiver,
            log_messages,
        }
    }
}

impl Widget for OutputView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        // Process any new logs for display.
        match self.log_receiver.as_ref() {
            Some(log_receiver) => {
                while let Ok(log_message) = log_receiver.try_recv() {
                    if let Ok(mut log_messages) = self.log_messages.try_write() {
                        log_messages.push(log_message);
                    }
                }
            }
            None => {}
        }

        let theme = &self.app_context.theme;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                if let Ok(log_messages) = self.log_messages.try_read() {
                    // Wrap messages inside a vertical scroll area
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .stick_to_bottom(true)
                        .show(user_interface, |user_interface| {
                            for log_message in log_messages.iter() {
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
