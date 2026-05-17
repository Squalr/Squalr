use super::super::SymbolLayoutEditorView;
use super::super::authoring::symbol_layout_draft_analyzer::SymbolLayoutDraftAnalyzer;
use super::super::authoring::symbol_layout_field_draft_factory::SymbolLayoutFieldDraftFactory;
use super::super::authoring::symbol_layout_variant_session::SymbolLayoutVariantSession;
use super::super::controls::symbol_layout_add_entry_button::{render_symbol_layout_add_entry_button, render_symbol_layout_centered_add_entry_button};
use super::super::controls::symbol_layout_kind_combo::render_symbol_layout_kind_combo;
use super::super::controls::symbol_layout_value_box::{render_symbol_layout_size_editor, render_symbol_layout_string_value_box};
use super::super::details::symbol_layout_details_focus::clear_struct_viewer_if_symbol_layout_focused;
use super::super::rows::symbol_layout_draft_field_tree_view::SymbolLayoutDraftFieldTreeView;
use super::super::rows::symbol_layout_field_row_action::focus_field_in_struct_viewer;
use crate::ui::widgets::controls::groupbox::GroupBox;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutEditDraft, SymbolLayoutEditorViewData, SymbolLayoutFieldOffsetMode,
};
use eframe::egui::{RichText, Ui};
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    symbol_layouts::symbol_layout_draft_ops::{SymbolLayoutDraftOps, SymbolLayoutUnassignedSelection},
};
use std::{cell::Cell, collections::BTreeSet};

