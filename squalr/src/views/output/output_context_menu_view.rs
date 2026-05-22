use crate::app_context::AppContext;
use crate::ui::widgets::controls::{
    context_menu::context_menu::{ContextMenu, ContextMenuSizing},
    toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
};
use crate::views::output::output_command_state::OutputCommandState;
use crate::views::output::output_context_menu_state::{OutputContextMenuState, OutputContextMenuTarget};
use eframe::egui::text::CCursorRange;
use eframe::egui::{Id, TextBuffer, Ui, ViewportCommand};
use std::ops::Range;
use std::sync::{Arc, RwLock};

pub struct OutputContextMenuView<'lifetime> {
    app_context: Arc<AppContext>,
    command_state: Arc<RwLock<OutputCommandState>>,
    context_menu_state: Arc<RwLock<OutputContextMenuState>>,
    log_copy_text: &'lifetime str,
    command_line_cursor_range: Option<CCursorRange>,
    command_line_text_edit_id: Option<Id>,
}

impl<'lifetime> OutputContextMenuView<'lifetime> {
    const OUTPUT_LOG_COPY_LABEL: &'static str = "Copy";
    const OUTPUT_LOG_COPY_ID: &'static str = "output_log_ctx_copy";
    const OUTPUT_INPUT_CUT_LABEL: &'static str = "Cut";
    const OUTPUT_INPUT_CUT_ID: &'static str = "output_input_ctx_cut";
    const OUTPUT_INPUT_COPY_LABEL: &'static str = "Copy";
    const OUTPUT_INPUT_COPY_ID: &'static str = "output_input_ctx_copy";
    const OUTPUT_INPUT_PASTE_LABEL: &'static str = "Paste";
    const OUTPUT_INPUT_PASTE_ID: &'static str = "output_input_ctx_paste";

    pub fn new(
        app_context: Arc<AppContext>,
        command_state: Arc<RwLock<OutputCommandState>>,
        context_menu_state: Arc<RwLock<OutputContextMenuState>>,
        log_copy_text: &'lifetime str,
        command_line_cursor_range: Option<CCursorRange>,
        command_line_text_edit_id: Option<Id>,
    ) -> Self {
        Self {
            app_context,
            command_state,
            context_menu_state,
            log_copy_text,
            command_line_cursor_range,
            command_line_text_edit_id,
        }
    }

    pub fn show(
        &self,
        user_interface: &mut Ui,
    ) {
        let context_menu_target = match self.context_menu_state.read() {
            Ok(context_menu_state) => context_menu_state.target(),
            Err(error) => {
                log::error!("Failed to acquire output context menu state read lock: {}", error);
                None
            }
        };

        let Some(context_menu_target) = context_menu_target else {
            return;
        };

        let mut is_open = true;

        match context_menu_target {
            OutputContextMenuTarget::Log { position } => {
                self.render_log_context_menu(user_interface, position, &mut is_open);
            }
            OutputContextMenuTarget::CommandInput { position } => {
                self.render_command_input_context_menu(user_interface, position, &mut is_open);
            }
        }

        if !is_open {
            self.hide_menu();
        }
    }

    fn selected_command_text(
        command_text: &str,
        cursor_range: Option<CCursorRange>,
    ) -> Option<String> {
        let cursor_range = cursor_range?;

        if cursor_range.is_empty() {
            return None;
        }

        Some(cursor_range.slice_str(command_text).to_string())
    }

    fn selected_command_char_range(cursor_range: Option<CCursorRange>) -> Option<Range<usize>> {
        let cursor_range = cursor_range?;

        if cursor_range.is_empty() {
            return None;
        }

        Some(cursor_range.as_sorted_char_range())
    }

    fn hide_menu(&self) {
        match self.context_menu_state.write() {
            Ok(mut context_menu_state) => context_menu_state.hide_menu(),
            Err(error) => log::error!("Failed to acquire output context menu state write lock: {}", error),
        }
    }

