mod authoring;
mod controls;
mod details;
mod list;
mod rows;
mod takeovers;
mod toolbars;

use crate::app_context::AppContext;
use crate::ui::list_navigation::ListNavigationDirection;
use crate::views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutEditDraft, SymbolLayoutEditorTakeOverState, SymbolLayoutEditorViewData,
};
use authoring::symbol_layout_field_draft_factory::SymbolLayoutFieldDraftFactory;
use details::symbol_layout_details_focus::{clear_struct_viewer_if_symbol_layout_focused, focus_selected_layout_in_struct_viewer};
use eframe::egui::{Align, Direction, Key, Layout, RichText, Ui, Widget};
use list::symbol_layout_list_panel_view::SymbolLayoutListPanelView;
use squalr_engine_api::commands::{
    project_symbols::{
        delete_layout::project_symbols_delete_layout_request::ProjectSymbolsDeleteLayoutRequest,
        upsert_layout::project_symbols_upsert_layout_request::ProjectSymbolsUpsertLayoutRequest,
    },
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone)]
pub struct SymbolLayoutEditorView {
    app_context: Arc<AppContext>,
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl SymbolLayoutEditorView {
    pub const WINDOW_ID: &'static str = "window_symbol_layout_editor";
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const FIELD_ROW_HEIGHT: f32 = 28.0;
    const LIST_ROW_HEIGHT: f32 = 28.0;
    const ICON_BUTTON_WIDTH: f32 = 36.0;
    const FIELD_INPUT_SPACING: f32 = 8.0;
    const TAKE_OVER_HEADER_HEIGHT: f32 = 32.0;
    const TAKE_OVER_PADDING_X: f32 = 0.0;
    const TAKE_OVER_PADDING_Y: f32 = 0.0;
    const TAKE_OVER_CONTENT_PADDING_X: f32 = 12.0;
    const TAKE_OVER_HEADER_TITLE_PADDING_X: f32 = 8.0;
    const TAKE_OVER_SECTION_SPACING: f32 = 12.0;
    const TAKE_OVER_GROUPBOX_SPACING: f32 = 8.0;
    const TAKE_OVER_GROUPBOX_SIDE_PADDING: f32 = 8.0;
    const TAKE_OVER_BOTTOM_PADDING: f32 = 8.0;
    const TAKE_OVER_ACTION_BUTTON_WIDTH: f32 = 120.0;
    const TAKE_OVER_ACTION_BUTTON_SPACING: f32 = 12.0;
    const FIELD_ADD_BUTTON_CORNER_RADIUS: u8 = 8;
    const FIELD_ROW_LEFT_PADDING: f32 = 8.0;
    const FIELD_ROW_ICON_SIZE: f32 = 16.0;
    const FIELD_ROW_ICON_GAP: f32 = 4.0;
    const FIELD_ROW_PREVIEW_GAP: f32 = 12.0;
    const FIELD_CONTEXT_MENU_WIDTH: f32 = 184.0;
    const UNION_VARIANT_CHILD_INDENT: f32 = 20.0;
    const DEFINE_FIELD_CONTAINER_SELECTOR_WIDTH: f32 = 118.0;
    const DEFINE_FIELD_GROUPBOX_SIDE_PADDING: f32 = 8.0;
    const LAYOUT_KIND_COMBO_WIDTH: f32 = 128.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_layout_editor_view_data = app_context
            .dependency_container
            .register(SymbolLayoutEditorViewData::new());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();

        Self {
            app_context,
            symbol_layout_editor_view_data,
            struct_viewer_view_data,
        }
    }

    fn get_opened_project_symbol_catalog(&self) -> Option<ProjectSymbolCatalog> {
        Self::get_opened_project_symbol_catalog_from_context(&self.app_context)
    }

    fn get_opened_project_symbol_catalog_from_context(app_context: &AppContext) -> Option<ProjectSymbolCatalog> {
        let opened_project = app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project = opened_project.read().ok()?;

        opened_project.as_ref().map(|opened_project| {
            opened_project
                .get_project_info()
                .get_project_symbol_catalog()
                .clone()
        })
    }

    fn persist_symbol_layout_descriptor_with_context(
        app_context: &AppContext,
        original_struct_layout_id: Option<String>,
        struct_layout_descriptor: &StructLayoutDescriptor,
    ) {
        ProjectSymbolsUpsertLayoutRequest::from_struct_layout_descriptor(original_struct_layout_id, struct_layout_descriptor).send(
            &app_context.engine_unprivileged_state,
            |response| {
                if !response.success {
                    log::error!(
                        "Failed to persist symbol layout `{}` through project-symbols upsert-layout command: {}.",
                        response.struct_layout_id,
                        response.error.as_deref().unwrap_or("unknown error")
                    );
                }
            },
        );
    }

    fn persist_symbol_layout_descriptor(
        &self,
        original_struct_layout_id: Option<String>,
        struct_layout_descriptor: &StructLayoutDescriptor,
    ) {
        Self::persist_symbol_layout_descriptor_with_context(&self.app_context, original_struct_layout_id, struct_layout_descriptor);
    }