impl SymbolLayoutEditorView {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_symbol_layout_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        take_over_title: &str,
        baseline_project_symbol_catalog: Option<&ProjectSymbolCatalog>,
        baseline_draft: Option<&SymbolLayoutEditDraft>,
        draft: Option<&SymbolLayoutEditDraft>,
        unassigned_split_offsets: &BTreeSet<u64>,
        selected_field_index: Option<usize>,
        selected_field_layout_id: Option<&str>,
        selected_unassigned_span: Option<&SymbolLayoutUnassignedSelection>,
        show_layout_name_editor: bool,
    ) {
        let Some(draft) = draft else {
            return;
        };
        let baseline_draft = baseline_draft.unwrap_or(draft);

        let mut edited_draft = draft.clone();
        let pending_variant_drafts =
            SymbolLayoutVariantSession::pending_variant_drafts_for_union(self.symbol_layout_editor_view_data.clone(), Some(&edited_draft));
        let effective_project_symbol_catalog =
            SymbolLayoutVariantSession::build_effective_project_symbol_catalog_from_pending_drafts(project_symbol_catalog, &pending_variant_drafts);
        let validation_result = SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(
            &effective_project_symbol_catalog,
            &edited_draft,
            unassigned_split_offsets,
        );
        let pending_variant_validation_result =
            SymbolLayoutVariantSession::build_pending_variant_layout_descriptors(project_symbol_catalog, &pending_variant_drafts);
        let usage_count = edited_draft
            .original_layout_id
            .as_deref()
            .map(|selected_layout_id| SymbolLayoutEditorViewData::count_symbol_claim_usages(project_symbol_catalog, selected_layout_id))
            .unwrap_or(0);
        let has_unsaved_changes = Self::symbol_layout_take_over_has_unsaved_changes(
            baseline_project_symbol_catalog,
            baseline_draft,
            &edited_draft,
            validation_result.as_ref().ok(),
            unassigned_split_offsets,
        ) || !pending_variant_drafts.is_empty();
        let is_creating_new_layout = edited_draft.original_layout_id.is_none();
        let is_union_layout = edited_draft.layout_kind.is_union();
        let can_save = validation_result.is_ok() && pending_variant_validation_result.is_ok() && has_unsaved_changes;
        let header_action_width = if is_union_layout { Self::ICON_BUTTON_WIDTH } else { 0.0 };
        let mut should_cancel_take_over = false;
        let mut should_save_draft = false;
        let should_append_field = Cell::new(false);
        let append_field_tail_offset = Cell::new(None);

        self.render_take_over_panel(
            user_interface,
            if show_layout_name_editor { take_over_title } else { "" },
            header_action_width,
            if show_layout_name_editor {
                Self::TAKE_OVER_CONTENT_PADDING_X
            } else {
                Self::TAKE_OVER_GROUPBOX_SIDE_PADDING
            },
            Self::TAKE_OVER_SECTION_SPACING,
            |user_interface| {
                if is_union_layout
                    && render_symbol_layout_add_entry_button(
                        self.app_context.clone(),
                        user_interface,
                        "Add a new union variant.",
                        Self::ICON_BUTTON_WIDTH,
                        Self::FIELD_ROW_HEIGHT,
                    )
                {
                    should_append_field.set(true);
                }
            },
            |user_interface| {
                if show_layout_name_editor {
                    user_interface.add(
                        GroupBox::new_from_theme(&self.app_context.theme, "Size", |user_interface| {
                            render_symbol_layout_size_editor(self.app_context.clone(), user_interface, &mut edited_draft, Self::FIELD_ROW_HEIGHT);
                        })
                        .desired_width(user_interface.available_width()),
                    );
                    user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);

                    user_interface.add(
                        GroupBox::new_from_theme(
                            &self.app_context.theme,
                            if is_creating_new_layout { "New Symbol Layout" } else { "Symbol Layout" },
                            |user_interface| {
                                let previous_layout_kind = edited_draft.layout_kind;

                                user_interface.horizontal(|user_interface| {
                                    user_interface.spacing_mut().item_spacing.x = Self::FIELD_INPUT_SPACING;
                                    let combo_width = Self::LAYOUT_KIND_COMBO_WIDTH.min(user_interface.available_width().max(1.0));
                                    let layout_id_width = (user_interface.available_width() - combo_width - Self::FIELD_INPUT_SPACING).max(1.0);

                                    render_symbol_layout_string_value_box(
                                        self.app_context.clone(),
                                        user_interface,
                                        &mut edited_draft.layout_id,
                                        "module.type",
                                        "symbol_layout_editor_layout_id",
                                        layout_id_width,
                                        Self::FIELD_ROW_HEIGHT,
                                    );
                                    render_symbol_layout_kind_combo(
                                        self.app_context.clone(),
                                        user_interface,
                                        &mut edited_draft.layout_kind,
                                        "symbol_layout_editor_layout_kind",
                                        combo_width,
                                        Self::FIELD_ROW_HEIGHT,
                                    );
                                });

                                if previous_layout_kind != edited_draft.layout_kind && edited_draft.layout_kind.is_union() {
                                    SymbolLayoutFieldDraftFactory::normalize_union_field_drafts(&effective_project_symbol_catalog, &mut edited_draft);
                                } else if previous_layout_kind != edited_draft.layout_kind {
                                    SymbolLayoutEditorViewData::clear_pending_variant_drafts_for_take_over(self.symbol_layout_editor_view_data.clone());
                                }
                                user_interface.add_space(6.0);

                                if !is_creating_new_layout {
                                    let status_text = if usage_count == 0 {
                                        String::from("Not used by any symbol claims yet.")
                                    } else if usage_count == 1 {
                                        String::from("Used by 1 symbol claim.")
                                    } else {
                                        format!("Used by {} symbol claims.", usage_count)
                                    };
                                    user_interface.label(RichText::new(status_text).color(self.app_context.theme.foreground_preview));
                                }
                            },
                        )
                        .desired_width(user_interface.available_width()),
                    );
                    user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);

                    user_interface.add(
                        GroupBox::new_from_theme(
                            &self.app_context.theme,
                            if is_union_layout { "Edit Union Variants" } else { "Edit Symbol Layout" },
                            |user_interface| {
                                SymbolLayoutDraftFieldTreeView::new(
                                    self,
                                    &effective_project_symbol_catalog,
                                    &mut edited_draft,
                                    selected_field_index,
                                    selected_field_layout_id,
                                    selected_unassigned_span,
                                )
                                .show(user_interface);
                                if !is_union_layout {
                                    user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                                    let tail_unassigned_offset =
                                        SymbolLayoutDraftAnalyzer::resolve_draft_tail_unassigned_offset(&effective_project_symbol_catalog, &edited_draft);
                                    if render_symbol_layout_centered_add_entry_button(
                                        self.app_context.clone(),
                                        user_interface,
                                        "Add a new field entry.",
                                        tail_unassigned_offset.is_some(),
                                        Self::TAKE_OVER_ACTION_BUTTON_WIDTH,
                                        Self::FIELD_ROW_HEIGHT,
                                        Self::FIELD_ADD_BUTTON_CORNER_RADIUS,
                                    ) {
                                        append_field_tail_offset.set(tail_unassigned_offset);
                                        should_append_field.set(true);
                                    }
                                }
                            },
                        )
                        .desired_width(user_interface.available_width()),
                    );
                    user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                } else {
                    let theme = &self.app_context.theme;
                    user_interface.add(
                        GroupBox::new_from_theme(
                            theme,
                            if is_union_layout { "Edit Union Variants" } else { "Edit Symbol Layout" },
                            |user_interface| {
                                render_symbol_layout_size_editor(self.app_context.clone(), user_interface, &mut edited_draft, Self::FIELD_ROW_HEIGHT);
                                user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                                SymbolLayoutDraftFieldTreeView::new(
                                    self,
                                    &effective_project_symbol_catalog,
                                    &mut edited_draft,
                                    selected_field_index,
                                    selected_field_layout_id,
                                    selected_unassigned_span,
                                )
                                .show(user_interface);
                                if !is_union_layout {
                                    user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                                    let tail_unassigned_offset =
                                        SymbolLayoutDraftAnalyzer::resolve_draft_tail_unassigned_offset(&effective_project_symbol_catalog, &edited_draft);
                                    if render_symbol_layout_centered_add_entry_button(
                                        self.app_context.clone(),
                                        user_interface,
                                        "Add a new field entry.",
                                        tail_unassigned_offset.is_some(),
                                        Self::TAKE_OVER_ACTION_BUTTON_WIDTH,
                                        Self::FIELD_ROW_HEIGHT,
                                        Self::FIELD_ADD_BUTTON_CORNER_RADIUS,
                                    ) {
                                        append_field_tail_offset.set(tail_unassigned_offset);
                                        should_append_field.set(true);
                                    }
                                }
                            },
                        )
                        .desired_width(user_interface.available_width()),
                    );
                    user_interface.add_space(Self::TAKE_OVER_SECTION_SPACING);
                }

                if let Err(validation_error) = validation_result.as_ref() {
                    user_interface.label(RichText::new(validation_error).color(self.app_context.theme.error_red));
                    user_interface.add_space(8.0);
                } else if let Err(validation_error) = pending_variant_validation_result.as_ref() {
                    user_interface.label(RichText::new(validation_error).color(self.app_context.theme.error_red));
                    user_interface.add_space(8.0);
                }

                user_interface.add_space(Self::TAKE_OVER_SECTION_SPACING);
                let (cancel_response, accept_response) = self.render_take_over_action_buttons(user_interface, "Accept", can_save);
                if cancel_response.clicked() {
                    should_cancel_take_over = true;
                }
                if accept_response.clicked() {
                    should_save_draft = true;
                }
            },
        );

        if should_append_field.get() {
            let field_index_to_focus = edited_draft.field_drafts.len();
            let mut field_draft = SymbolLayoutFieldDraftFactory::create_field_draft_for_layout_kind(
                &self.app_context,
                &effective_project_symbol_catalog,
                edited_draft.layout_kind,
                &edited_draft.layout_id,
                field_index_to_focus,
            );
            if let Some(field_offset_in_bytes) = append_field_tail_offset.get() {
                field_draft.field_name = format!("field_{:08X}", field_offset_in_bytes);
                field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
                field_draft.static_offset_in_bytes = field_offset_in_bytes.to_string();
            }
            field_draft.field_name = SymbolLayoutDraftOps::build_unique_field_name(&edited_draft, &field_draft.field_name);
            edited_draft.field_drafts.push(field_draft);
            SymbolLayoutEditorViewData::select_field(self.symbol_layout_editor_view_data.clone(), field_index_to_focus);
            focus_field_in_struct_viewer(self, &effective_project_symbol_catalog, &edited_draft, field_index_to_focus);
        }

        if should_cancel_take_over {
            SymbolLayoutEditorViewData::cancel_take_over_state(self.symbol_layout_editor_view_data.clone());
            clear_struct_viewer_if_symbol_layout_focused(self.struct_viewer_view_data.clone());
            return;
        }

        if should_save_draft {
            match SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(
                &effective_project_symbol_catalog,
                &edited_draft,
                unassigned_split_offsets,
            ) {
                Ok(struct_layout_descriptor) => {
                    if let Ok(pending_variant_descriptors) = pending_variant_validation_result {
                        for (original_layout_id, variant_struct_layout_descriptor) in pending_variant_descriptors {
                            self.persist_symbol_layout_descriptor(original_layout_id, &variant_struct_layout_descriptor);
                        }
                    }
                    self.persist_symbol_layout_descriptor(edited_draft.original_layout_id.clone(), &struct_layout_descriptor);
                    SymbolLayoutEditorViewData::select_symbol_layout(
                        self.symbol_layout_editor_view_data.clone(),
                        Some(edited_draft.layout_id.trim().to_string()),
                    );
                    SymbolLayoutEditorViewData::cancel_take_over_state(self.symbol_layout_editor_view_data.clone());
                    clear_struct_viewer_if_symbol_layout_focused(self.struct_viewer_view_data.clone());
                    return;
                }
                Err(error) => {
                    log::error!("Failed to apply symbol layout draft: {}.", error);
                }
            }
        }

        SymbolLayoutEditorViewData::update_draft(self.symbol_layout_editor_view_data.clone(), edited_draft);
    }
}
