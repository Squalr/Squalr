use super::super::SymbolLayoutEditorView;
use super::super::authoring::symbol_layout_draft_analyzer::SymbolLayoutDraftAnalyzer;
use super::super::authoring::symbol_layout_variant_session::SymbolLayoutVariantSession;
use super::symbol_layout_field_context_menu::render_field_context_menu;
use super::symbol_layout_field_row_action::SymbolLayoutFieldRowAction;
use super::symbol_layout_field_row_view::SymbolLayoutFieldRowView;
use super::symbol_layout_unassigned_context_menu::render_unassigned_context_menu;
use super::symbol_layout_unassigned_row_action::{SymbolLayoutUnassignedRowAction, apply_unassigned_row_action};
use super::symbol_layout_unassigned_row_view::SymbolLayoutUnassignedRowView;
use super::union_variant_preview_row_view::UnionVariantPreviewRowView;
use crate::views::symbol_layout_editor::symbol_layout_editor_view::controls::symbol_layout_add_entry_button::render_symbol_layout_centered_add_entry_button;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{SymbolLayoutEditDraft, SymbolLayoutFieldEditDraft};
use eframe::egui::{Align, Layout, Ui, vec2};
use squalr_engine_api::structures::{
    projects::{
        project_symbol_catalog::ProjectSymbolCatalog,
        symbol_layouts::symbol_layout_draft_ops::{
            SymbolLayoutDraftOps, SymbolLayoutFieldSpan, SymbolLayoutUnassignedAdjacentField, SymbolLayoutUnassignedRowContext, SymbolLayoutUnassignedSelection,
        },
    },
    structs::symbolic_struct_definition::SymbolicLayoutKind,
};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
enum SymbolLayoutVariantLayoutRowAction {
    Field {
        variant_layout_id: String,
        field_index: usize,
        field_row_action: SymbolLayoutFieldRowAction,
    },
    Unassigned {
        variant_layout_id: String,
        row_context: SymbolLayoutUnassignedRowContext,
        row_action: SymbolLayoutUnassignedRowAction,
    },
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct SymbolLayoutDraftFieldTreeView<'view, 'draft> {
    symbol_layout_editor_view: &'view SymbolLayoutEditorView,
    project_symbol_catalog: &'view ProjectSymbolCatalog,
    draft: &'draft mut SymbolLayoutEditDraft,
    selected_field_index: Option<usize>,
    selected_field_layout_id: Option<&'view str>,
    selected_unassigned_span: Option<&'view SymbolLayoutUnassignedSelection>,
}

impl<'view, 'draft> SymbolLayoutDraftFieldTreeView<'view, 'draft> {
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn new(
        symbol_layout_editor_view: &'view SymbolLayoutEditorView,
        project_symbol_catalog: &'view ProjectSymbolCatalog,
        draft: &'draft mut SymbolLayoutEditDraft,
        selected_field_index: Option<usize>,
        selected_field_layout_id: Option<&'view str>,
        selected_unassigned_span: Option<&'view SymbolLayoutUnassignedSelection>,
    ) -> Self {
        Self {
            symbol_layout_editor_view,
            project_symbol_catalog,
            draft,
            selected_field_index,
            selected_field_layout_id,
            selected_unassigned_span,
        }
    }

    fn render_union_variant_child_row<R>(
        user_interface: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        user_interface
            .horizontal(|user_interface| {
                user_interface.spacing_mut().item_spacing.x = 0.0;
                user_interface.add_space(SymbolLayoutEditorView::UNION_VARIANT_CHILD_INDENT);
                user_interface
                    .allocate_ui_with_layout(vec2(user_interface.available_width().max(1.0), 0.0), Layout::top_down(Align::Min), add_contents)
                    .inner
            })
            .inner
    }

