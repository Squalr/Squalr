use crate::app_context::AppContext;
use squalr_engine_api::commands::text::clap::ErrorKind;
use squalr_engine_api::commands::text::{TextCommand, TextCommandParseError, parse_command_line};
use std::sync::Arc;

pub struct OutputCommandDispatcher;

impl OutputCommandDispatcher {
    pub fn dispatch(
        app_context: &Arc<AppContext>,
        command_text: String,
    ) {
        match parse_command_line(&command_text) {
            Ok(TextCommand::Privileged(command)) => {
                let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();
                let egui_context = app_context.context.clone();

                engine_unprivileged_state.dispatch_command(command, move |_response| {
                    egui_context.request_repaint();
                });
            }
            Ok(TextCommand::Unprivileged(command)) => {
                let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();
                let egui_context = app_context.context.clone();

                engine_unprivileged_state.dispatch_unprivileged_command(command, move |_response| {
                    egui_context.request_repaint();
                });
            }
            Err(TextCommandParseError::Command(error)) if matches!(error.kind, ErrorKind::HelpDisplayed | ErrorKind::VersionDisplayed) => {
                log::info!("{}", error);
            }
            Err(error) => {
                log::error!("Error parsing output command: {}", error);
            }
        }

        app_context.context.request_repaint();
    }
}
