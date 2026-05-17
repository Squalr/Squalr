use super::super::SymbolLayoutEditorView;
use super::super::authoring::symbol_layout_draft_analyzer::SymbolLayoutDraftAnalyzer;
use super::super::controls::symbol_layout_define_field_container_selector::render_define_field_container_selector;
use super::super::controls::symbol_layout_define_field_type_combo::render_define_field_type_combo;
use super::super::controls::symbol_layout_value_box::render_symbol_layout_string_value_box;
use super::super::details::symbol_layout_details_focus::focus_unassigned_span_in_struct_viewer;
use super::super::rows::symbol_layout_field_row_action::focus_field_in_struct_viewer;
use crate::ui::widgets::controls::groupbox::GroupBox;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutDefineFieldReturnState, SymbolLayoutEditDraft, SymbolLayoutEditorViewData, SymbolLayoutFieldEditDraft, SymbolLayoutFieldOffsetMode,
};
use eframe::egui::{Align, Button as EguiButton, Direction, Layout, RichText, Stroke, Ui, vec2};
use squalr_engine_api::structures::projects::{project_symbol_catalog::ProjectSymbolCatalog, symbol_layouts::symbol_layout_draft_ops::SymbolLayoutDraftOps};

impl SymbolLayoutEditorView {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_define_field_from_unassigned_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
        span_offset_in_bytes: u64,
        span_size_in_bytes: u64,
        return_state: &SymbolLayoutDefineFieldReturnState,
        draft: Option<&SymbolLayoutEditDraft>,
        define_field_draft: Option<&SymbolLayoutFieldEditDraft>,
    ) {
        let Some(draft) = draft else {
            return;
        };
        let Some(define_field_draft) = define_field_draft else {
            return;
        };
        let theme = &self.app_context.theme;
        let mut edited_define_field_draft = define_field_draft.clone();
        let mut validation_result = SymbolLayoutDraftAnalyzer::validate_define_field_draft(
            project_symbol_catalog,
            &edited_define_field_draft,
            span_offset_in_bytes,
            span_size_in_bytes,
        );
        let mut should_cancel = false;
        let mut should_create = false;

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                let panel_width = user_interface.available_width();

                user_interface.add(
                    GroupBox::new_from_theme(theme, "Define Field", |user_interface| {
                        user_interface.horizontal(|user_interface| {
                            user_interface.add_space(Self::DEFINE_FIELD_GROUPBOX_SIDE_PADDING);
                            let content_width = (user_interface.available_width() - Self::DEFINE_FIELD_GROUPBOX_SIDE_PADDING).max(1.0);
                            user_interface.allocate_ui_with_layout(vec2(content_width, 0.0), Layout::top_down(Align::Min), |user_interface| {
                                user_interface.label(RichText::new(format!("{} + 0x{:X}", layout_id, span_offset_in_bytes)).color(theme.foreground_preview));
                                user_interface.add_space(8.0);

                                user_interface.label(RichText::new("Name").color(theme.foreground));
                                user_interface.add_space(2.0);
                                render_symbol_layout_string_value_box(
                                    self.app_context.clone(),
                                    user_interface,
                                    &mut edited_define_field_draft.field_name,
                                    "field_name",
                                    "symbol_layout_define_field_name",
                                    user_interface.available_width(),
                                    Self::TOOLBAR_HEIGHT,
                                );
                                user_interface.add_space(8.0);

                                let max_relative_offset = span_size_in_bytes.saturating_sub(1);
                                user_interface.label(RichText::new(format!("Offset in UNASSIGNED (0 to {})", max_relative_offset)).color(theme.foreground));
                                user_interface.add_space(2.0);
                                render_symbol_layout_string_value_box(
                                    self.app_context.clone(),
                                    user_interface,
                                    &mut edited_define_field_draft.static_offset_in_bytes,
                                    "0",
                                    "symbol_layout_define_field_offset",
                                    user_interface.available_width(),
                                    Self::TOOLBAR_HEIGHT,
                                );

                                validation_result = SymbolLayoutDraftAnalyzer::validate_define_field_draft(
                                    project_symbol_catalog,
                                    &edited_define_field_draft,
                                    span_offset_in_bytes,
                                    span_size_in_bytes,
                                );
                                if let Err(validation_error) = validation_result.as_ref()
                                    && validation_error != "Field name is required."
                                {
                                    user_interface.add_space(4.0);
                                    user_interface.label(RichText::new(validation_error).color(theme.warning));
                                }
                                user_interface.add_space(8.0);

                                user_interface.horizontal(|user_interface| {
                                    user_interface.spacing_mut().item_spacing.x = 4.0;
                                    let selector_width = Self::DEFINE_FIELD_CONTAINER_SELECTOR_WIDTH.min(user_interface.available_width());
                                    render_define_field_container_selector(
                                        self.app_context.clone(),
                                        user_interface,
                                        &mut edited_define_field_draft.container_edit,
                                        &format!("symbol_layout_define_field_container_{}_{}", layout_id, span_offset_in_bytes),
                                        selector_width,
                                        Self::TOOLBAR_HEIGHT,
                                    );

                                    let type_selector_width = user_interface.available_width();
                                    render_define_field_type_combo(
                                        self.app_context.clone(),
                                        user_interface,
                                        project_symbol_catalog,
                                        &mut edited_define_field_draft,
                                        &format!("symbol_layout_define_field_type_{}_{}", layout_id, span_offset_in_bytes),
                                        type_selector_width,
                                        Self::TOOLBAR_HEIGHT,
                                    );
                                });

                                validation_result = SymbolLayoutDraftAnalyzer::validate_define_field_draft(
                                    project_symbol_catalog,
                                    &edited_define_field_draft,
                                    span_offset_in_bytes,
                                    span_size_in_bytes,
                                );

                                if let Err(validation_error) = validation_result.as_ref()
                                    && validation_error == "Field name is required."
                                {
                                    user_interface.add_space(6.0);
                                    user_interface.label(RichText::new(validation_error).color(theme.error_red));
                                }

                                user_interface.add_space(12.0);
                                user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
                                    let button_size = vec2(Self::TAKE_OVER_ACTION_BUTTON_WIDTH, Self::TOOLBAR_HEIGHT);
                                    let button_spacing = Self::TAKE_OVER_ACTION_BUTTON_SPACING;
                                    let total_button_row_width = button_size.x * 2.0 + button_spacing;
                                    let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

                                    user_interface.horizontal(|user_interface| {
                                        user_interface.add_space(side_spacing);
                                        user_interface.spacing_mut().item_spacing.x = button_spacing;

                                        let cancel_response = user_interface.add_sized(
                                            button_size,
                                            EguiButton::new(RichText::new("Cancel").color(theme.foreground))
                                                .fill(theme.background_control_secondary)
                                                .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                                        );
                                        if cancel_response.clicked() {
                                            should_cancel = true;
                                        }

                                        let can_create = validation_result.is_ok();
                                        let create_fill = if can_create {
                                            theme.background_control_primary
                                        } else {
                                            theme.background_control_secondary
                                        };
                                        let create_stroke = if can_create {
                                            theme.background_control_primary_dark
                                        } else {
                                            theme.background_control_secondary_dark
                                        };
                                        let create_response = user_interface.add_sized(
                                            button_size,
                                            EguiButton::new(RichText::new("Create").color(if can_create {
                                                theme.foreground
                                            } else {
                                                theme.foreground_preview
                                            }))
                                            .fill(create_fill)
                                            .stroke(Stroke::new(1.0, create_stroke)),
                                        );
                                        if can_create && create_response.clicked() {
                                            should_create = true;
                                        }
                                    });
                                });
                            });
                        });
                    })
                    .desired_width(panel_width),
                );
            },
        );

        if should_cancel {
            SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
            focus_unassigned_span_in_struct_viewer(
                self.app_context.clone(),
                self.struct_viewer_view_data.clone(),
                draft,
                span_offset_in_bytes,
                span_size_in_bytes,
            );
            return;
        }

        if should_create && validation_result.is_ok() {
            let mut updated_draft = draft.clone();
            let mut new_field_draft = edited_define_field_draft.clone();
            new_field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
            let (field_offset_in_bytes, _field_size_in_bytes) = validation_result.unwrap_or((span_offset_in_bytes, 0));
            new_field_draft.static_offset_in_bytes = format!("0x{:X}", field_offset_in_bytes);
            let field_spans = SymbolLayoutDraftAnalyzer::resolve_draft_field_spans(project_symbol_catalog, draft)
                .map(|(_layout_size_in_bytes, field_spans)| field_spans)
                .unwrap_or_default();
            let insert_index = SymbolLayoutDraftOps::field_insert_index_for_offset(&field_spans, updated_draft.field_drafts.len(), field_offset_in_bytes);

            updated_draft.field_drafts.insert(insert_index, new_field_draft);
            SymbolLayoutEditorViewData::update_draft(self.symbol_layout_editor_view_data.clone(), updated_draft.clone());
            SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
            SymbolLayoutEditorViewData::select_field(self.symbol_layout_editor_view_data.clone(), insert_index);
            focus_field_in_struct_viewer(self, project_symbol_catalog, &updated_draft, insert_index);
            return;
        }

        SymbolLayoutEditorViewData::replace_define_field_draft(self.symbol_layout_editor_view_data.clone(), edited_define_field_draft);
    }
}