    fn render_union_variant_layout_rows(
        &self,
        user_interface: &mut Ui,
        union_draft: &SymbolLayoutEditDraft,
        variant_index: usize,
        variant_field_draft: &SymbolLayoutFieldEditDraft,
    ) -> Option<SymbolLayoutVariantLayoutRowAction> {
        let mut variant_draft = SymbolLayoutVariantSession::create_union_variant_layout_draft_with_pending(
            self.project_symbol_catalog,
            self.symbol_layout_editor_view
                .symbol_layout_editor_view_data
                .clone(),
            union_draft,
            variant_index,
            variant_field_draft,
            |data_type_ref| {
                self.symbol_layout_editor_view
                    .resolve_data_type_size_in_bytes(data_type_ref)
            },
        );
        let variant_layout_id = variant_draft.layout_id.clone();

        let Some((layout_size_in_bytes, mut field_spans)) =
            SymbolLayoutDraftAnalyzer::resolve_draft_field_spans(self.project_symbol_catalog, &variant_draft, |data_type_ref| {
                self.symbol_layout_editor_view
                    .resolve_data_type_size_in_bytes(data_type_ref)
            })
        else {
            Self::render_union_variant_child_row(user_interface, |user_interface| {
                UnionVariantPreviewRowView::new(self.symbol_layout_editor_view.app_context.clone(), "UNASSIGNED", "variant layout unresolved")
                    .show(user_interface);
            });
            return None;
        };
        let unassigned_split_offsets = self
            .symbol_layout_editor_view
            .symbol_layout_editor_view_data
            .read("SymbolLayoutEditor variant unassigned split offsets")
            .map(|symbol_layout_editor_view_data| symbol_layout_editor_view_data.get_unassigned_split_offsets_for_layout(Some(variant_layout_id.as_str())))
            .unwrap_or_default();
        let mut pending_variant_layout_action = None;
        let mut next_visible_offset = 0_u64;
        let mut previous_visible_field = None;

        field_spans.sort_by_key(|field_span| (field_span.offset_in_bytes, field_span.field_position));

        for field_span in field_spans.iter().copied() {
            if field_span.offset_in_bytes > next_visible_offset {
                let unassigned_size = field_span.offset_in_bytes.saturating_sub(next_visible_offset);
                let move_down_field = Some(SymbolLayoutUnassignedAdjacentField {
                    field_position: field_span.field_position,
                    offset_in_bytes: field_span.offset_in_bytes,
                    size_in_bytes: field_span.size_in_bytes,
                });
                for unassigned_row_context in SymbolLayoutDraftOps::build_unassigned_row_contexts(
                    next_visible_offset,
                    unassigned_size,
                    &unassigned_split_offsets,
                    previous_visible_field,
                    move_down_field,
                ) {
                    let is_selected = self
                        .selected_unassigned_span
                        .is_some_and(|selected_unassigned_span| {
                            selected_unassigned_span.matches(
                                Some(variant_layout_id.as_str()),
                                unassigned_row_context.offset_in_bytes,
                                unassigned_row_context.size_in_bytes,
                            )
                        });
                    let unassigned_row_action = Self::render_union_variant_child_row(user_interface, |user_interface| {
                        SymbolLayoutUnassignedRowView::new(
                            self.symbol_layout_editor_view.app_context.clone(),
                            self.symbol_layout_editor_view
                                .symbol_layout_editor_view_data
                                .clone(),
                            Some(variant_layout_id.as_str()),
                            &unassigned_row_context,
                            true,
                            false,
                            is_selected,
                        )
                        .show(user_interface)
                    });
                    if let Some(unassigned_row_action) = unassigned_row_action {
                        pending_variant_layout_action = Some(SymbolLayoutVariantLayoutRowAction::Unassigned {
                            variant_layout_id: variant_layout_id.clone(),
                            row_context: unassigned_row_context,
                            row_action: unassigned_row_action,
                        });
                    }
                }
            }

            let can_move_up = SymbolLayoutDraftOps::can_move_struct_field_up(&field_spans, &unassigned_split_offsets, field_span.field_position);
            let can_move_down =
                SymbolLayoutDraftOps::can_move_struct_field_down(&field_spans, layout_size_in_bytes, &unassigned_split_offsets, field_span.field_position);
            let is_selected = self.selected_field_layout_id == Some(variant_layout_id.as_str()) && self.selected_field_index == Some(field_span.field_position);
            if let Some(field_draft) = variant_draft.field_drafts.get_mut(field_span.field_position) {
                let field_row_action = Self::render_union_variant_child_row(user_interface, |user_interface| {
                    SymbolLayoutFieldRowView::new(
                        self.symbol_layout_editor_view.app_context.clone(),
                        self.symbol_layout_editor_view
                            .symbol_layout_editor_view_data
                            .clone(),
                        self.project_symbol_catalog,
                        SymbolicLayoutKind::Struct,
                        field_draft,
                        field_span.field_position,
                        is_selected,
                        can_move_up,
                        can_move_down,
                        Some(variant_layout_id.as_str()),
                        true,
                    )
                    .show(user_interface)
                });
                if let Some(field_row_action) = field_row_action {
                    pending_variant_layout_action = Some(SymbolLayoutVariantLayoutRowAction::Field {
                        variant_layout_id: variant_layout_id.clone(),
                        field_index: field_span.field_position,
                        field_row_action,
                    });
                }
            }

            next_visible_offset = next_visible_offset.max(
                field_span
                    .offset_in_bytes
                    .saturating_add(field_span.size_in_bytes),
            );
            previous_visible_field = Some(SymbolLayoutUnassignedAdjacentField {
                field_position: field_span.field_position,
                offset_in_bytes: field_span.offset_in_bytes,
                size_in_bytes: field_span.size_in_bytes,
            });
        }

        if layout_size_in_bytes > next_visible_offset {
            let unassigned_size = layout_size_in_bytes.saturating_sub(next_visible_offset);
            for unassigned_row_context in SymbolLayoutDraftOps::build_unassigned_row_contexts(
                next_visible_offset,
                unassigned_size,
                &unassigned_split_offsets,
                previous_visible_field,
                None,
            ) {
                let is_selected = self
                    .selected_unassigned_span
                    .is_some_and(|selected_unassigned_span| {
                        selected_unassigned_span.matches(
                            Some(variant_layout_id.as_str()),
                            unassigned_row_context.offset_in_bytes,
                            unassigned_row_context.size_in_bytes,
                        )
                    });
                let unassigned_row_action = Self::render_union_variant_child_row(user_interface, |user_interface| {
                    SymbolLayoutUnassignedRowView::new(
                        self.symbol_layout_editor_view.app_context.clone(),
                        self.symbol_layout_editor_view
                            .symbol_layout_editor_view_data
                            .clone(),
                        Some(variant_layout_id.as_str()),
                        &unassigned_row_context,
                        true,
                        false,
                        is_selected,
                    )
                    .show(user_interface)
                });
                if let Some(unassigned_row_action) = unassigned_row_action {
                    pending_variant_layout_action = Some(SymbolLayoutVariantLayoutRowAction::Unassigned {
                        variant_layout_id: variant_layout_id.clone(),
                        row_context: unassigned_row_context,
                        row_action: unassigned_row_action,
                    });
                }
            }
        }

        let field_context_menu_target = self
            .symbol_layout_editor_view
            .symbol_layout_editor_view_data
            .read("SymbolLayoutEditor variant field context menu")
            .and_then(|symbol_layout_editor_view_data| {
                symbol_layout_editor_view_data
                    .get_field_context_menu_target()
                    .cloned()
            });

        if let Some(field_context_menu_target) = field_context_menu_target
            && field_context_menu_target.get_layout_id() == Some(variant_layout_id.as_str())
            && field_context_menu_target.get_field_index() < variant_draft.field_drafts.len()
            && let Some(field_row_action) = render_field_context_menu(
                self.symbol_layout_editor_view,
                user_interface,
                SymbolicLayoutKind::Struct,
                &field_context_menu_target,
                variant_draft.field_drafts.len(),
                true,
            )
        {
            pending_variant_layout_action = Some(SymbolLayoutVariantLayoutRowAction::Field {
                variant_layout_id: variant_layout_id.clone(),
                field_index: field_context_menu_target.get_field_index(),
                field_row_action,
            });
        }

        pending_variant_layout_action
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn show(
        self,
        user_interface: &mut Ui,
    ) {
        let field_count = self.draft.field_drafts.len();
        let layout_kind = self.draft.layout_kind;
        let mut pending_field_row_action = None;
        let mut pending_variant_field_row_action = None;
        let mut pending_unassigned_row_action: Option<(Option<String>, SymbolLayoutUnassignedRowContext, SymbolLayoutUnassignedRowAction)> = None;
        let field_spans = SymbolLayoutDraftAnalyzer::resolve_draft_field_spans(self.project_symbol_catalog, self.draft, |data_type_ref| {
            self.symbol_layout_editor_view
                .resolve_data_type_size_in_bytes(data_type_ref)
        });
        let field_spans_by_position = field_spans
            .as_ref()
            .map(|(_layout_size_in_bytes, field_spans)| {
                field_spans
                    .iter()
                    .map(|field_span| (field_span.field_position, *field_span))
                    .collect::<HashMap<usize, SymbolLayoutFieldSpan>>()
            })
            .unwrap_or_default();
        let mut field_render_indices = (0..field_count).collect::<Vec<_>>();
        if !layout_kind.is_union() && !field_spans_by_position.is_empty() {
            field_render_indices.sort_by_key(|field_index| {
                field_spans_by_position
                    .get(field_index)
                    .map(|field_span| (field_span.offset_in_bytes, field_span.field_position))
                    .unwrap_or((u64::MAX, *field_index))
            });
        }
        let unassigned_split_offsets = self
            .symbol_layout_editor_view
            .symbol_layout_editor_view_data
            .read("SymbolLayoutEditor unassigned split offsets")
            .map(|symbol_layout_editor_view_data| {
                symbol_layout_editor_view_data
                    .get_unassigned_split_offsets()
                    .clone()
            })
            .unwrap_or_default();
        let mut next_visible_offset = 0_u64;
        let mut previous_visible_field = None;

        if layout_kind.is_union() {
            for field_index in 0..field_count {
                let union_draft_preview = self.draft.clone();
                let Some(field_draft) = self.draft.field_drafts.get_mut(field_index) else {
                    continue;
                };

                if let Some(field_row_action) = SymbolLayoutFieldRowView::new(
                    self.symbol_layout_editor_view.app_context.clone(),
                    self.symbol_layout_editor_view
                        .symbol_layout_editor_view_data
                        .clone(),
                    self.project_symbol_catalog,
                    layout_kind,
                    field_draft,
                    field_index,
                    self.selected_field_layout_id.is_none() && self.selected_field_index == Some(field_index),
                    field_index > 0,
                    field_index + 1 < field_count,
                    None,
                    true,
                )
                .show(user_interface)
                {
                    pending_field_row_action = Some((field_index, field_row_action));
                }

                let variant_field_preview_draft = field_draft.clone();
                if let Some(variant_layout_action) =
                    self.render_union_variant_layout_rows(user_interface, &union_draft_preview, field_index, &variant_field_preview_draft)
                {
                    match variant_layout_action {
                        SymbolLayoutVariantLayoutRowAction::Field {
                            variant_layout_id,
                            field_index,
                            field_row_action,
                        } => {
                            pending_variant_field_row_action = Some((variant_layout_id, field_index, field_row_action));
                        }
                        SymbolLayoutVariantLayoutRowAction::Unassigned {
                            variant_layout_id,
                            row_context,
                            row_action,
                        } => {
                            pending_unassigned_row_action = Some((Some(variant_layout_id), row_context, row_action));
                        }
                    }
                }

                let variant_tail_unassigned_offset = SymbolLayoutDraftAnalyzer::resolve_variant_tail_unassigned_offset(
                    self.project_symbol_catalog,
                    self.symbol_layout_editor_view
                        .symbol_layout_editor_view_data
                        .clone(),
                    &union_draft_preview,
                    field_index,
                    &variant_field_preview_draft,
                    |data_type_ref| {
                        self.symbol_layout_editor_view
                            .resolve_data_type_size_in_bytes(data_type_ref)
                    },
                );
                if Self::render_union_variant_child_row(user_interface, |user_interface| {
                    render_symbol_layout_centered_add_entry_button(
                        self.symbol_layout_editor_view.app_context.clone(),
                        user_interface,
                        "Add a new field to this union variant.",
                        variant_tail_unassigned_offset.is_some(),
                        SymbolLayoutEditorView::TAKE_OVER_ACTION_BUTTON_WIDTH,
                        SymbolLayoutEditorView::FIELD_ROW_HEIGHT,
                        SymbolLayoutEditorView::FIELD_ADD_BUTTON_CORNER_RADIUS,
                    )
                }) {
                    pending_field_row_action = Some((field_index, SymbolLayoutFieldRowAction::InsertFieldIntoVariant));
                }

                if field_index + 1 < field_count {
                    user_interface.add_space(SymbolLayoutEditorView::TAKE_OVER_GROUPBOX_SPACING);
                }
            }
        } else {
            for field_index in field_render_indices {
                let Some(field_draft) = self.draft.field_drafts.get_mut(field_index) else {
                    continue;
                };
                if let Some(field_span) = field_spans_by_position.get(&field_index) {
                    if field_span.offset_in_bytes > next_visible_offset {
                        let unassigned_size = field_span.offset_in_bytes.saturating_sub(next_visible_offset);
                        let move_down_field = Some(SymbolLayoutUnassignedAdjacentField {
                            field_position: field_span.field_position,
                            offset_in_bytes: field_span.offset_in_bytes,
                            size_in_bytes: field_span.size_in_bytes,
                        });
                        for unassigned_row_context in SymbolLayoutDraftOps::build_unassigned_row_contexts(
                            next_visible_offset,
                            unassigned_size,
                            &unassigned_split_offsets,
                            previous_visible_field,
                            move_down_field,
                        ) {
                            let is_selected = self
                                .selected_unassigned_span
                                .is_some_and(|selected_unassigned_span| {
                                    selected_unassigned_span.matches(None, unassigned_row_context.offset_in_bytes, unassigned_row_context.size_in_bytes)
                                });
                            if let Some(unassigned_row_action) = SymbolLayoutUnassignedRowView::new(
                                self.symbol_layout_editor_view.app_context.clone(),
                                self.symbol_layout_editor_view
                                    .symbol_layout_editor_view_data
                                    .clone(),
                                None,
                                &unassigned_row_context,
                                true,
                                true,
                                is_selected,
                            )
                            .show(user_interface)
                            {
                                pending_unassigned_row_action = Some((None, unassigned_row_context, unassigned_row_action));
                            }
                        }
                    }
                    next_visible_offset = next_visible_offset.max(
                        field_span
                            .offset_in_bytes
                            .saturating_add(field_span.size_in_bytes),
                    );
                    previous_visible_field = Some(SymbolLayoutUnassignedAdjacentField {
                        field_position: field_span.field_position,
                        offset_in_bytes: field_span.offset_in_bytes,
                        size_in_bytes: field_span.size_in_bytes,
                    });
                }
                let (can_move_up, can_move_down) = if let Some((layout_size_in_bytes, field_spans)) = field_spans.as_ref() {
                    (
                        SymbolLayoutDraftOps::can_move_struct_field_up(field_spans, &unassigned_split_offsets, field_index),
                        SymbolLayoutDraftOps::can_move_struct_field_down(field_spans, *layout_size_in_bytes, &unassigned_split_offsets, field_index),
                    )
                } else {
                    (false, false)
                };
                if let Some(field_row_action) = SymbolLayoutFieldRowView::new(
                    self.symbol_layout_editor_view.app_context.clone(),
                    self.symbol_layout_editor_view
                        .symbol_layout_editor_view_data
                        .clone(),
                    self.project_symbol_catalog,
                    layout_kind,
                    field_draft,
                    field_index,
                    self.selected_field_layout_id.is_none() && self.selected_field_index == Some(field_index),
                    can_move_up,
                    can_move_down,
                    None,
                    true,
                )
                .show(user_interface)
                {
                    pending_field_row_action = Some((field_index, field_row_action));
                }
            }
        }

        if !layout_kind.is_union()
            && let Some((layout_size_in_bytes, _field_spans)) = field_spans.as_ref()
            && *layout_size_in_bytes > next_visible_offset
        {
            let unassigned_size = layout_size_in_bytes.saturating_sub(next_visible_offset);
            let move_up_field = previous_visible_field;
            for unassigned_row_context in
                SymbolLayoutDraftOps::build_unassigned_row_contexts(next_visible_offset, unassigned_size, &unassigned_split_offsets, move_up_field, None)
            {
                let is_selected = self
                    .selected_unassigned_span
                    .is_some_and(|selected_unassigned_span| {
                        selected_unassigned_span.matches(None, unassigned_row_context.offset_in_bytes, unassigned_row_context.size_in_bytes)
                    });
                if let Some(unassigned_row_action) = SymbolLayoutUnassignedRowView::new(
                    self.symbol_layout_editor_view.app_context.clone(),
                    self.symbol_layout_editor_view
                        .symbol_layout_editor_view_data
                        .clone(),
                    None,
                    &unassigned_row_context,
                    true,
                    true,
                    is_selected,
                )
                .show(user_interface)
                {
                    pending_unassigned_row_action = Some((None, unassigned_row_context, unassigned_row_action));
                }
            }
        }

        let (field_context_menu_target, unassigned_context_menu_target) = self
            .symbol_layout_editor_view
            .symbol_layout_editor_view_data
            .read("SymbolLayoutEditor context menus")
            .and_then(|symbol_layout_editor_view_data| {
                Some((
                    symbol_layout_editor_view_data
                        .get_field_context_menu_target()
                        .cloned(),
                    symbol_layout_editor_view_data
                        .get_unassigned_context_menu_target()
                        .cloned(),
                ))
            })
            .unwrap_or((None, None));

        if let Some(field_context_menu_target) = field_context_menu_target
            && field_context_menu_target.get_layout_id().is_none()
            && field_context_menu_target.get_field_index() < field_count
            && let Some(field_row_action) = render_field_context_menu(
                self.symbol_layout_editor_view,
                user_interface,
                self.draft.layout_kind,
                &field_context_menu_target,
                field_count,
                !self.draft.layout_kind.is_union(),
            )
        {
            pending_field_row_action = Some((field_context_menu_target.get_field_index(), field_row_action));
        }

        if let Some(unassigned_context_menu_target) = unassigned_context_menu_target
            && let Some(unassigned_row_action) = render_unassigned_context_menu(
                self.symbol_layout_editor_view,
                user_interface,
                &unassigned_context_menu_target,
                unassigned_context_menu_target.get_layout_id().is_none(),
            )
        {
            let unassigned_row_context = SymbolLayoutUnassignedRowContext {
                offset_in_bytes: unassigned_context_menu_target.get_offset_in_bytes(),
                size_in_bytes: unassigned_context_menu_target.get_size_in_bytes(),
                move_up_field: None,
                move_down_field: None,
                move_up_unassigned_span: None,
                move_down_unassigned_span: None,
                merge_above_span: unassigned_context_menu_target.get_merge_above_span().cloned(),
                merge_below_span: unassigned_context_menu_target.get_merge_below_span().cloned(),
            };
            pending_unassigned_row_action = Some((
                unassigned_context_menu_target
                    .get_layout_id()
                    .map(str::to_string),
                unassigned_row_context,
                unassigned_row_action,
            ));
        }

        if let Some((target_layout_id, unassigned_row_context, unassigned_row_action)) = pending_unassigned_row_action {
            apply_unassigned_row_action(
                self.symbol_layout_editor_view,
                self.project_symbol_catalog,
                self.draft,
                target_layout_id,
                unassigned_row_context,
                unassigned_row_action,
            );
        }

        if let Some((variant_layout_id, field_index, field_row_action)) = pending_variant_field_row_action {
            field_row_action.apply_to_variant_layout(
                self.symbol_layout_editor_view,
                self.project_symbol_catalog,
                self.draft,
                variant_layout_id,
                field_index,
            );
        }

        if let Some((field_index, field_row_action)) = pending_field_row_action {
            field_row_action.apply_to_layout_draft(
                self.symbol_layout_editor_view,
                self.project_symbol_catalog,
                self.draft,
                field_index,
                field_spans.as_ref(),
                &unassigned_split_offsets,
            );
        }
    }
}