    fn delete_symbol_layout(
        &self,
        _project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
    ) {
        ProjectSymbolsDeleteLayoutRequest::new(layout_id).send(&self.app_context.engine_unprivileged_state, |response| {
            if !response.success {
                log::error!(
                    "Failed to delete symbol layout `{}` through project-symbols delete-layout command: {}.",
                    response.struct_layout_id,
                    response.error.as_deref().unwrap_or("unknown error")
                );
            }
        });
        SymbolLayoutEditorViewData::cancel_take_over_state(self.symbol_layout_editor_view_data.clone());
        clear_struct_viewer_if_symbol_layout_focused(self.struct_viewer_view_data.clone());
    }

    fn symbol_layout_take_over_has_unsaved_changes(
        baseline_project_symbol_catalog: Option<&ProjectSymbolCatalog>,
        baseline_draft: &SymbolLayoutEditDraft,
        edited_draft: &SymbolLayoutEditDraft,
        edited_struct_layout_descriptor: Option<&squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor>,
        unassigned_split_offsets: &BTreeSet<u64>,
    ) -> bool {
        if let (Some(baseline_project_symbol_catalog), Some(original_layout_id), Some(edited_struct_layout_descriptor)) = (
            baseline_project_symbol_catalog,
            edited_draft.original_layout_id.as_deref(),
            edited_struct_layout_descriptor,
        ) && let Some(baseline_struct_layout_descriptor) = baseline_project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == original_layout_id)
        {
            return edited_struct_layout_descriptor.get_struct_layout_id() != baseline_struct_layout_descriptor.get_struct_layout_id()
                || edited_struct_layout_descriptor.get_struct_layout_definition() != baseline_struct_layout_descriptor.get_struct_layout_definition();
        }

