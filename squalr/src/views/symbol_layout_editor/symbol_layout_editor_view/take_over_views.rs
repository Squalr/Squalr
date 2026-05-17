use super::SymbolLayoutEditorView;
use crate::ui::widgets::controls::groupbox::GroupBox;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutDefineFieldReturnState, SymbolLayoutEditDraft, SymbolLayoutEditorViewData, SymbolLayoutFieldEditDraft, SymbolLayoutFieldOffsetMode,
};
use eframe::egui::{Align, Align2, Button as EguiButton, Direction, Key, Layout, Response, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, pos2, vec2};
use epaint::CornerRadius;
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    symbol_layouts::symbol_layout_draft_ops::{SymbolLayoutDraftOps, SymbolLayoutUnassignedSelection},
};
use std::{cell::Cell, collections::BTreeSet};
impl SymbolLayoutEditorView {
    fn render_take_over_action_buttons(
        &self,
        user_interface: &mut Ui,
        accept_label: &str,
        can_accept: bool,
    ) -> (Response, Response) {
        let theme = &self.app_context.theme;
        let button_size = vec2(Self::TAKE_OVER_ACTION_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT);
        let total_button_width = button_size.x * 2.0 + Self::TAKE_OVER_ACTION_BUTTON_SPACING;
        let side_spacing = ((user_interface.available_width() - total_button_width) * 0.5).max(0.0);

        let responses = user_interface
            .horizontal(|user_interface| {
                user_interface.add_space(side_spacing);
                user_interface.spacing_mut().item_spacing.x = Self::TAKE_OVER_ACTION_BUTTON_SPACING;

                let cancel_response = user_interface.add_sized(
                    button_size,
                    EguiButton::new(RichText::new("Cancel").color(theme.foreground))
                        .fill(theme.background_control_secondary)
                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                );

                let accept_button = EguiButton::new(RichText::new(accept_label).color(if can_accept { theme.foreground } else { theme.foreground_preview }))
                    .fill(if can_accept {
                        theme.background_control_primary
                    } else {
                        theme.background_control_secondary
                    })
                    .stroke(Stroke::new(
                        1.0,
                        if can_accept {
                            theme.background_control_primary_dark
                        } else {
                            theme.background_control_secondary_dark
                        },
                    ));
                let accept_response = user_interface
                    .add_enabled_ui(can_accept, |user_interface| user_interface.add_sized(button_size, accept_button))
                    .inner;

                (cancel_response, accept_response)
            })
            .inner;

        user_interface.add_space(Self::TAKE_OVER_BOTTOM_PADDING);

        responses
    }

    fn render_delete_take_over_action_buttons(
        &self,
        user_interface: &mut Ui,
    ) -> (Response, Response) {
        let theme = &self.app_context.theme;
        let button_size = vec2(Self::TAKE_OVER_ACTION_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT);
        let total_button_width = button_size.x * 2.0 + Self::TAKE_OVER_ACTION_BUTTON_SPACING;
        let side_spacing = ((user_interface.available_width() - total_button_width) * 0.5).max(0.0);

        let responses = user_interface
            .horizontal(|user_interface| {
                user_interface.add_space(side_spacing);
                user_interface.spacing_mut().item_spacing.x = Self::TAKE_OVER_ACTION_BUTTON_SPACING;

                let delete_response = user_interface.add_sized(
                    button_size,
                    EguiButton::new(RichText::new("Delete").color(theme.foreground))
                        .fill(theme.background_control_danger)
                        .stroke(Stroke::new(1.0, theme.background_control_danger_dark)),
                );

                let cancel_response = user_interface.add_sized(
                    button_size,
                    EguiButton::new(RichText::new("Cancel").color(theme.foreground))
                        .fill(theme.background_control_secondary)
                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                );

                (delete_response, cancel_response)
            })
            .inner;

        user_interface.add_space(Self::TAKE_OVER_BOTTOM_PADDING);

        responses
    }

    fn render_take_over_panel(
        &self,
        user_interface: &mut Ui,
        title: &str,
        header_action_width: f32,
        content_padding_x: f32,
        body_top_spacing: f32,
        render_header_actions: impl FnOnce(&mut Ui),
        add_contents: impl FnOnce(&mut Ui),
    ) {
        let theme = &self.app_context.theme;
        let (panel_rect, _) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::hover());
        user_interface
            .painter()
            .rect_filled(panel_rect, CornerRadius::ZERO, theme.background_panel);

