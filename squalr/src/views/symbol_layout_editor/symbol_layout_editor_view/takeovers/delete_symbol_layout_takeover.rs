use super::super::SymbolLayoutEditorView;
use crate::ui::widgets::controls::groupbox::GroupBox;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::SymbolLayoutEditorViewData;
use eframe::egui::{Key, RichText, Ui};
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;

impl SymbolLayoutEditorView {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_delete_confirmation_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
    ) {
        let usage_count = SymbolLayoutEditorViewData::count_symbol_claim_usages(project_symbol_catalog, layout_id);

        let mut should_cancel_take_over = false;
        let mut should_delete_layout = false;
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);

        if can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            should_delete_layout = true;
        }

        self.render_take_over_panel(
            user_interface,
            "Delete Symbol Layout",
            0.0,
            Self::TAKE_OVER_CONTENT_PADDING_X,
            Self::TAKE_OVER_SECTION_SPACING,
            |_user_interface| {},
            |user_interface| {
                let theme = &self.app_context.theme;
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Confirmation", |user_interface| {
                        user_interface.label(RichText::new(format!("Delete `{}`?", layout_id)).color(theme.foreground));
                        user_interface.add_space(4.0);
                        let (usage_text, usage_text_color) = if usage_count == 0 {
                            (String::from("No existing references will be changed."), theme.foreground_preview)
                        } else {
                            (format!("{} existing references will be changed to raw u8.", usage_count), theme.warning)
                        };
                        user_interface.label(RichText::new(usage_text).color(usage_text_color));
                    })
                    .desired_width(user_interface.available_width()),
                );

                user_interface.add_space(Self::TAKE_OVER_SECTION_SPACING);
                let (delete_response, cancel_response) = self.render_delete_take_over_action_buttons(user_interface);
                if delete_response.clicked() {
                    should_delete_layout = true;
                }
                if cancel_response.clicked() {
                    should_cancel_take_over = true;
                }
            },
        );

        if should_cancel_take_over {
            SymbolLayoutEditorViewData::cancel_take_over_state(self.symbol_layout_editor_view_data.clone());
            return;
        }

        if should_delete_layout {
            self.delete_symbol_layout(project_symbol_catalog, layout_id);
        }
    }
}
