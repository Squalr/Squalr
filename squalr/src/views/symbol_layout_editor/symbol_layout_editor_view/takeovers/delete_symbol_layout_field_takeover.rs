use super::super::SymbolLayoutEditorView;
use super::super::rows::symbol_layout_field_row_action::focus_field_in_struct_viewer;
use crate::ui::widgets::controls::groupbox::GroupBox;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{SymbolLayoutEditDraft, SymbolLayoutEditorViewData};
use eframe::egui::{Key, RichText, Ui};
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;

impl SymbolLayoutEditorView {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_field_delete_confirmation_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
        field_index: usize,
        draft: Option<&SymbolLayoutEditDraft>,
    ) {
        let Some(draft) = draft else {
            SymbolLayoutEditorViewData::return_to_open_symbol_layout(self.symbol_layout_editor_view_data.clone(), layout_id.to_string());
            return;
        };

        let field_label = draft
            .field_drafts
            .get(field_index)
            .map(|field_draft| {
                if field_draft.field_name.trim().is_empty() {
                    String::from("<unnamed>")
                } else {
                    field_draft.field_name.trim().to_string()
                }
            })
            .unwrap_or_else(|| String::from("<unnamed>"));

        let mut should_cancel_delete = false;
        let mut should_delete_field = false;
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);

        if can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            should_delete_field = true;
        }

        self.render_take_over_panel(
            user_interface,
            "Delete Struct Entry",
            0.0,
            Self::TAKE_OVER_CONTENT_PADDING_X,
            Self::TAKE_OVER_SECTION_SPACING,
            |_user_interface| {},
            |user_interface| {
                let theme = &self.app_context.theme;
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Confirmation", |user_interface| {
                        user_interface.label(RichText::new(format!("Delete `{}`?", field_label)).color(theme.foreground));
                    })
                    .desired_width(user_interface.available_width()),
                );

                user_interface.add_space(Self::TAKE_OVER_SECTION_SPACING);
                let (delete_response, cancel_response) = self.render_delete_take_over_action_buttons(user_interface);
                if delete_response.clicked() {
                    should_delete_field = true;
                }
                if cancel_response.clicked() {
                    should_cancel_delete = true;
                }
            },
        );

        if should_cancel_delete {
            SymbolLayoutEditorViewData::return_to_open_symbol_layout(self.symbol_layout_editor_view_data.clone(), layout_id.to_string());
            return;
        }

        if should_delete_field {
            let mut edited_draft = draft.clone();
            if let Some(field_index_to_focus) =
                SymbolLayoutEditorViewData::remove_field_from_draft(&mut edited_draft, field_index, self.default_data_type_ref())
            {
                SymbolLayoutEditorViewData::update_draft(self.symbol_layout_editor_view_data.clone(), edited_draft.clone());
                SymbolLayoutEditorViewData::return_to_open_symbol_layout(self.symbol_layout_editor_view_data.clone(), layout_id.to_string());
                SymbolLayoutEditorViewData::select_field(self.symbol_layout_editor_view_data.clone(), field_index_to_focus);
                focus_field_in_struct_viewer(self, project_symbol_catalog, &edited_draft, field_index_to_focus);
            } else {
                SymbolLayoutEditorViewData::return_to_open_symbol_layout(self.symbol_layout_editor_view_data.clone(), layout_id.to_string());
            }
        }
    }
}
