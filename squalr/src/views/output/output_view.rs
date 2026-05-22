use crate::app_context::AppContext;
use crate::views::output::output_command_dispatcher::OutputCommandDispatcher;
use crate::views::output::output_command_state::OutputCommandState;
use eframe::egui::{Align, Button, Key, Layout, Response, RichText, ScrollArea, Sense, TextEdit, Ui, UiBuilder, ViewportCommand, Widget};
use epaint::{CornerRadius, Rect, Stroke, Vec2, pos2};
use log::Level;
use squalr_engine_api::structures::logging::log_event::LogEvent;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct OutputView {
    app_context: Arc<AppContext>,
    command_state: Arc<RwLock<OutputCommandState>>,
}

impl OutputView {
    pub const WINDOW_ID: &'static str = "window_output";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self {
            app_context,
            command_state: Arc::new(RwLock::new(OutputCommandState::new())),
        }
    }

    fn build_log_history_copy_text(log_history: &VecDeque<LogEvent>) -> String {
        log_history
            .iter()
            .map(|log_message| log_message.message.as_str())
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn render_output_context_menu(
        response: &Response,
        copy_text: String,
    ) {
        response.context_menu(|menu_user_interface| {
            if menu_user_interface
                .add_enabled(!copy_text.is_empty(), Button::new("Copy"))
                .clicked()
            {
                menu_user_interface.ctx().copy_text(copy_text);
                menu_user_interface.close();
            }
        });
    }

    fn render_command_input_context_menu(response: &Response) {
        response.context_menu(|menu_user_interface| {
            if menu_user_interface.button("Cut").clicked() {
                menu_user_interface
                    .ctx()
                    .send_viewport_cmd(ViewportCommand::RequestCut);
                menu_user_interface.close();
            }

            if menu_user_interface.button("Copy").clicked() {
                menu_user_interface
                    .ctx()
                    .send_viewport_cmd(ViewportCommand::RequestCopy);
                menu_user_interface.close();
            }

            if menu_user_interface.button("Paste").clicked() {
                menu_user_interface
                    .ctx()
                    .send_viewport_cmd(ViewportCommand::RequestPaste);
                menu_user_interface.close();
            }
        });
    }

    fn draw_command_line(
        &self,
        user_interface: &mut Ui,
        command_line_rectangle: Rect,
    ) -> Option<String> {
        let theme = &self.app_context.theme;
        let mut command_to_dispatch = None;

        user_interface
            .painter()
            .rect_filled(command_line_rectangle, CornerRadius::ZERO, theme.background_primary);
        user_interface.painter().line_segment(
            [
                command_line_rectangle.left_top(),
                command_line_rectangle.right_top(),
            ],
            Stroke::new(1.0, theme.submenu_border),
        );

        let mut command_line_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(command_line_rectangle)
                .sense(Sense::click()),
        );

        match self.command_state.write() {
            Ok(mut command_state) => {
                let text_edit_response = command_line_user_interface.put(
                    command_line_rectangle,
                    TextEdit::singleline(command_state.command_text_mut())
                        .vertical_align(Align::Center)
                        .font(theme.font_library.font_noto_sans.font_normal.clone())
                        .background_color(theme.background_primary)
                        .text_color(theme.foreground)
                        .frame(false),
                );

                Self::render_command_input_context_menu(&text_edit_response);

                if text_edit_response.has_focus() && command_line_user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp)) {
                    command_state.navigate_previous();
                }

                if text_edit_response.has_focus() && command_line_user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown)) {
                    command_state.navigate_next();
                }

                if command_line_user_interface.input(|input_state| input_state.key_pressed(Key::Enter))
                    && (text_edit_response.has_focus() || text_edit_response.lost_focus())
                {
                    command_to_dispatch = command_state.submit_command();
                    text_edit_response.request_focus();
                }
            }
            Err(error) => {
                log::error!("Failed to acquire output command state write lock: {}", error);
            }
        }

        command_to_dispatch
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
                let outer_rectangle = user_interface.available_rect_before_wrap();
                let inset_amount = Vec2::new(8.0, 4.0);
                let inner_rectangle = outer_rectangle.shrink2(inset_amount);
                let command_line_height = 36.0;
                let command_line_top = (outer_rectangle.max.y - command_line_height).max(outer_rectangle.min.y);
                let log_rectangle = Rect::from_min_max(inner_rectangle.min, pos2(inner_rectangle.max.x, command_line_top.max(inner_rectangle.min.y)));
                let command_line_rectangle = Rect::from_min_max(pos2(outer_rectangle.min.x, command_line_top), outer_rectangle.max);
                let log_builder = UiBuilder::new()
                    .max_rect(log_rectangle)
                    .layout(Layout::top_down(Align::Min));
                let mut log_user_interface = user_interface.new_child(log_builder);
                let mut log_copy_text = String::new();

                match log_history.read() {
                    Ok(log_history) => {
                        log_copy_text = Self::build_log_history_copy_text(&log_history);

                        ScrollArea::vertical()
                            .id_salt("output")
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .show(&mut log_user_interface, |log_user_interface| {
                                for log_message in log_history.iter() {
                                    let color = match log_message.level {
                                        Level::Error => theme.background_control_danger,
                                        Level::Warn => theme.background_control_warning,
                                        Level::Info => theme.foreground,
                                        Level::Debug => theme.background_control_info,
                                        Level::Trace => theme.background_control_success,
                                    };

                                    log_user_interface.label(
                                        RichText::new(&log_message.message)
                                            .color(color)
                                            .font(theme.font_library.font_noto_sans.font_normal.clone()),
                                    );
                                }
                            });
                    }
                    Err(error) => {
                        log::error!("Failed to acquire output log history read lock: {}", error);
                    }
                }

                let log_context_menu_response = user_interface.interact(log_rectangle, user_interface.id().with("output_log_context_menu"), Sense::click());
                Self::render_output_context_menu(&log_context_menu_response, log_copy_text);

                if command_line_rectangle.height() > 0.0 {
                    if let Some(command_text) = self.draw_command_line(user_interface, command_line_rectangle) {
                        OutputCommandDispatcher::dispatch(&self.app_context, command_text);
                    }
                }
            })
            .response;

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_history_copy_text_joins_messages_with_newlines() {
        let mut log_history = VecDeque::new();
        log_history.push_back(LogEvent {
            message: "first".to_string(),
            level: Level::Info,
        });
        log_history.push_back(LogEvent {
            message: "second".to_string(),
            level: Level::Warn,
        });

        assert_eq!(OutputView::build_log_history_copy_text(&log_history), "first\nsecond");
    }
}