        edited_draft != baseline_draft || !unassigned_split_offsets.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolLayoutEditorView;
    use super::authoring::symbol_layout_draft_analyzer::SymbolLayoutDraftAnalyzer;
    use super::authoring::symbol_layout_variant_session::SymbolLayoutVariantSession;
    use super::details::symbol_layout_details_focus::build_field_details_struct;
    use super::rows::symbol_layout_field_row_action::grow_draft_size_to_fit_fields;
    use super::rows::symbol_layout_field_row_view::SymbolLayoutFieldRowView;
    use crate::views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData;
    use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
        SymbolLayoutEditDraft, SymbolLayoutEditorViewData, SymbolLayoutFieldEditDraft, SymbolLayoutFieldOffsetMode,
    };
    use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::SymbolLayoutFieldContainerKind;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::{built_in_types::u32::data_type_u32::DataTypeU32, data_type_ref::DataTypeRef},
        data_values::anonymous_value_string_format::AnonymousValueStringFormat,
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::{
            project_symbol_catalog::ProjectSymbolCatalog,
            symbol_layouts::symbol_layout_draft_ops::{
                SymbolLayoutDraftOps, SymbolLayoutFieldSpan, SymbolLayoutUnassignedAdjacentField, SymbolLayoutUnassignedRowContext,
            },
        },
        structs::symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
    };
    use std::collections::BTreeSet;

    fn create_static_field_draft(
        field_name: &str,
        offset_in_bytes: u64,
    ) -> SymbolLayoutFieldEditDraft {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new(DataTypeU32::DATA_TYPE_ID));

        field_draft.field_name = field_name.to_string();
        field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = offset_in_bytes.to_string();
        field_draft
    }

    #[test]
    fn format_field_data_type_preview_includes_fixed_array_length() {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("u16"));

        field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::FixedArray;
        field_draft.container_edit.fixed_array_length = String::from("4");

        assert_eq!(SymbolLayoutFieldRowView::format_data_type_preview(&field_draft), "u16[4]");
    }

    #[test]
    fn build_field_details_struct_hides_static_offset_authoring_rows() {
        let field_draft = create_static_field_draft("health", 16);
        let details_struct = build_field_details_struct(&ProjectSymbolCatalog::default(), SymbolicLayoutKind::Struct, &field_draft);

        assert!(
            !details_struct
                .get_fields()
                .iter()
                .any(|field| field.get_name().contains("offset"))
        );
    }

    #[test]
    fn symbol_layout_take_over_dirty_check_includes_unassigned_split_offsets() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let baseline_draft = SymbolLayoutEditDraft {
            original_layout_id: Some(String::from("module.root")),
            layout_id: String::from("module.root"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_static_field_draft("value", 16)],
        };
        let baseline_descriptor = SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &baseline_draft)
            .expect("Expected baseline descriptor to build.");
        let baseline_project_symbol_catalog = ProjectSymbolCatalog::new(vec![baseline_descriptor]);
        let edited_split_offsets = BTreeSet::from([8]);
        let edited_descriptor = SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(
            &project_symbol_catalog,
            &baseline_draft,
            &edited_split_offsets,
        )
        .expect("Expected edited descriptor to build.");

        assert!(SymbolLayoutEditorView::symbol_layout_take_over_has_unsaved_changes(
            Some(&baseline_project_symbol_catalog),
            &baseline_draft,
            &baseline_draft,
            Some(&edited_descriptor),
            &edited_split_offsets,
        ));
    }

    #[test]
    fn symbol_layout_take_over_dirty_check_ignores_unchanged_baseline_descriptor() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let baseline_draft = SymbolLayoutEditDraft {
            original_layout_id: Some(String::from("module.root")),
            layout_id: String::from("module.root"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_static_field_draft("value", 16)],
        };
        let baseline_descriptor = SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &baseline_draft)
            .expect("Expected baseline descriptor to build.");
        let baseline_project_symbol_catalog = ProjectSymbolCatalog::new(vec![baseline_descriptor.clone()]);

        assert!(!SymbolLayoutEditorView::symbol_layout_take_over_has_unsaved_changes(
            Some(&baseline_project_symbol_catalog),
            &baseline_draft,
            &baseline_draft,
            Some(&baseline_descriptor),
            &BTreeSet::new(),
        ));
    }

    #[test]
    fn grow_draft_size_to_fit_fields_expands_declared_size_for_static_offsets() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("4"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_static_field_draft("health", 8)],
        };

        grow_draft_size_to_fit_fields(&ProjectSymbolCatalog::default(), &mut draft);

        assert_eq!(draft.size_text, "12");
    }

    #[test]
    fn field_insert_index_for_offset_inserts_after_prior_spans() {
        let field_spans = [
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 0,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 12,
                size_in_bytes: 4,
            },
        ];

        assert_eq!(SymbolLayoutDraftOps::field_insert_index_for_offset(&field_spans, 2, 8), 1);
    }

    #[test]
    fn resolve_tail_unassigned_offset_uses_layout_end_gap_only() {
        let field_spans = [
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 4,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 12,
                size_in_bytes: 4,
            },
        ];

        assert_eq!(SymbolLayoutDraftOps::resolve_tail_unassigned_offset(&field_spans, 24), Some(16));
    }

    #[test]
    fn resolve_tail_unassigned_offset_rejects_internal_gaps_without_tail_space() {
        let field_spans = [
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 0,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 12,
                size_in_bytes: 4,
            },
        ];

        assert_eq!(SymbolLayoutDraftOps::resolve_first_unassigned_offset(&field_spans, 16), Some(4));
        assert_eq!(SymbolLayoutDraftOps::resolve_tail_unassigned_offset(&field_spans, 16), None);
    }

    #[test]
    fn move_unassigned_span_up_places_previous_field_after_gap() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_static_field_draft("health", 0),
                create_static_field_draft("mana", 16),
            ],
        };
        let row_context = SymbolLayoutUnassignedRowContext {
            offset_in_bytes: 4,
            size_in_bytes: 12,
            move_up_field: Some(SymbolLayoutUnassignedAdjacentField {
                field_position: 0,
                offset_in_bytes: 0,
                size_in_bytes: 4,
            }),
            move_down_field: None,
            move_up_unassigned_span: None,
            move_down_unassigned_span: None,
            merge_above_span: None,
            merge_below_span: None,
        };

        let updated_unassigned_selection = SymbolLayoutDraftOps::move_unassigned_span_up(&mut draft, row_context).expect("Expected span to move.");

        assert_eq!(updated_unassigned_selection.get_offset_in_bytes(), 0);
        assert_eq!(updated_unassigned_selection.get_size_in_bytes(), 12);
        assert_eq!(draft.field_drafts[0].offset_mode, SymbolLayoutFieldOffsetMode::Static);
        assert_eq!(draft.field_drafts[0].static_offset_in_bytes, "12");
    }

    #[test]
    fn move_unassigned_span_up_preserves_previous_unassigned_boundary() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_static_field_draft("health", 12),
                create_static_field_draft("mana", 28),
            ],
        };
        let row_context = SymbolLayoutUnassignedRowContext {
            offset_in_bytes: 16,
            size_in_bytes: 12,
            move_up_field: Some(SymbolLayoutUnassignedAdjacentField {
                field_position: 0,
                offset_in_bytes: 12,
                size_in_bytes: 4,
            }),
            move_down_field: None,
            move_up_unassigned_span: None,
            move_down_unassigned_span: None,
            merge_above_span: None,
            merge_below_span: None,
        };
        let updated_unassigned_selection = SymbolLayoutDraftOps::move_unassigned_span_up(&mut draft, row_context).expect("Expected span to move.");
        let mut split_offsets = BTreeSet::new();

        if let Some(split_offset_to_preserve) = SymbolLayoutDraftOps::split_offset_to_preserve_unassigned_move_up(&updated_unassigned_selection) {
            split_offsets.insert(split_offset_to_preserve);
        }
        let descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(&project_symbol_catalog, &draft, &split_offsets)
                .expect("Expected moved unassigned descriptor to build.");
        let fields = descriptor.get_struct_layout_definition().get_fields();

        assert_eq!(updated_unassigned_selection.get_offset_in_bytes(), 12);
        assert_eq!(updated_unassigned_selection.get_size_in_bytes(), 12);
        assert_eq!(split_offsets, BTreeSet::from([12]));
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0].to_string(), "unassigned[12]");
        assert_eq!(fields[1].to_string(), "unassigned[12]");
        assert_eq!(fields[2].to_string(), "health:u32");
        assert_eq!(fields[3].to_string(), "mana:u32");
    }

    #[test]
    fn move_unassigned_span_down_places_next_field_before_gap() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_static_field_draft("health", 0),
                create_static_field_draft("mana", 16),
            ],
        };
        let row_context = SymbolLayoutUnassignedRowContext {
            offset_in_bytes: 4,
            size_in_bytes: 12,
            move_up_field: None,
            move_down_field: Some(SymbolLayoutUnassignedAdjacentField {
                field_position: 1,
                offset_in_bytes: 16,
                size_in_bytes: 4,
            }),
            move_up_unassigned_span: None,
            move_down_unassigned_span: None,
            merge_above_span: None,
            merge_below_span: None,
        };

        let updated_unassigned_selection = SymbolLayoutDraftOps::move_unassigned_span_down(&mut draft, row_context).expect("Expected span to move.");

        assert_eq!(updated_unassigned_selection.get_offset_in_bytes(), 8);
        assert_eq!(updated_unassigned_selection.get_size_in_bytes(), 12);
        assert_eq!(draft.field_drafts[1].offset_mode, SymbolLayoutFieldOffsetMode::Static);
        assert_eq!(draft.field_drafts[1].static_offset_in_bytes, "4");
    }

    #[test]
    fn move_unassigned_span_down_preserves_next_unassigned_boundary() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_static_field_draft("health", 0),
                create_static_field_draft("mana", 16),
            ],
        };
        let row_context = SymbolLayoutUnassignedRowContext {
            offset_in_bytes: 4,
            size_in_bytes: 12,
            move_up_field: None,
            move_down_field: Some(SymbolLayoutUnassignedAdjacentField {
                field_position: 1,
                offset_in_bytes: 16,
                size_in_bytes: 4,
            }),
            move_up_unassigned_span: None,
            move_down_unassigned_span: None,
            merge_above_span: None,
            merge_below_span: None,
        };
        let updated_unassigned_selection = SymbolLayoutDraftOps::move_unassigned_span_down(&mut draft, row_context).expect("Expected span to move.");
        let mut split_offsets = BTreeSet::new();

        if let Some(split_offset_to_preserve) = SymbolLayoutDraftOps::split_offset_to_preserve_unassigned_move_down(&updated_unassigned_selection) {
            split_offsets.insert(split_offset_to_preserve);
        }
        let descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(&project_symbol_catalog, &draft, &split_offsets)
                .expect("Expected moved unassigned descriptor to build.");
        let fields = descriptor.get_struct_layout_definition().get_fields();

        assert_eq!(updated_unassigned_selection.get_offset_in_bytes(), 8);
        assert_eq!(updated_unassigned_selection.get_size_in_bytes(), 12);
        assert_eq!(split_offsets, BTreeSet::from([20]));
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0].to_string(), "health:u32");
        assert_eq!(fields[1].to_string(), "mana:u32");
        assert_eq!(fields[2].to_string(), "unassigned[12]");
        assert_eq!(fields[3].to_string(), "unassigned[12]");
    }

    #[test]
    fn move_struct_field_down_over_gap_places_field_before_next_field() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_static_field_draft("field_a", 0),
                create_static_field_draft("field_b", 16),
            ],
        };
        let field_spans = [
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 0,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 16,
                size_in_bytes: 4,
            },
        ];

        assert!(SymbolLayoutDraftOps::move_struct_field_down(&mut draft, &field_spans, 32, &BTreeSet::new(), 0));

        assert_eq!(draft.field_drafts[0].field_name, "field_a");
        assert_eq!(draft.field_drafts[0].static_offset_in_bytes, "12");
        assert_eq!(draft.field_drafts[1].field_name, "field_b");
        assert_eq!(draft.field_drafts[1].static_offset_in_bytes, "16");
    }

    #[test]
    fn move_struct_field_down_crosses_only_next_split_unassigned_row() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_static_field_draft("field_a", 0),
                create_static_field_draft("field_b", 16),
            ],
        };
        let field_spans = [
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 0,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 16,
                size_in_bytes: 4,
            },
        ];
        let split_offsets = BTreeSet::from([8]);

        assert!(SymbolLayoutDraftOps::move_struct_field_down(&mut draft, &field_spans, 32, &split_offsets, 0));

        assert_eq!(draft.field_drafts[0].field_name, "field_a");
        assert_eq!(draft.field_drafts[0].static_offset_in_bytes, "4");
        assert_eq!(draft.field_drafts[0].offset_mode, SymbolLayoutFieldOffsetMode::Static);
        assert_eq!(draft.field_drafts[1].field_name, "field_b");
        assert_eq!(draft.field_drafts[1].static_offset_in_bytes, "16");
    }

    #[test]
    fn move_struct_field_down_converts_sequential_field_to_static_offset() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                {
                    let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new(DataTypeU32::DATA_TYPE_ID));
                    field_draft.field_name = String::from("field_a");
                    field_draft
                },
                create_static_field_draft("field_b", 16),
            ],
        };
        let field_spans = [
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 0,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 16,
                size_in_bytes: 4,
            },
        ];

        assert!(SymbolLayoutDraftOps::move_struct_field_down(&mut draft, &field_spans, 32, &BTreeSet::new(), 0));

        assert_eq!(draft.field_drafts[0].field_name, "field_a");
        assert_eq!(draft.field_drafts[0].offset_mode, SymbolLayoutFieldOffsetMode::Static);
        assert_eq!(draft.field_drafts[0].static_offset_in_bytes, "12");
        assert_eq!(draft.field_drafts[1].field_name, "field_b");
        assert_eq!(draft.field_drafts[1].static_offset_in_bytes, "16");
    }

    #[test]
    fn move_struct_field_up_crosses_only_previous_split_unassigned_row() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_static_field_draft("field_a", 12),
                create_static_field_draft("field_b", 16),
            ],
        };
        let field_spans = [
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 12,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 16,
                size_in_bytes: 4,
            },
        ];
        let split_offsets = BTreeSet::from([4]);

        assert!(SymbolLayoutDraftOps::move_struct_field_up(&mut draft, &field_spans, &split_offsets, 0));

        assert_eq!(draft.field_drafts[0].field_name, "field_a");
        assert_eq!(draft.field_drafts[0].static_offset_in_bytes, "4");
        assert_eq!(draft.field_drafts[1].field_name, "field_b");
        assert_eq!(draft.field_drafts[1].static_offset_in_bytes, "16");
    }

    #[test]
    fn move_struct_field_up_preserves_adjacent_unassigned_boundary() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("12"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_static_field_draft("field_a", 4)],
        };
        let field_spans = [SymbolLayoutFieldSpan {
            field_position: 0,
            offset_in_bytes: 4,
            size_in_bytes: 4,
        }];
        let mut split_offsets = BTreeSet::new();
        let split_offset_to_preserve = SymbolLayoutDraftOps::split_offset_to_preserve_field_move_up(&field_spans, 12, &split_offsets, 0);

        assert!(SymbolLayoutDraftOps::move_struct_field_up(&mut draft, &field_spans, &split_offsets, 0));
        if let Some(split_offset_to_preserve) = split_offset_to_preserve {
            split_offsets.insert(split_offset_to_preserve);
        }
        let descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(&project_symbol_catalog, &draft, &split_offsets)
                .expect("Expected moved field descriptor to build.");
        let fields = descriptor.get_struct_layout_definition().get_fields();

        assert_eq!(split_offsets, BTreeSet::from([8]));
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].to_string(), "field_a:u32");
        assert_eq!(fields[1].to_string(), "unassigned[4]");
        assert_eq!(fields[2].to_string(), "unassigned[4]");
    }

    #[test]
    fn move_struct_field_down_swaps_contiguous_neighbor_offsets() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("player"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("8"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_static_field_draft("field_a", 0),
                create_static_field_draft("field_b", 4),
            ],
        };
        let field_spans = [
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 0,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 4,
                size_in_bytes: 4,
            },
        ];

        assert!(SymbolLayoutDraftOps::move_struct_field_down(&mut draft, &field_spans, 8, &BTreeSet::new(), 0));

        assert_eq!(draft.field_drafts[0].field_name, "field_a");
        assert_eq!(draft.field_drafts[0].static_offset_in_bytes, "4");
        assert_eq!(draft.field_drafts[1].field_name, "field_b");
        assert_eq!(draft.field_drafts[1].static_offset_in_bytes, "0");
    }

    #[test]
    fn resolve_first_unassigned_offset_finds_gap_between_fields() {
        let field_spans = [
            SymbolLayoutFieldSpan {
                field_position: 0,
                offset_in_bytes: 0,
                size_in_bytes: 4,
            },
            SymbolLayoutFieldSpan {
                field_position: 1,
                offset_in_bytes: 12,
                size_in_bytes: 4,
            },
        ];

        assert_eq!(SymbolLayoutDraftOps::resolve_first_unassigned_offset(&field_spans, 24), Some(4));
    }

    #[test]
    fn resolve_first_unassigned_offset_finds_tail_gap() {
        let field_spans = [SymbolLayoutFieldSpan {
            field_position: 0,
            offset_in_bytes: 0,
            size_in_bytes: 4,
        }];

        assert_eq!(SymbolLayoutDraftOps::resolve_first_unassigned_offset(&field_spans, 16), Some(4));
    }

    #[test]
    fn create_union_variant_layout_draft_materializes_missing_variant_as_empty_struct() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let union_draft = SymbolLayoutEditDraft {
            original_layout_id: Some(String::from("actor.state")),
            layout_id: String::from("actor.state"),
            layout_kind: SymbolicLayoutKind::Union,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: Vec::new(),
        };
        let mut variant_field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new(DataTypeU32::DATA_TYPE_ID));

        variant_field_draft.field_name = String::from("Variant 1");

        let variant_draft = SymbolLayoutVariantSession::create_union_variant_layout_draft(&project_symbol_catalog, &union_draft, 0, &variant_field_draft);

        assert_eq!(variant_draft.original_layout_id, None);
        assert_eq!(variant_draft.layout_id, "actor.state.variant_1");
        assert_eq!(variant_draft.layout_kind, SymbolicLayoutKind::Struct);
        assert_eq!(variant_draft.size_text, "32");
        assert!(variant_draft.field_drafts.is_empty());
    }

    #[test]
    fn create_union_variant_layout_draft_uses_existing_referenced_layout() {
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
            String::from("actor.state.alive"),
            SymbolicStructDefinition::new(String::from("actor.state.alive"), Vec::new()),
        )]);
        let union_draft = SymbolLayoutEditDraft {
            original_layout_id: Some(String::from("actor.state")),
            layout_id: String::from("actor.state"),
            layout_kind: SymbolicLayoutKind::Union,
            size_text: String::from("64"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: Vec::new(),
        };
        let variant_field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("actor.state.alive"));

        let variant_draft = SymbolLayoutVariantSession::create_union_variant_layout_draft(&project_symbol_catalog, &union_draft, 0, &variant_field_draft);

        assert_eq!(variant_draft.original_layout_id.as_deref(), Some("actor.state.alive"));
        assert_eq!(variant_draft.layout_id, "actor.state.alive");
        assert_eq!(variant_draft.size_text, "64");
    }

    #[test]
    fn effective_catalog_includes_pending_variant_layout_descriptor() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let mut variant_draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("actor.state.variant_1"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("16"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: Vec::new(),
        };

        variant_draft
            .field_drafts
            .push(create_static_field_draft("health", 0));

        let effective_project_symbol_catalog = SymbolLayoutVariantSession::build_effective_project_symbol_catalog_from_pending_drafts(
            &project_symbol_catalog,
            &[(variant_draft, BTreeSet::new())],
        );

        assert!(
            effective_project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "actor.state.variant_1")
        );
    }

    #[test]
    fn build_unassigned_row_contexts_splits_contiguous_unassigned_rows() {
        let split_offsets = BTreeSet::from([4, 12]);
        let row_contexts = SymbolLayoutDraftOps::build_unassigned_row_contexts(0, 16, &split_offsets, None, None);

        assert_eq!(row_contexts.len(), 3);
        assert_eq!(row_contexts[0].offset_in_bytes, 0);
        assert_eq!(row_contexts[0].size_in_bytes, 4);
        assert_eq!(row_contexts[1].offset_in_bytes, 4);
        assert_eq!(row_contexts[1].size_in_bytes, 8);
        assert_eq!(row_contexts[2].offset_in_bytes, 12);
        assert_eq!(row_contexts[2].size_in_bytes, 4);
        assert_eq!(
            row_contexts[1]
                .merge_above_span
                .as_ref()
                .map(|span| (span.get_offset_in_bytes(), span.get_size_in_bytes())),
            Some((0, 12))
        );
        assert_eq!(
            row_contexts[1]
                .merge_below_span
                .as_ref()
                .map(|span| (span.get_offset_in_bytes(), span.get_size_in_bytes())),
            Some((4, 12))
        );
    }

    #[test]
    fn validate_define_field_draft_allows_offset_inside_unassigned_span() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new(DataTypeU32::DATA_TYPE_ID));

        field_draft.field_name = String::from("middle_field");
        field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = String::from("0x10");

        assert_eq!(
            SymbolLayoutDraftAnalyzer::validate_define_field_draft(&project_symbol_catalog, &field_draft, 0x100, 0x40),
            Ok((0x110, 4))
        );
    }

    #[test]
    fn validate_define_field_draft_rejects_field_that_crosses_unassigned_span() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new(DataTypeU32::DATA_TYPE_ID));

        field_draft.field_name = String::from("too_wide");
        field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = String::from("0x3E");

        assert!(SymbolLayoutDraftAnalyzer::validate_define_field_draft(&project_symbol_catalog, &field_draft, 0x100, 0x40).is_err());
    }

    #[test]
    fn format_field_data_type_preview_includes_fixed_array_display_resolver() {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("u64"));

        field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::FixedArray;
        field_draft.container_edit.fixed_array_length = String::from("1024");
        field_draft.container_edit.display_count_resolver_id = String::from("entity.count");

        assert_eq!(
            SymbolLayoutFieldRowView::format_data_type_preview(&field_draft),
            "u64[1024] display resolver(entity.count)"
        );
    }

    #[test]
    fn format_field_data_type_preview_includes_pointer_size() {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("u32"));

        field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::Pointer;
        field_draft.container_edit.pointer_size = PointerScanPointerSize::Pointer64;

        assert_eq!(SymbolLayoutFieldRowView::format_data_type_preview(&field_draft), "u32*(u64)");
    }

    #[test]
    fn format_field_data_type_preview_includes_fixed_pointer_array() {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("Entity"));

        field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::FixedPointerArray;
        field_draft.container_edit.pointer_size = PointerScanPointerSize::Pointer64;
        field_draft.container_edit.fixed_array_length = String::from("1024");
        field_draft.container_edit.display_count_resolver_id = String::from("entity.count");

        assert_eq!(
            SymbolLayoutFieldRowView::format_data_type_preview(&field_draft),
            "Entity*(u64)[1024] display resolver(entity.count)"
        );
    }

    #[test]
    fn format_field_data_type_preview_includes_dynamic_array_resolver() {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("game.item"));

        field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::DynamicArray;
        field_draft.container_edit.dynamic_array_count_resolver_id = String::from("inventory.count");

        assert_eq!(
            SymbolLayoutFieldRowView::format_data_type_preview(&field_draft),
            "game.item[resolver(inventory.count)]"
        );
    }

    #[test]
    fn build_field_details_struct_splits_builtin_data_types_from_symbol_layouts() {
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
            String::from("player.stats"),
            SymbolicStructDefinition::new(String::from("player.stats"), Vec::new()),
        )]);
        let builtin_field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new(DataTypeU32::DATA_TYPE_ID));
        let symbol_layout_field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("player.stats"));

        let builtin_details_struct = build_field_details_struct(&project_symbol_catalog, SymbolicLayoutKind::Struct, &builtin_field_draft);
        let symbol_layout_details_struct = build_field_details_struct(&project_symbol_catalog, SymbolicLayoutKind::Struct, &symbol_layout_field_draft);

        assert!(
            builtin_details_struct
                .get_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_DATA_TYPE)
                .is_some()
        );
        assert!(
            builtin_details_struct
                .get_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_SYMBOL_LAYOUT)
                .is_none()
        );
        assert!(
            symbol_layout_details_struct
                .get_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_DATA_TYPE)
                .is_none()
        );
        assert!(
            symbol_layout_details_struct
                .get_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_SYMBOL_LAYOUT)
                .is_some()
        );
    }

    #[test]
    fn build_field_details_struct_limits_union_variants_to_symbol_layout_selector() {
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
            String::from("player.stats"),
            SymbolicStructDefinition::new(String::from("player.stats"), Vec::new()),
        )]);
        let variant_field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("player.stats"));

        let details_struct = build_field_details_struct(&project_symbol_catalog, SymbolicLayoutKind::Union, &variant_field_draft);

        assert!(
            details_struct
                .get_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_SYMBOL_LAYOUT)
                .is_some()
        );
        assert!(
            details_struct
                .get_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_ELEMENT_TYPE)
                .is_none()
        );
        assert!(
            !details_struct
                .get_fields()
                .iter()
                .any(|field| field.get_name().contains("offset"))
        );
    }
}