    fn render_log_context_menu(
        &self,
        user_interface: &mut Ui,
        position: epaint::Pos2,
        is_open: &mut bool,
    ) {
        let context_menu_width = ContextMenuSizing::width_for_labels(self.app_context.as_ref(), user_interface, [Self::OUTPUT_LOG_COPY_LABEL]);

        ContextMenu::new(
            self.app_context.clone(),
            "output_log_context_menu",
            position,
            |menu_user_interface, should_close| {
                if menu_user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            Self::OUTPUT_LOG_COPY_LABEL,
                            Self::OUTPUT_LOG_COPY_ID,
                            &None,
                            context_menu_width,
                        )
                        .disabled(self.log_copy_text.is_empty()),
                    )
                    .clicked()
                {
                    menu_user_interface
                        .ctx()
                        .copy_text(self.log_copy_text.to_string());
                    *should_close = true;
                }
            },
        )
        .width(context_menu_width)
        .corner_radius(8)
        .show(user_interface, is_open);
    }

    fn render_command_input_context_menu(
        &self,
        user_interface: &mut Ui,
        position: epaint::Pos2,
        is_open: &mut bool,
    ) {
        let context_menu_width = ContextMenuSizing::width_for_labels(
            self.app_context.as_ref(),
            user_interface,
            [
                Self::OUTPUT_INPUT_CUT_LABEL,
                Self::OUTPUT_INPUT_COPY_LABEL,
                Self::OUTPUT_INPUT_PASTE_LABEL,
            ],
        );
        let selected_command_text = match self.command_state.read() {
            Ok(command_state) => Self::selected_command_text(command_state.command_text(), self.command_line_cursor_range),
            Err(error) => {
                log::error!("Failed to acquire output command state read lock: {}", error);
                None
            }
        };
        let has_selected_command_text = selected_command_text.is_some();

        ContextMenu::new(
            self.app_context.clone(),
            "output_command_input_context_menu",
            position,
            |menu_user_interface, should_close| {
                if menu_user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            Self::OUTPUT_INPUT_CUT_LABEL,
                            Self::OUTPUT_INPUT_CUT_ID,
                            &None,
                            context_menu_width,
                        )
                        .disabled(!has_selected_command_text),
                    )
                    .clicked()
                {
                    if let Some(selected_command_text) = selected_command_text.clone() {
                        menu_user_interface.ctx().copy_text(selected_command_text);

                        if let Some(selected_char_range) = Self::selected_command_char_range(self.command_line_cursor_range) {
                            match self.command_state.write() {
                                Ok(mut command_state) => command_state
                                    .command_text_mut()
                                    .delete_char_range(selected_char_range),
                                Err(error) => log::error!("Failed to acquire output command state write lock: {}", error),
                            }
                        }
                    }

                    *should_close = true;
                }

                if menu_user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            Self::OUTPUT_INPUT_COPY_LABEL,
                            Self::OUTPUT_INPUT_COPY_ID,
                            &None,
                            context_menu_width,
                        )
                        .disabled(!has_selected_command_text),
                    )
                    .clicked()
                {
                    if let Some(selected_command_text) = selected_command_text.clone() {
                        menu_user_interface.ctx().copy_text(selected_command_text);
                    }

                    *should_close = true;
                }

                if menu_user_interface
                    .add(ToolbarMenuItemView::new(
                        self.app_context.clone(),
                        Self::OUTPUT_INPUT_PASTE_LABEL,
                        Self::OUTPUT_INPUT_PASTE_ID,
                        &None,
                        context_menu_width,
                    ))
                    .clicked()
                {
                    if let Some(text_edit_id) = self.command_line_text_edit_id {
                        menu_user_interface.memory_mut(|memory| memory.request_focus(text_edit_id));
                    }

                    menu_user_interface
                        .ctx()
                        .send_viewport_cmd(ViewportCommand::RequestPaste);
                    *should_close = true;
                }
            },
        )
        .width(context_menu_width)
        .corner_radius(8)
        .show(user_interface, is_open);
    }
}
