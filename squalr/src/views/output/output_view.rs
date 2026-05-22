use crate::app_context::AppContext;
use crate::ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button};
use crate::views::output::output_command_dispatcher::OutputCommandDispatcher;
use crate::views::output::output_command_state::OutputCommandState;
use eframe::egui::{Align, Key, Layout, Response, RichText, ScrollArea, Sense, TextEdit, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, Rect, Stroke, StrokeKind, Vec2, pos2, vec2};
use log::Level;
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

    fn draw_command_line(
        &self,
        user_interface: &mut Ui,
        command_line_rectangle: Rect,
    ) -> Option<String> {
        let theme = &self.app_context.theme;
        let mut command_to_dispatch = None;
        let prompt_width = 20.0;
        let button_size = vec2(30.0, 26.0);
        let row_padding = vec2(8.0, 4.0);
        let spacing = 6.0;
        let prompt_rectangle = Rect::from_min_size(
            command_line_rectangle.min + row_padding,
            vec2(prompt_width, command_line_rectangle.height() - row_padding.y * 2.0),
        );
        let send_button_rectangle = Rect::from_min_size(
            pos2(
                command_line_rectangle.max.x - row_padding.x - button_size.x,
                command_line_rectangle.center().y - button_size.y * 0.5,
            ),
            button_size,
        );
        let input_rectangle = Rect::from_min_max(
            pos2(prompt_rectangle.max.x + spacing, command_line_rectangle.min.y + row_padding.y),
            pos2(send_button_rectangle.min.x - spacing, command_line_rectangle.max.y - row_padding.y),
        );

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
        user_interface.painter().text(
            prompt_rectangle.center(),
            eframe::egui::Align2::CENTER_CENTER,
            ">",
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        let mut command_line_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(command_line_rectangle)
                .sense(Sense::click()),
        );

        let is_send_enabled = match self.command_state.read() {
            Ok(command_state) => command_state.has_pending_command(),
            Err(error) => {
                log::error!("Failed to acquire output command state read lock: {}", error);
                false
            }
        };
        let send_button = command_line_user_interface.put(
            send_button_rectangle,
            Button::new_from_theme(theme)
                .background_color(epaint::Color32::TRANSPARENT)
                .disabled(!is_send_enabled)
                .with_tooltip_text("Run command."),
        );
        IconDraw::draw(
            &command_line_user_interface,
            send_button.rect,
            &theme.icon_library.icon_handle_navigation_right_arrow,
        );

        match self.command_state.write() {
            Ok(mut command_state) => {
                let text_edit_response = command_line_user_interface.put(
                    input_rectangle,
                    TextEdit::singleline(command_state.command_text_mut())
                        .vertical_align(Align::Center)
                        .font(theme.font_library.font_noto_sans.font_normal.clone())
                        .background_color(theme.background_primary)
                        .text_color(theme.foreground)
                        .frame(true),
                );

                command_line_user_interface.painter().rect_stroke(
                    input_rectangle,
                    CornerRadius::ZERO,
                    Stroke::new(1.0, theme.submenu_border),
                    StrokeKind::Inside,
                );

                if text_edit_response.has_focus() && command_line_user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp)) {
                    command_state.navigate_previous();
                }

                if text_edit_response.has_focus() && command_line_user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown)) {
                    command_state.navigate_next();
                }

                if send_button.clicked()
                    || (text_edit_response.has_focus() && command_line_user_interface.input(|input_state| input_state.key_pressed(Key::Enter)))
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
                let log_rectangle = Rect::from_min_max(
                    inner_rectangle.min,
                    pos2(inner_rectangle.max.x, (inner_rectangle.max.y - command_line_height).max(inner_rectangle.min.y)),
                );
                let command_line_rectangle = Rect::from_min_max(pos2(inner_rectangle.min.x, log_rectangle.max.y), inner_rectangle.max);
                let log_builder = UiBuilder::new()
                    .max_rect(log_rectangle)
                    .layout(Layout::top_down(Align::Min));
                let mut log_user_interface = user_interface.new_child(log_builder);

                match log_history.read() {
                    Ok(log_history) => {
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