        let inner_rect = panel_rect.shrink2(vec2(Self::TAKE_OVER_PADDING_X, Self::TAKE_OVER_PADDING_Y));
        let mut panel_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(inner_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        panel_user_interface.set_clip_rect(inner_rect);

        if !title.is_empty() || header_action_width > 0.0 {
            let (header_rect, _) = panel_user_interface.allocate_exact_size(
                vec2(panel_user_interface.available_width().max(1.0), Self::TAKE_OVER_HEADER_HEIGHT),
                Sense::hover(),
            );
            panel_user_interface
                .painter()
                .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);
            let header_inner_rect = header_rect;
            let mut header_user_interface = panel_user_interface.new_child(
                UiBuilder::new()
                    .max_rect(header_inner_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            header_user_interface.set_clip_rect(header_inner_rect);

            if header_action_width > 0.0 {
                header_user_interface.allocate_ui_with_layout(
                    vec2(header_action_width, Self::TAKE_OVER_HEADER_HEIGHT),
                    Layout::left_to_right(Align::Center),
                    |user_interface| {
                        render_header_actions(user_interface);
                    },
                );
            }

            let title_width = (header_user_interface.available_width() - Self::TAKE_OVER_HEADER_TITLE_PADDING_X).max(0.0);
            let (title_rect, _) = header_user_interface.allocate_exact_size(vec2(title_width, Self::TAKE_OVER_HEADER_HEIGHT), Sense::hover());
            header_user_interface.painter().text(
                pos2(title_rect.left() + Self::TAKE_OVER_HEADER_TITLE_PADDING_X, title_rect.center().y),
                Align2::LEFT_CENTER,
                title,
                theme.font_library.font_noto_sans.font_window_title.clone(),
                theme.foreground,
            );
        }

        if body_top_spacing > 0.0 {
            panel_user_interface.add_space(body_top_spacing);
        }
        ScrollArea::vertical()
            .id_salt(format!("symbol_layout_editor_take_over_body_{title}"))
            .auto_shrink([false, false])
            .show(&mut panel_user_interface, |user_interface| {
                let content_width = (user_interface.available_width() - content_padding_x * 2.0).max(0.0);
                user_interface.horizontal(|user_interface| {
                    user_interface.add_space(content_padding_x);
                    user_interface.allocate_ui_with_layout(vec2(content_width, 0.0), Layout::top_down(Align::Min), |user_interface| {
                        add_contents(user_interface);
                    });
                });
            });
    }

    pub(super) fn render_symbol_layout_take_over(
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
        let pending_variant_drafts = self.pending_variant_drafts_for_union(Some(&edited_draft));
        let effective_project_symbol_catalog =
            Self::build_effective_project_symbol_catalog_from_pending_drafts(project_symbol_catalog, &pending_variant_drafts);
        let validation_result = SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(
            &effective_project_symbol_catalog,
            &edited_draft,
            unassigned_split_offsets,
        );
        let pending_variant_validation_result = Self::build_pending_variant_layout_descriptors(project_symbol_catalog, &pending_variant_drafts);
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
                if is_union_layout && self.render_add_entry_button(user_interface, "Add a new union variant.") {
                    should_append_field.set(true);
                }
            },
            |user_interface| {
                if show_layout_name_editor {
                    user_interface.add(
                        GroupBox::new_from_theme(&self.app_context.theme, "Size", |user_interface| {
                            self.render_layout_size_editor(user_interface, &mut edited_draft);
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

                                    self.render_string_value_box(
                                        user_interface,
                                        &mut edited_draft.layout_id,
                                        "module.type",
                                        "symbol_layout_editor_layout_id",
                                        layout_id_width,
                                        Self::FIELD_ROW_HEIGHT,
                                    );
                                    self.render_layout_kind_combo(user_interface, &mut edited_draft.layout_kind, "symbol_layout_editor_layout_kind");
                                });

                                if previous_layout_kind != edited_draft.layout_kind && edited_draft.layout_kind.is_union() {
                                    self.normalize_union_field_drafts(&effective_project_symbol_catalog, &mut edited_draft);
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
                                self.render_field_rows(
                                    user_interface,
                                    &effective_project_symbol_catalog,
                                    &mut edited_draft,
                                    selected_field_index,
                                    selected_field_layout_id,
                                    selected_unassigned_span,
                                );
                                if !is_union_layout {
                                    user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                                    let tail_unassigned_offset = self.resolve_draft_tail_unassigned_offset(&effective_project_symbol_catalog, &edited_draft);
                                    if self.render_centered_add_entry_button(user_interface, "Add a new field entry.", tail_unassigned_offset.is_some()) {
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
                                self.render_layout_size_editor(user_interface, &mut edited_draft);
                                user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                                self.render_field_rows(
                                    user_interface,
                                    &effective_project_symbol_catalog,
                                    &mut edited_draft,
                                    selected_field_index,
                                    selected_field_layout_id,
                                    selected_unassigned_span,
                                );
                                if !is_union_layout {
                                    user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                                    let tail_unassigned_offset = self.resolve_draft_tail_unassigned_offset(&effective_project_symbol_catalog, &edited_draft);
                                    if self.render_centered_add_entry_button(user_interface, "Add a new field entry.", tail_unassigned_offset.is_some()) {
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
            let mut field_draft = self.create_field_draft_for_layout_kind(
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
            self.focus_field_in_struct_viewer(&effective_project_symbol_catalog, &edited_draft, field_index_to_focus);
        }

        if should_cancel_take_over {
            SymbolLayoutEditorViewData::cancel_take_over_state(self.symbol_layout_editor_view_data.clone());
            self.clear_struct_viewer_if_symbol_layout_focused();
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
                    self.clear_struct_viewer_if_symbol_layout_focused();
                    return;
                }
                Err(error) => {
                    log::error!("Failed to apply symbol layout draft: {}.", error);
                }
            }
        }

        SymbolLayoutEditorViewData::update_draft(self.symbol_layout_editor_view_data.clone(), edited_draft);
    }

    pub(super) fn render_define_field_from_unassigned_take_over(
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
        let mut validation_result =
            Self::validate_define_field_draft(project_symbol_catalog, &edited_define_field_draft, span_offset_in_bytes, span_size_in_bytes);
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
                                self.render_string_value_box(
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
                                self.render_string_value_box(
                                    user_interface,
                                    &mut edited_define_field_draft.static_offset_in_bytes,
                                    "0",
                                    "symbol_layout_define_field_offset",
                                    user_interface.available_width(),
                                    Self::TOOLBAR_HEIGHT,
                                );

                                validation_result = Self::validate_define_field_draft(
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
                                    self.render_define_field_container_selector(
                                        user_interface,
                                        &mut edited_define_field_draft.container_edit,
                                        &format!("symbol_layout_define_field_container_{}_{}", layout_id, span_offset_in_bytes),
                                        selector_width,
                                    );

                                    let type_selector_width = user_interface.available_width();
                                    self.render_define_field_type_combo(
                                        user_interface,
                                        project_symbol_catalog,
                                        &mut edited_define_field_draft,
                                        &format!("symbol_layout_define_field_type_{}_{}", layout_id, span_offset_in_bytes),
                                        type_selector_width,
                                    );
                                });

                                validation_result = Self::validate_define_field_draft(
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
            self.focus_unassigned_span_in_struct_viewer(draft, span_offset_in_bytes, span_size_in_bytes);
            return;
        }

        if should_create && validation_result.is_ok() {
            let mut updated_draft = draft.clone();
            let mut new_field_draft = edited_define_field_draft.clone();
            new_field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
            let (field_offset_in_bytes, _field_size_in_bytes) = validation_result.unwrap_or((span_offset_in_bytes, 0));
            new_field_draft.static_offset_in_bytes = format!("0x{:X}", field_offset_in_bytes);
            let field_spans = self
                .resolve_draft_field_spans(project_symbol_catalog, draft)
                .map(|(_layout_size_in_bytes, field_spans)| field_spans)
                .unwrap_or_default();
            let insert_index = SymbolLayoutDraftOps::field_insert_index_for_offset(&field_spans, updated_draft.field_drafts.len(), field_offset_in_bytes);

            updated_draft.field_drafts.insert(insert_index, new_field_draft);
            SymbolLayoutEditorViewData::update_draft(self.symbol_layout_editor_view_data.clone(), updated_draft.clone());
            SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
            SymbolLayoutEditorViewData::select_field(self.symbol_layout_editor_view_data.clone(), insert_index);
            self.focus_field_in_struct_viewer(project_symbol_catalog, &updated_draft, insert_index);
            return;
        }

        SymbolLayoutEditorViewData::replace_define_field_draft(self.symbol_layout_editor_view_data.clone(), edited_define_field_draft);
    }

    pub(super) fn render_delete_confirmation_take_over(
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

    pub(super) fn render_field_delete_confirmation_take_over(
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
                self.focus_field_in_struct_viewer(project_symbol_catalog, &edited_draft, field_index_to_focus);
            } else {
                SymbolLayoutEditorViewData::return_to_open_symbol_layout(self.symbol_layout_editor_view_data.clone(), layout_id.to_string());
            }
        }
    }
}
