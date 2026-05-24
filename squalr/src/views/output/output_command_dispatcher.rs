use crate::app_context::AppContext;
use squalr_engine_api::commands::command_invocation::CommandInvocationSource;
use squalr_engine_api::commands::command_line::clap::ErrorKind;
use squalr_engine_api::commands::command_line::{CommandLineCommand, CommandLineParseError, format_prompt_command_error, parse_prompt_command_line};
use std::sync::Arc;

pub struct OutputCommandDispatcher;

impl OutputCommandDispatcher {
    pub fn dispatch(
        app_context: &Arc<AppContext>,
        command_text: String,
    ) {
        log::info!("{}", Self::format_command_echo(&command_text));

        match parse_prompt_command_line(&command_text) {
            Ok(CommandLineCommand::Privileged(command)) => {
                let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();
                let egui_context = app_context.context.clone();

                engine_unprivileged_state.dispatch_privileged_command_from_source(CommandInvocationSource::Prompt, command, move |_response| {
                    egui_context.request_repaint();
                });
            }
            Ok(CommandLineCommand::Unprivileged(command)) => {
                let engine_unprivileged_state = app_context.engine_unprivileged_state.clone();
                let egui_context = app_context.context.clone();

                engine_unprivileged_state.dispatch_unprivileged_command_from_source(CommandInvocationSource::Prompt, command, move |_response| {
                    egui_context.request_repaint();
                });
            }
            Err(CommandLineParseError::Command(error)) if matches!(error.kind, ErrorKind::HelpDisplayed | ErrorKind::VersionDisplayed) => {
                log::info!("{}", format_prompt_command_error(&error));
            }
            Err(CommandLineParseError::Command(error)) => {
                log::error!("{}", format_prompt_command_error(&error));
            }
            Err(error) => {
                log::error!("{}", error);
            }
        }

        app_context.context.request_repaint();
    }

    fn format_command_echo(command_text: &str) -> String {
        format!("> {}", command_text)
    }
}

#[cfg(test)]
mod tests {
    use super::OutputCommandDispatcher;

    #[test]
    fn command_echo_uses_prompt_prefix() {
        assert_eq!(OutputCommandDispatcher::format_command_echo("scan new"), "> scan new");
    }
}