impl Widget for SymbolLayoutEditorView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> eframe::egui::Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
            return user_interface
                .allocate_ui_with_layout(
                    user_interface.available_size(),
                    Layout::centered_and_justified(Direction::TopDown),
                    |user_interface| {
                        user_interface
                            .label(RichText::new("Open a project to author reusable symbol layouts.").color(self.app_context.theme.foreground_preview));
                    },
                )
                .response;
        };

        SymbolLayoutEditorViewData::synchronize(self.symbol_layout_editor_view_data.clone(), &project_symbol_catalog);
        let (
            selected_layout_id,
            filter_text,
            take_over_state,
            baseline_project_symbol_catalog,
            baseline_draft,
            draft,
            unassigned_split_offsets,
            selected_field_index,
            selected_field_layout_id,
            selected_unassigned_span,
            define_field_draft,
        ) = self
            .symbol_layout_editor_view_data
            .read("SymbolLayoutEditor view")
            .map(|symbol_layout_editor_view_data| {
                (
                    symbol_layout_editor_view_data
                        .get_selected_layout_id()
                        .map(str::to_string),
                    symbol_layout_editor_view_data.get_filter_text().to_string(),
                    symbol_layout_editor_view_data.get_take_over_state().cloned(),
                    symbol_layout_editor_view_data
                        .get_baseline_project_symbol_catalog()
                        .cloned(),
                    symbol_layout_editor_view_data.get_baseline_draft().cloned(),
                    symbol_layout_editor_view_data.get_draft().cloned(),
                    symbol_layout_editor_view_data
                        .get_unassigned_split_offsets()
                        .clone(),
                    symbol_layout_editor_view_data.get_selected_field_index(),
                    symbol_layout_editor_view_data
                        .get_selected_field_layout_id()
                        .map(str::to_string),
                    symbol_layout_editor_view_data
                        .get_selected_unassigned_span()
                        .cloned(),
                    symbol_layout_editor_view_data.get_define_field_draft().cloned(),
                )
            })
            .unwrap_or((None, String::new(), None, None, None, None, BTreeSet::new(), None, None, None, None));
        let is_take_over_active = take_over_state.is_some();
        let is_window_focused = self
            .app_context
            .window_focus_manager
            .is_window_focused(Self::WINDOW_ID);
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);

        if is_window_focused && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) && is_take_over_active {
            if let Some(SymbolLayoutEditorTakeOverState::DeleteFieldConfirmation { layout_id, .. }) = take_over_state.as_ref() {
                SymbolLayoutEditorViewData::return_to_open_symbol_layout(self.symbol_layout_editor_view_data.clone(), layout_id.clone());
            } else if let Some(SymbolLayoutEditorTakeOverState::DefineFieldFromUnassignedSpan { return_state, .. }) = take_over_state.as_ref() {
                SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
            } else {
                SymbolLayoutEditorViewData::cancel_take_over_state(self.symbol_layout_editor_view_data.clone());
                clear_struct_viewer_if_symbol_layout_focused(self.struct_viewer_view_data.clone());
            }
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            if let Some(selected_layout_id) = selected_layout_id.as_deref() {
                SymbolLayoutEditorViewData::begin_open_symbol_layout(self.symbol_layout_editor_view_data.clone(), &project_symbol_catalog, selected_layout_id);
            }
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp)) {
            let next_layout_id = SymbolLayoutEditorViewData::navigate_symbol_layout_selection(
                self.symbol_layout_editor_view_data.clone(),
                &project_symbol_catalog,
                ListNavigationDirection::Up,
            );
            focus_selected_layout_in_struct_viewer(
                self.app_context.clone(),
                self.struct_viewer_view_data.clone(),
                &project_symbol_catalog,
                next_layout_id.as_deref(),
            );
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown)) {
            let next_layout_id = SymbolLayoutEditorViewData::navigate_symbol_layout_selection(
                self.symbol_layout_editor_view_data.clone(),
                &project_symbol_catalog,
                ListNavigationDirection::Down,
            );
            focus_selected_layout_in_struct_viewer(
                self.app_context.clone(),
                self.struct_viewer_view_data.clone(),
                &project_symbol_catalog,
                next_layout_id.as_deref(),
            );
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::Delete)) {
            if let Some(selected_layout_id) = selected_layout_id.as_deref() {
                SymbolLayoutEditorViewData::request_delete_confirmation(self.symbol_layout_editor_view_data.clone(), selected_layout_id.to_string());
            }
        }

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let content_rect = user_interface.available_rect_before_wrap();
                let mut content_user_interface = user_interface.new_child(
                    eframe::egui::UiBuilder::new()
                        .max_rect(content_rect)
                        .layout(Layout::top_down(Align::Min)),
                );
                match take_over_state.as_ref() {
                    Some(SymbolLayoutEditorTakeOverState::CreateSymbolLayout) => {
                        self.render_symbol_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "",
                            baseline_project_symbol_catalog.as_ref(),
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            &unassigned_split_offsets,
                            selected_field_index,
                            selected_field_layout_id.as_deref(),
                            selected_unassigned_span.as_ref(),
                            true,
                        );
                    }
                    Some(SymbolLayoutEditorTakeOverState::RenameSymbolLayout { .. }) => {
                        self.render_symbol_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "Rename Symbol Layout",
                            baseline_project_symbol_catalog.as_ref(),
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            &unassigned_split_offsets,
                            selected_field_index,
                            selected_field_layout_id.as_deref(),
                            selected_unassigned_span.as_ref(),
                            true,
                        );
                    }
                    Some(SymbolLayoutEditorTakeOverState::OpenSymbolLayout { .. }) => {
                        self.render_symbol_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "Edit Symbol Layout",
                            baseline_project_symbol_catalog.as_ref(),
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            &unassigned_split_offsets,
                            selected_field_index,
                            selected_field_layout_id.as_deref(),
                            selected_unassigned_span.as_ref(),
                            false,
                        );
                    }
                    Some(SymbolLayoutEditorTakeOverState::DefineFieldFromUnassignedSpan {
                        layout_id,
                        offset,
                        size,
                        return_state,
                    }) => {
                        self.render_define_field_from_unassigned_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            layout_id,
                            *offset,
                            *size,
                            return_state,
                            draft.as_ref(),
                            define_field_draft.as_ref(),
                        );
                    }
                    Some(SymbolLayoutEditorTakeOverState::DeleteConfirmation { layout_id }) => {
                        self.render_delete_confirmation_take_over(&mut content_user_interface, &project_symbol_catalog, layout_id);
                    }
                    Some(SymbolLayoutEditorTakeOverState::DeleteFieldConfirmation { layout_id, field_index }) => {
                        self.render_field_delete_confirmation_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            layout_id,
                            *field_index,
                            draft.as_ref(),
                        );
                    }
                    None => {
                        SymbolLayoutListPanelView::new(
                            self.app_context.clone(),
                            self.symbol_layout_editor_view_data.clone(),
                            self.struct_viewer_view_data.clone(),
                            &project_symbol_catalog,
                            selected_layout_id.as_deref(),
                            &filter_text,
                            SymbolLayoutFieldDraftFactory::default_data_type_ref(&self.app_context),
                            false,
                        )
                        .show(&mut content_user_interface);
                    }
                }
            })
            .response
    }
}
