use super::super::SymbolLayoutEditorView;
use super::super::authoring::symbol_layout_draft_analyzer::SymbolLayoutDraftAnalyzer;
use super::super::details::symbol_layout_details_focus::focus_unassigned_span_in_struct_viewer;
use super::super::rows::symbol_layout_field_row_action::focus_field_in_struct_viewer;
use crate::ui::widgets::controls::groupbox::GroupBox;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutDefineFieldReturnState, SymbolLayoutEditDraft, SymbolLayoutEditorViewData,
};
use eframe::egui::{Key, RichText, Ui};
use squalr_engine_api::structures::projects::{project_symbol_catalog::ProjectSymbolCatalog, symbol_layouts::symbol_layout_draft_ops::SymbolLayoutDraftOps};

impl SymbolLayoutEditorView {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_field_unassign_confirmation_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        field_index: usize,
        return_state: &SymbolLayoutDefineFieldReturnState,
        draft: Option<&SymbolLayoutEditDraft>,
    ) {
        let Some(draft) = draft else {
            SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
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
        let is_union_layout = draft.layout_kind.is_union();
        let action_label = if is_union_layout { "Remove" } else { "Unassign" };
        let title = if is_union_layout { "Remove Variant" } else { "Unassign Field" };
        let prompt = if is_union_layout {
            format!("Remove `{}`?", field_label)
        } else {
            format!("Unassign `{}`?", field_label)
        };

        let mut should_cancel_unassign = false;
        let mut should_unassign_field = false;
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);

        if can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            should_unassign_field = true;
        }

        self.render_take_over_panel(
            user_interface,
            title,
            0.0,
            Self::TAKE_OVER_CONTENT_PADDING_X,
            Self::TAKE_OVER_SECTION_SPACING,
            |_user_interface| {},
            |user_interface| {
                let theme = &self.app_context.theme;
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Confirmation", |user_interface| {
                        user_interface.label(RichText::new(prompt).color(theme.foreground));
                    })
                    .desired_width(user_interface.available_width()),
                );

                user_interface.add_space(Self::TAKE_OVER_SECTION_SPACING);
                let (unassign_response, cancel_response) = self.render_danger_take_over_action_buttons(user_interface, action_label);
                if unassign_response.clicked() {
                    should_unassign_field = true;
                }
                if cancel_response.clicked() {
                    should_cancel_unassign = true;
                }
            },
        );

        if should_cancel_unassign {
            SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
            return;
        }

        if should_unassign_field {
            let unassigned_field_span = (!draft.layout_kind.is_union())
                .then(|| {
                    SymbolLayoutDraftAnalyzer::resolve_draft_field_spans(project_symbol_catalog, draft, |data_type_ref| {
                        self.resolve_data_type_size_in_bytes(data_type_ref)
                    })
                })
                .flatten()
                .and_then(|(layout_size_in_bytes, field_spans)| {
                    SymbolLayoutDraftOps::resolve_field_span_by_position(&field_spans, field_index).map(|field_span| (layout_size_in_bytes, field_span))
                });
            let mut edited_draft = draft.clone();
            if SymbolLayoutEditorViewData::unassign_field_from_draft(&mut edited_draft, field_index) {
                SymbolLayoutEditorViewData::update_draft(self.symbol_layout_editor_view_data.clone(), edited_draft.clone());
                SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
                if let Some((layout_size_in_bytes, field_span)) = unassigned_field_span {
                    for split_offset_in_bytes in SymbolLayoutDraftOps::split_offsets_to_preserve_unassigned_field(field_span, layout_size_in_bytes) {
                        SymbolLayoutEditorViewData::insert_unassigned_split_offset_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            None,
                            split_offset_in_bytes,
                        );
                    }
                    SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                        self.symbol_layout_editor_view_data.clone(),
                        None,
                        field_span.offset_in_bytes,
                        field_span.size_in_bytes,
                    );
                    focus_unassigned_span_in_struct_viewer(
                        self.app_context.clone(),
                        self.struct_viewer_view_data.clone(),
                        &edited_draft,
                        field_span.offset_in_bytes,
                        field_span.size_in_bytes,
                    );
                } else if !edited_draft.field_drafts.is_empty() {
                    let field_index_to_focus = field_index.min(edited_draft.field_drafts.len().saturating_sub(1));
                    SymbolLayoutEditorViewData::select_field(self.symbol_layout_editor_view_data.clone(), field_index_to_focus);
                    focus_field_in_struct_viewer(self, project_symbol_catalog, &edited_draft, field_index_to_focus);
                } else {
                    SymbolLayoutEditorViewData::clear_field_selection(self.symbol_layout_editor_view_data.clone());
                }
            } else {
                SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
            }
        }
    }
}
