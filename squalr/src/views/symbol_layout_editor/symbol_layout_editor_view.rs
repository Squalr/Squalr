use crate::app_context::AppContext;
use crate::ui::converters::{data_type_to_icon_converter::DataTypeToIconConverter, data_type_to_string_converter::DataTypeToStringConverter};
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::list_navigation::ListNavigationDirection;
use crate::ui::widgets::controls::{
    button::Button as ThemeButton,
    combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
    context_menu::context_menu::ContextMenu,
    data_value_box::data_value_box_view::DataValueBoxView,
    groupbox::GroupBox,
    search_box::SearchBoxView,
    state_layer::StateLayer,
    toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
};
use crate::views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData};
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutDefineFieldReturnState, SymbolLayoutEditDraft, SymbolLayoutEditorTakeOverState, SymbolLayoutEditorViewData, SymbolLayoutFieldContextMenuTarget,
    SymbolLayoutFieldEditDraft, SymbolLayoutFieldElementType, SymbolLayoutFieldOffsetMode, SymbolLayoutUnassignedContextMenuTarget,
    SymbolLayoutUnassignedSelection,
};
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::{SymbolLayoutFieldContainerEdit, SymbolLayoutFieldContainerKind};
use eframe::egui::{
    Align, Align2, Button as EguiButton, Direction, Grid, Id, Key, Layout, Response, RichText, ScrollArea, Sense, Stroke, Ui, UiBuilder, Widget, pos2, vec2,
};
use epaint::{Color32, CornerRadius, Rect, StrokeKind};
use squalr_engine_api::commands::{
    privileged_command_request::PrivilegedCommandRequest, project::save::project_save_request::ProjectSaveRequest,
    registry::set_project_symbols::registry_set_project_symbols_request::RegistrySetProjectSymbolsRequest,
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::{
    data_types::{
        built_in_types::{i32::data_type_i32::DataTypeI32, string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
        data_type_ref::DataTypeRef,
    },
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::{
        symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
        valued_struct::ValuedStruct,
        valued_struct_field::ValuedStructField,
    },
};
use std::{cell::Cell, collections::BTreeSet, str::FromStr, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SymbolLayoutFieldRowAction {
    InsertAfter,
    InsertFieldIntoVariant,
    RequestRemoveFieldConfirmation,
    MoveUp,
    MoveDown,
    SelectField,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SymbolLayoutUnassignedRowAction {
    SelectSpan,
    DefineField,
    MoveUp,
    MoveDown,
    SplitRange,
    MergeAbove,
    MergeBelow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SymbolLayoutRowAction {
    Select,
    Open,
    Rename,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SymbolLayoutFieldTypeOptionKind {
    BuiltIn,
    SymbolLayout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SymbolLayoutFieldTypeOption {
    data_type_ref: DataTypeRef,
    label: String,
    kind: SymbolLayoutFieldTypeOptionKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SymbolLayoutFieldSpan {
    field_position: usize,
    offset_in_bytes: u64,
    size_in_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SymbolLayoutUnassignedAdjacentField {
    field_position: usize,
    offset_in_bytes: u64,
    size_in_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SymbolLayoutUnassignedRowContext {
    offset_in_bytes: u64,
    size_in_bytes: u64,
    move_up_field: Option<SymbolLayoutUnassignedAdjacentField>,
    move_down_field: Option<SymbolLayoutUnassignedAdjacentField>,
    move_up_unassigned_span: Option<SymbolLayoutUnassignedSelection>,
    move_down_unassigned_span: Option<SymbolLayoutUnassignedSelection>,
    merge_above_span: Option<SymbolLayoutUnassignedSelection>,
    merge_below_span: Option<SymbolLayoutUnassignedSelection>,
}

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
    const FIELD_ROW_LEFT_PADDING: f32 = 8.0;
    const FIELD_ROW_ICON_SIZE: f32 = 16.0;
    const FIELD_ROW_ICON_GAP: f32 = 4.0;
    const FIELD_ROW_PREVIEW_GAP: f32 = 12.0;
    const FIELD_CONTEXT_MENU_WIDTH: f32 = 184.0;
    const DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT: usize = 2;
    const DEFINE_FIELD_BUILT_IN_TYPE_ITEM_WIDTH: f32 = 128.0;
    const DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_SPACING: f32 = 4.0;
    const DEFINE_FIELD_CONTAINER_SELECTOR_WIDTH: f32 = 118.0;
    const DEFINE_FIELD_BUILT_IN_TYPE_IDS: [&'static str; 18] = [
        "u8", "i8", "i16", "i16be", "i32", "i32be", "i64", "i64be", "u16", "u16be", "u32", "u32be", "u64", "u64be", "f32", "f32be", "f64", "f64be",
    ];
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

    fn persist_project_symbol_catalog(
        &self,
        updated_project_symbol_catalog: ProjectSymbolCatalog,
    ) {
        Self::persist_project_symbol_catalog_with_context(&self.app_context, updated_project_symbol_catalog);
    }

    fn persist_project_symbol_catalog_with_context(
        app_context: &AppContext,
        updated_project_symbol_catalog: ProjectSymbolCatalog,
    ) {
        let opened_project_lock = app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let did_update_project = match opened_project_lock.write() {
            Ok(mut opened_project) => {
                if let Some(opened_project) = opened_project.as_mut() {
                    let project_info = opened_project.get_project_info_mut();

                    *project_info.get_project_symbol_catalog_mut() = updated_project_symbol_catalog.clone();
                    project_info.set_has_unsaved_changes(true);
                    true
                } else {
                    false
                }
            }
            Err(error) => {
                log::error!("Failed to acquire opened project while persisting symbol layout changes: {}.", error);
                false
            }
        };

        if !did_update_project {
            return;
        }

        let project_save_request = ProjectSaveRequest {};
        project_save_request.send(&app_context.engine_unprivileged_state, |project_save_response| {
            if !project_save_response.success {
                log::error!("Failed to save project after applying symbol layout changes.");
            }
        });

        let registry_set_project_symbols_request = RegistrySetProjectSymbolsRequest {
            project_symbol_catalog: updated_project_symbol_catalog,
        };
        let did_dispatch_registry_sync = registry_set_project_symbols_request.send(&app_context.engine_unprivileged_state, |_response| {});
        if !did_dispatch_registry_sync {
            log::error!("Failed to dispatch project symbol registry sync after symbol layout changes.");
        }
    }

    fn delete_symbol_layout(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
    ) {
        match SymbolLayoutEditorViewData::remove_symbol_layout_from_catalog(project_symbol_catalog, layout_id) {
            Ok(updated_project_symbol_catalog) => {
                self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                SymbolLayoutEditorViewData::cancel_take_over_state(self.symbol_layout_editor_view_data.clone());
                self.clear_struct_viewer_if_symbol_layout_focused();
            }
            Err(error) => {
                log::error!("Failed to delete symbol layout: {}.", error);
            }
        }
    }

    fn default_data_type_ref(&self) -> DataTypeRef {
        let registered_data_types = self
            .app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs();

        registered_data_types
            .iter()
            .find(|data_type_ref| data_type_ref.get_data_type_id() == DataTypeI32::DATA_TYPE_ID)
            .cloned()
            .or_else(|| registered_data_types.first().cloned())
            .unwrap_or_else(|| DataTypeRef::new(DataTypeI32::DATA_TYPE_ID))
    }

    fn build_field_type_options(project_symbol_catalog: &ProjectSymbolCatalog) -> Vec<SymbolLayoutFieldTypeOption> {
        let mut type_options = Self::DEFINE_FIELD_BUILT_IN_TYPE_IDS
            .iter()
            .map(|data_type_id| SymbolLayoutFieldTypeOption {
                data_type_ref: DataTypeRef::new(data_type_id),
                label: DataTypeToStringConverter::convert_data_type_to_string(data_type_id),
                kind: SymbolLayoutFieldTypeOptionKind::BuiltIn,
            })
            .collect::<Vec<_>>();

        for struct_layout_descriptor in project_symbol_catalog.get_struct_layout_descriptors() {
            let struct_layout_id = struct_layout_descriptor.get_struct_layout_id();
            let struct_data_type_ref = DataTypeRef::new(struct_layout_id);

            if !type_options
                .iter()
                .any(|type_option| type_option.data_type_ref == struct_data_type_ref)
            {
                type_options.push(SymbolLayoutFieldTypeOption {
                    data_type_ref: struct_data_type_ref,
                    label: struct_layout_id.to_string(),
                    kind: SymbolLayoutFieldTypeOptionKind::SymbolLayout,
                });
            }
        }

        type_options
    }

    fn filter_field_type_options(
        type_options: &[SymbolLayoutFieldTypeOption],
        search_text: &str,
    ) -> Vec<SymbolLayoutFieldTypeOption> {
        let normalized_search_text = search_text.trim().to_lowercase();

        if normalized_search_text.is_empty() {
            return type_options.to_vec();
        }

        type_options
            .iter()
            .filter(|type_option| {
                type_option
                    .label
                    .to_lowercase()
                    .contains(&normalized_search_text)
                    || type_option
                        .data_type_ref
                        .get_data_type_id()
                        .to_lowercase()
                        .contains(&normalized_search_text)
            })
            .cloned()
            .collect()
    }

    fn define_field_type_popup_width(combo_width: f32) -> f32 {
        let built_in_grid_width = Self::DEFINE_FIELD_BUILT_IN_TYPE_ITEM_WIDTH * Self::DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT as f32
            + Self::DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_SPACING * (Self::DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT.saturating_sub(1) as f32);

        combo_width.max(built_in_grid_width)
    }

    fn define_field_builtin_type_item_width(popup_width: f32) -> f32 {
        let spacing_width = Self::DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_SPACING * (Self::DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT.saturating_sub(1) as f32);

        ((popup_width - spacing_width) / Self::DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT as f32).max(1.0)
    }

    fn define_field_type_search_storage_id(menu_id: &str) -> Id {
        Id::new(("symbol_layout_define_field_type_search", menu_id))
    }

    fn define_field_container_label(container_edit: &SymbolLayoutFieldContainerEdit) -> String {
        match container_edit.kind {
            SymbolLayoutFieldContainerKind::Element => String::from("Value"),
            SymbolLayoutFieldContainerKind::Pointer => format!("Ptr {}", container_edit.pointer_size),
            _ => container_edit.kind.label().to_string(),
        }
    }

    fn create_field_draft_for_layout_kind(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_kind: SymbolicLayoutKind,
        owning_layout_id: &str,
        field_position: usize,
    ) -> SymbolLayoutFieldEditDraft {
        if layout_kind.is_union() {
            let struct_layout_id = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| {
                    struct_layout_descriptor.get_struct_layout_id() != owning_layout_id
                        && struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_layout_kind()
                            == SymbolicLayoutKind::Struct
                })
                .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id().to_string());

            let mut field_draft = SymbolLayoutFieldEditDraft::new(
                struct_layout_id
                    .as_deref()
                    .map(DataTypeRef::new)
                    .unwrap_or_else(|| self.default_data_type_ref()),
            );
            field_draft.field_name = format!("Variant {}", field_position + 1);
            field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::Element;
            field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Sequential;

            return field_draft;
        }

        let mut field_draft = SymbolLayoutFieldEditDraft::new(self.default_data_type_ref());
        field_draft.field_name = format!("field_{}", field_position + 1);
        field_draft
    }

    fn create_field_draft_for_unassigned_span(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_kind: SymbolicLayoutKind,
        owning_layout_id: &str,
        field_position: usize,
        offset_in_bytes: u64,
    ) -> SymbolLayoutFieldEditDraft {
        let mut field_draft = self.create_field_draft_for_layout_kind(project_symbol_catalog, layout_kind, owning_layout_id, field_position);

        if !layout_kind.is_union() {
            field_draft.field_name = format!("field_{:X}", offset_in_bytes);
            field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
            field_draft.static_offset_in_bytes = String::from("0");
        }

        field_draft
    }

    fn field_insert_index_for_offset(
        field_spans: &[SymbolLayoutFieldSpan],
        field_count: usize,
        offset_in_bytes: u64,
    ) -> usize {
        field_spans
            .iter()
            .filter(|field_span| field_span.offset_in_bytes < offset_in_bytes)
            .map(|field_span| field_span.field_position.saturating_add(1))
            .max()
            .unwrap_or(0)
            .min(field_count)
    }

    fn set_field_static_offset(
        draft: &mut SymbolLayoutEditDraft,
        field_position: usize,
        offset_in_bytes: u64,
    ) -> bool {
        let Some(field_draft) = draft.field_drafts.get_mut(field_position) else {
            return false;
        };

        field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = offset_in_bytes.to_string();
        true
    }

    fn build_unique_field_name(
        draft: &SymbolLayoutEditDraft,
        base_name: &str,
    ) -> String {
        let trimmed_base_name = base_name.trim();
        let base_name = if trimmed_base_name.is_empty() {
            if draft.layout_kind.is_union() { "variant" } else { "field" }
        } else {
            trimmed_base_name
        };
        if !draft
            .field_drafts
            .iter()
            .any(|field_draft| field_draft.field_name.trim() == base_name)
        {
            return base_name.to_string();
        }

        let mut suffix_index = 2_u64;
        loop {
            let candidate_name = format!("{}_{}", base_name, suffix_index);
            if !draft
                .field_drafts
                .iter()
                .any(|field_draft| field_draft.field_name.trim() == candidate_name)
            {
                return candidate_name;
            }
            suffix_index = suffix_index.saturating_add(1);
        }
    }

    fn resolve_field_span_by_position(
        field_spans: &[SymbolLayoutFieldSpan],
        field_position: usize,
    ) -> Option<SymbolLayoutFieldSpan> {
        field_spans
            .iter()
            .copied()
            .find(|field_span| field_span.field_position == field_position)
    }

    fn resolve_adjacent_field_span(
        field_spans: &[SymbolLayoutFieldSpan],
        current_field_span: SymbolLayoutFieldSpan,
        direction: ListNavigationDirection,
    ) -> Option<SymbolLayoutFieldSpan> {
        let current_sort_key = (current_field_span.offset_in_bytes, current_field_span.field_position);
        match direction {
            ListNavigationDirection::Up => field_spans
                .iter()
                .copied()
                .filter(|field_span| field_span.field_position != current_field_span.field_position)
                .filter(|field_span| (field_span.offset_in_bytes, field_span.field_position) < current_sort_key)
                .max_by_key(|field_span| (field_span.offset_in_bytes, field_span.field_position)),
            ListNavigationDirection::Down => field_spans
                .iter()
                .copied()
                .filter(|field_span| field_span.field_position != current_field_span.field_position)
                .filter(|field_span| (field_span.offset_in_bytes, field_span.field_position) > current_sort_key)
                .min_by_key(|field_span| (field_span.offset_in_bytes, field_span.field_position)),
        }
    }

    fn resolve_unassigned_row_before_field(
        current_field_span: SymbolLayoutFieldSpan,
        previous_field_span: Option<SymbolLayoutFieldSpan>,
        split_offsets: &BTreeSet<u64>,
    ) -> Option<SymbolLayoutUnassignedSelection> {
        let gap_start = previous_field_span
            .map(|previous_field_span| {
                previous_field_span
                    .offset_in_bytes
                    .saturating_add(previous_field_span.size_in_bytes)
            })
            .unwrap_or(0);
        let gap_end = current_field_span.offset_in_bytes;

        if gap_end <= gap_start {
            return None;
        }

        let row_start = split_offsets
            .iter()
            .copied()
            .filter(|split_offset| *split_offset > gap_start && *split_offset < gap_end)
            .max()
            .unwrap_or(gap_start);

        Some(SymbolLayoutUnassignedSelection::new(row_start, gap_end.saturating_sub(row_start)))
    }

    fn resolve_unassigned_row_after_field(
        current_field_span: SymbolLayoutFieldSpan,
        next_field_span: Option<SymbolLayoutFieldSpan>,
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
    ) -> Option<SymbolLayoutUnassignedSelection> {
        let gap_start = current_field_span
            .offset_in_bytes
            .saturating_add(current_field_span.size_in_bytes);
        let gap_end = next_field_span
            .map(|next_field_span| next_field_span.offset_in_bytes)
            .unwrap_or(layout_size_in_bytes);

        if gap_end <= gap_start {
            return None;
        }

        let row_end = split_offsets
            .iter()
            .copied()
            .filter(|split_offset| *split_offset > gap_start && *split_offset < gap_end)
            .min()
            .unwrap_or(gap_end);

        Some(SymbolLayoutUnassignedSelection::new(gap_start, row_end.saturating_sub(gap_start)))
    }

    fn move_struct_field_up(
        draft: &mut SymbolLayoutEditDraft,
        field_spans: &[SymbolLayoutFieldSpan],
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> bool {
        let Some(current_field_span) = Self::resolve_field_span_by_position(field_spans, field_index) else {
            return false;
        };
        let previous_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Up);

        if let Some(previous_unassigned_row) = Self::resolve_unassigned_row_before_field(current_field_span, previous_field_span, split_offsets) {
            return Self::set_field_static_offset(draft, field_index, previous_unassigned_row.get_offset_in_bytes());
        }

        let Some(previous_field_span) = previous_field_span else {
            return false;
        };
        let moved_current_offset = previous_field_span.offset_in_bytes;
        let moved_previous_offset = previous_field_span
            .offset_in_bytes
            .saturating_add(current_field_span.size_in_bytes);
        Self::set_field_static_offset(draft, field_index, moved_current_offset)
            && Self::set_field_static_offset(draft, previous_field_span.field_position, moved_previous_offset)
    }

    fn move_struct_field_down(
        draft: &mut SymbolLayoutEditDraft,
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> bool {
        let Some(current_field_span) = Self::resolve_field_span_by_position(field_spans, field_index) else {
            return false;
        };
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Down);

        if let Some(next_unassigned_row) = Self::resolve_unassigned_row_after_field(current_field_span, next_field_span, layout_size_in_bytes, split_offsets) {
            return Self::set_field_static_offset(
                draft,
                field_index,
                current_field_span
                    .offset_in_bytes
                    .saturating_add(next_unassigned_row.get_size_in_bytes()),
            );
        }

        let Some(next_field_span) = next_field_span else {
            return false;
        };
        let moved_next_offset = current_field_span.offset_in_bytes;
        let moved_current_offset = current_field_span
            .offset_in_bytes
            .saturating_add(next_field_span.size_in_bytes);
        Self::set_field_static_offset(draft, next_field_span.field_position, moved_next_offset)
            && Self::set_field_static_offset(draft, field_index, moved_current_offset)
    }

    fn can_move_struct_field_up(
        field_spans: &[SymbolLayoutFieldSpan],
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> bool {
        let Some(current_field_span) = Self::resolve_field_span_by_position(field_spans, field_index) else {
            return false;
        };
        let previous_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Up);

        Self::resolve_unassigned_row_before_field(current_field_span, previous_field_span, split_offsets).is_some() || previous_field_span.is_some()
    }

    fn can_move_struct_field_down(
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        field_index: usize,
    ) -> bool {
        let Some(current_field_span) = Self::resolve_field_span_by_position(field_spans, field_index) else {
            return false;
        };
        let next_field_span = Self::resolve_adjacent_field_span(field_spans, current_field_span, ListNavigationDirection::Down);

        Self::resolve_unassigned_row_after_field(current_field_span, next_field_span, layout_size_in_bytes, split_offsets).is_some()
            || next_field_span.is_some()
    }

    fn resolve_first_unassigned_offset(
        field_spans: &[SymbolLayoutFieldSpan],
        layout_size_in_bytes: u64,
    ) -> Option<u64> {
        let mut sorted_field_spans = field_spans.to_vec();
        sorted_field_spans.sort_by_key(|field_span| (field_span.offset_in_bytes, field_span.field_position));

        let mut next_visible_offset = 0_u64;
        for field_span in sorted_field_spans {
            if field_span.offset_in_bytes > next_visible_offset {
                return Some(next_visible_offset);
            }
            next_visible_offset = next_visible_offset.max(
                field_span
                    .offset_in_bytes
                    .saturating_add(field_span.size_in_bytes),
            );
        }

        (next_visible_offset < layout_size_in_bytes).then_some(next_visible_offset)
    }

    fn append_field_to_variant_layout(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &mut SymbolLayoutEditDraft,
        variant_index: usize,
    ) -> bool {
        let Some(variant_field_draft) = union_draft.field_drafts.get(variant_index) else {
            return false;
        };
        let mut variant_draft = Self::create_union_variant_layout_draft(project_symbol_catalog, union_draft, variant_index, variant_field_draft);

        let Some((layout_size_in_bytes, field_spans)) = self.resolve_draft_field_spans(project_symbol_catalog, &variant_draft) else {
            return false;
        };
        let Some(field_offset_in_bytes) = Self::resolve_first_unassigned_offset(&field_spans, layout_size_in_bytes) else {
            return false;
        };

        let field_position = variant_draft.field_drafts.len();
        let mut field_draft =
            self.create_field_draft_for_layout_kind(project_symbol_catalog, SymbolicLayoutKind::Struct, &variant_draft.layout_id, field_position);
        field_draft.field_name = format!("field_{:08X}", field_offset_in_bytes);
        field_draft.field_name = Self::build_unique_field_name(&variant_draft, &field_draft.field_name);
        field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = field_offset_in_bytes.to_string();
        variant_draft.field_drafts.push(field_draft);
        if let Some(variant_field_draft) = union_draft.field_drafts.get_mut(variant_index) {
            variant_field_draft
                .data_type_selection
                .select_single_data_type(DataTypeRef::new(&variant_draft.layout_id));
            variant_field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::Element;
            variant_field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Sequential;
        }

        self.persist_variant_layout_draft(project_symbol_catalog, &variant_draft)
    }

    fn create_union_variant_layout_draft(
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_index: usize,
        variant_field_draft: &SymbolLayoutFieldEditDraft,
    ) -> SymbolLayoutEditDraft {
        let variant_layout_id = variant_field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id();
        if let Some(variant_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == variant_layout_id)
        {
            return Self::create_union_variant_layout_draft_for_id(project_symbol_catalog, union_draft, variant_layout_descriptor.get_struct_layout_id());
        }

        let variant_layout_id = Self::build_union_variant_layout_id(project_symbol_catalog, union_draft, variant_index);
        Self::create_virtual_union_variant_layout_draft(union_draft, variant_layout_id)
    }

    fn create_union_variant_layout_draft_for_id(
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_layout_id: &str,
    ) -> SymbolLayoutEditDraft {
        if let Some(variant_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == variant_layout_id)
        {
            let mut variant_draft = SymbolLayoutEditorViewData::create_draft_from_descriptor(variant_layout_descriptor);

            variant_draft.size_text = union_draft.size_text.clone();
            variant_draft.size_format = union_draft.size_format;

            return variant_draft;
        }

        Self::create_virtual_union_variant_layout_draft(union_draft, variant_layout_id.to_string())
    }

    fn create_virtual_union_variant_layout_draft(
        union_draft: &SymbolLayoutEditDraft,
        variant_layout_id: String,
    ) -> SymbolLayoutEditDraft {
        SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: variant_layout_id,
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: union_draft.size_text.clone(),
            size_format: union_draft.size_format,
            field_drafts: Vec::new(),
        }
    }

    fn build_union_variant_layout_id(
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_index: usize,
    ) -> String {
        let trimmed_union_layout_id = union_draft.layout_id.trim();
        let base_layout_id = if trimmed_union_layout_id.is_empty() {
            format!("union.variant_{}", variant_index + 1)
        } else {
            format!("{}.variant_{}", trimmed_union_layout_id, variant_index + 1)
        };
        if !project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == base_layout_id)
        {
            return base_layout_id;
        }

        let mut suffix_index = 2_u64;
        loop {
            let candidate_layout_id = format!("{}_{}", base_layout_id, suffix_index);
            if !project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == candidate_layout_id)
            {
                return candidate_layout_id;
            }

            suffix_index = suffix_index.saturating_add(1);
        }
    }

    fn persist_variant_layout_draft(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        variant_draft: &SymbolLayoutEditDraft,
    ) -> bool {
        match SymbolLayoutEditorViewData::apply_draft_to_catalog(project_symbol_catalog, variant_draft) {
            Ok(updated_project_symbol_catalog) => {
                self.persist_project_symbol_catalog(updated_project_symbol_catalog);
                true
            }
            Err(error) => {
                log::error!("Failed to update union variant layout `{}`: {}.", variant_draft.layout_id, error);
                false
            }
        }
    }

    fn move_unassigned_span_up(
        draft: &mut SymbolLayoutEditDraft,
        row_context: SymbolLayoutUnassignedRowContext,
    ) -> Option<SymbolLayoutUnassignedSelection> {
        let adjacent_field = row_context.move_up_field?;
        let moved_field_offset = adjacent_field
            .offset_in_bytes
            .checked_add(row_context.size_in_bytes)?;

        Self::set_field_static_offset(draft, adjacent_field.field_position, moved_field_offset)
            .then(|| SymbolLayoutUnassignedSelection::new(adjacent_field.offset_in_bytes, row_context.size_in_bytes))
    }

    fn move_unassigned_span_down(
        draft: &mut SymbolLayoutEditDraft,
        row_context: SymbolLayoutUnassignedRowContext,
    ) -> Option<SymbolLayoutUnassignedSelection> {
        let adjacent_field = row_context.move_down_field?;
        let moved_unassigned_offset = row_context
            .offset_in_bytes
            .checked_add(adjacent_field.size_in_bytes)?;

        Self::set_field_static_offset(draft, adjacent_field.field_position, row_context.offset_in_bytes)
            .then(|| SymbolLayoutUnassignedSelection::new(moved_unassigned_offset, row_context.size_in_bytes))
    }

    fn build_unassigned_row_contexts(
        offset_in_bytes: u64,
        size_in_bytes: u64,
        split_offsets: &BTreeSet<u64>,
        move_up_field: Option<SymbolLayoutUnassignedAdjacentField>,
        move_down_field: Option<SymbolLayoutUnassignedAdjacentField>,
    ) -> Vec<SymbolLayoutUnassignedRowContext> {
        if size_in_bytes == 0 {
            return Vec::new();
        }

        let Some(end_offset_in_bytes) = offset_in_bytes.checked_add(size_in_bytes) else {
            return Vec::new();
        };
        let mut row_boundaries = Vec::new();

        row_boundaries.push(offset_in_bytes);
        row_boundaries.extend(
            split_offsets
                .iter()
                .copied()
                .filter(|split_offset| *split_offset > offset_in_bytes && *split_offset < end_offset_in_bytes),
        );
        row_boundaries.push(end_offset_in_bytes);

        row_boundaries
            .windows(2)
            .enumerate()
            .filter_map(|(segment_position, row_boundary_pair)| {
                let row_offset = *row_boundary_pair.first()?;
                let row_end = *row_boundary_pair.get(1)?;
                let row_size = row_end.checked_sub(row_offset)?;
                if row_size == 0 {
                    return None;
                }

                let previous_span = (segment_position > 0).then(|| {
                    let previous_offset = row_boundaries[segment_position - 1];
                    SymbolLayoutUnassignedSelection::new(previous_offset, row_offset.saturating_sub(previous_offset))
                });
                let next_span = (segment_position + 2 < row_boundaries.len()).then(|| {
                    let next_end = row_boundaries[segment_position + 2];
                    SymbolLayoutUnassignedSelection::new(row_end, next_end.saturating_sub(row_end))
                });
                let merge_above_span = previous_span.as_ref().map(|previous_span| {
                    SymbolLayoutUnassignedSelection::new(previous_span.get_offset_in_bytes(), previous_span.get_size_in_bytes().saturating_add(row_size))
                });
                let merge_below_span = next_span
                    .as_ref()
                    .map(|next_span| SymbolLayoutUnassignedSelection::new(row_offset, row_size.saturating_add(next_span.get_size_in_bytes())));

                Some(SymbolLayoutUnassignedRowContext {
                    offset_in_bytes: row_offset,
                    size_in_bytes: row_size,
                    move_up_field: (segment_position == 0).then_some(move_up_field).flatten(),
                    move_down_field: (segment_position + 2 == row_boundaries.len())
                        .then_some(move_down_field)
                        .flatten(),
                    move_up_unassigned_span: previous_span,
                    move_down_unassigned_span: next_span,
                    merge_above_span,
                    merge_below_span,
                })
            })
            .collect()
    }

    fn build_symbolic_field_definition_from_draft(field_draft: &SymbolLayoutFieldEditDraft) -> Result<SymbolicFieldDefinition, String> {
        let trimmed_data_type_id = field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id()
            .trim()
            .to_string();
        if trimmed_data_type_id.is_empty() {
            return Err(String::from("Field data type is required."));
        }

        Ok(SymbolicFieldDefinition::new_named_with_resolutions_and_display_count(
            field_draft.field_name.trim().to_string(),
            DataTypeRef::new(&trimmed_data_type_id),
            field_draft.container_edit.to_container_type()?,
            field_draft.container_edit.to_count_resolution()?,
            field_draft.container_edit.to_display_count_resolution()?,
            field_draft.to_offset_resolution()?,
        )
        .with_hidden(field_draft.is_hidden))
    }

    fn validate_define_field_draft(
        project_symbol_catalog: &ProjectSymbolCatalog,
        field_draft: &SymbolLayoutFieldEditDraft,
        span_offset_in_bytes: u64,
        span_size_in_bytes: u64,
    ) -> Result<(u64, u64), String> {
        if field_draft.field_name.trim().is_empty() {
            return Err(String::from("Field name is required."));
        }

        if field_draft.offset_mode != SymbolLayoutFieldOffsetMode::Static {
            return Err(String::from("Field offset must be static."));
        }

        let symbolic_field_definition = Self::build_symbolic_field_definition_from_draft(field_draft)?;
        let relative_offset_in_bytes = match symbolic_field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Static(relative_offset_in_bytes) => *relative_offset_in_bytes,
            _ => return Err(String::from("Field offset must be static.")),
        };
        let field_size_in_bytes = SymbolLayoutEditorViewData::resolve_symbolic_field_size_in_bytes(
            project_symbol_catalog,
            &symbolic_field_definition,
            &mut std::collections::HashSet::new(),
        );

        if field_size_in_bytes == 0 {
            return Err(String::from("Field has no byte size."));
        }

        let relative_field_end_in_bytes = relative_offset_in_bytes
            .checked_add(field_size_in_bytes)
            .ok_or_else(|| String::from("Field range is too large."))?;

        if relative_field_end_in_bytes > span_size_in_bytes {
            if field_size_in_bytes > span_size_in_bytes {
                return Err(format!(
                    "`{}` uses {} byte(s); selected span has {}.",
                    symbolic_field_definition.get_data_type_ref().get_data_type_id(),
                    field_size_in_bytes,
                    span_size_in_bytes
                ));
            }

            return Err(format!(
                "`{}` uses {} byte(s); choose 0 to {}.",
                symbolic_field_definition.get_data_type_ref().get_data_type_id(),
                field_size_in_bytes,
                span_size_in_bytes.saturating_sub(field_size_in_bytes)
            ));
        }

        let field_offset_in_bytes = span_offset_in_bytes
            .checked_add(relative_offset_in_bytes)
            .ok_or_else(|| String::from("Field offset is too large."))?;

        Ok((field_offset_in_bytes, field_size_in_bytes))
    }

    fn normalize_union_field_drafts(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &mut SymbolLayoutEditDraft,
    ) {
        if !draft.layout_kind.is_union() {
            return;
        }

        for field_position in 0..draft.field_drafts.len() {
            let replacement_data_type_ref = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| {
                    struct_layout_descriptor.get_struct_layout_id() != draft.layout_id
                        && struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_layout_kind()
                            == SymbolicLayoutKind::Struct
                })
                .map(|struct_layout_descriptor| DataTypeRef::new(struct_layout_descriptor.get_struct_layout_id()));

            if let Some(field_draft) = draft.field_drafts.get_mut(field_position) {
                if let Some(replacement_data_type_ref) = replacement_data_type_ref.clone() {
                    field_draft
                        .data_type_selection
                        .replace_selected_data_types(vec![replacement_data_type_ref]);
                }
                field_draft.container_edit = Default::default();
                field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Sequential;
                field_draft.static_offset_in_bytes.clear();
                field_draft.offset_resolver_id.clear();
            }
        }
    }

    fn string_data_type_ref() -> DataTypeRef {
        DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)
    }

    fn format_field_data_type_preview(field_draft: &SymbolLayoutFieldEditDraft) -> String {
        let data_type_id = field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id()
            .trim();
        let container_suffix = match field_draft.container_edit.kind {
            SymbolLayoutFieldContainerKind::Element => String::new(),
            SymbolLayoutFieldContainerKind::Array => ContainerType::Array.to_string(),
            SymbolLayoutFieldContainerKind::FixedArray => {
                let fixed_array_length = field_draft.container_edit.fixed_array_length.trim();

                if fixed_array_length.is_empty() {
                    String::from("[?]")
                } else if !field_draft
                    .container_edit
                    .display_count_resolver_id
                    .trim()
                    .is_empty()
                {
                    format!(
                        "[{}] display resolver({})",
                        fixed_array_length,
                        field_draft.container_edit.display_count_resolver_id.trim()
                    )
                } else {
                    format!("[{}]", fixed_array_length)
                }
            }
            SymbolLayoutFieldContainerKind::DynamicArray => {
                let resolver_id = field_draft
                    .container_edit
                    .dynamic_array_count_resolver_id
                    .trim();

                if resolver_id.is_empty() {
                    ContainerType::Array.to_string()
                } else {
                    format!("[resolver({})]", resolver_id)
                }
            }
            SymbolLayoutFieldContainerKind::Pointer => ContainerType::Pointer(field_draft.container_edit.pointer_size).to_string(),
            SymbolLayoutFieldContainerKind::FixedPointerArray => {
                let fixed_array_length = field_draft.container_edit.fixed_array_length.trim();

                if fixed_array_length.is_empty() {
                    format!("*({})[?]", field_draft.container_edit.pointer_size)
                } else if !field_draft
                    .container_edit
                    .display_count_resolver_id
                    .trim()
                    .is_empty()
                {
                    format!(
                        "*({})[{}] display resolver({})",
                        field_draft.container_edit.pointer_size,
                        fixed_array_length,
                        field_draft.container_edit.display_count_resolver_id.trim()
                    )
                } else {
                    format!("*({})[{}]", field_draft.container_edit.pointer_size, fixed_array_length)
                }
            }
            SymbolLayoutFieldContainerKind::DynamicPointerArray => {
                let resolver_id = field_draft
                    .container_edit
                    .dynamic_array_count_resolver_id
                    .trim();

                if resolver_id.is_empty() {
                    format!("*({})[]", field_draft.container_edit.pointer_size)
                } else {
                    format!("*({})[resolver({})]", field_draft.container_edit.pointer_size, resolver_id)
                }
            }
        };

        let visibility_suffix = if field_draft.is_hidden { " hidden" } else { "" };

        format!("{}{}{}", data_type_id, container_suffix, visibility_suffix)
    }

    fn render_flat_icon_button_at(
        &self,
        user_interface: &mut Ui,
        button_rect: Rect,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.put(
            button_rect,
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(Color32::TRANSPARENT)
                .disabled(is_disabled),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_response.rect,
            icon_handle,
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        button_response
    }

    fn render_flat_icon_button(
        &self,
        user_interface: &mut Ui,
        icon_handle: &eframe::egui::TextureHandle,
        tooltip_text: &str,
        is_disabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_response = user_interface.add_sized(
            vec2(Self::ICON_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT),
            ThemeButton::new_from_theme(theme)
                .with_tooltip_text(tooltip_text)
                .background_color(Color32::TRANSPARENT)
                .disabled(is_disabled),
        );

        IconDraw::draw_tinted(
            user_interface,
            button_response.rect,
            icon_handle,
            if is_disabled { theme.foreground_preview } else { theme.foreground },
        );

        button_response
    }

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

    fn render_string_value_box(
        &self,
        user_interface: &mut Ui,
        value: &mut String,
        preview_text: &str,
        id: &str,
        width: f32,
        height: f32,
    ) {
        let validation_data_type_ref = Self::string_data_type_ref();
        let mut value_string = AnonymousValueString::new(value.clone(), AnonymousValueStringFormat::String, ContainerType::None);

        user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut value_string,
                &validation_data_type_ref,
                false,
                true,
                preview_text,
                id,
            )
            .allowed_anonymous_value_string_formats(vec![AnonymousValueStringFormat::String])
            .show_format_button(false)
            .normalize_value_format(false)
            .use_format_text_colors(false)
            .width(width)
            .height(height),
        );

        *value = value_string.get_anonymous_value_string().to_string();
    }

    fn render_u64_data_value_box(
        &self,
        user_interface: &mut Ui,
        value: &mut String,
        value_format: &mut AnonymousValueStringFormat,
        preview_text: &str,
        id: &str,
        width: f32,
        height: f32,
    ) {
        let validation_data_type_ref = DataTypeRef::new(DataTypeU64::DATA_TYPE_ID);
        let mut value_string = AnonymousValueString::new(value.clone(), *value_format, ContainerType::None);

        user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut value_string,
                &validation_data_type_ref,
                false,
                true,
                preview_text,
                id,
            )
            .allowed_anonymous_value_string_formats(vec![
                AnonymousValueStringFormat::Binary,
                AnonymousValueStringFormat::Decimal,
                AnonymousValueStringFormat::Hexadecimal,
            ])
            .show_format_button(true)
            .normalize_value_format(false)
            .use_format_text_colors(true)
            .width(width)
            .height(height),
        );

        *value = value_string.get_anonymous_value_string().to_string();
        *value_format = value_string.get_anonymous_value_string_format();
    }

    fn render_define_field_container_selector(
        &self,
        user_interface: &mut Ui,
        container_edit: &mut SymbolLayoutFieldContainerEdit,
        menu_id: &str,
        width: f32,
    ) {
        let mut selected_container_edit = None;
        let current_label = Self::define_field_container_label(container_edit);

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                current_label,
                menu_id,
                None,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    let value_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), "Value", None, width));

                    if value_response.clicked() {
                        selected_container_edit = Some(SymbolLayoutFieldContainerEdit::default());
                        *should_close = true;
                    }

                    popup_user_interface.separator();

                    for pointer_size in PointerScanPointerSize::ALL {
                        let pointer_label = format!("Ptr {}", pointer_size);
                        let pointer_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), &pointer_label, None, width));

                        if pointer_response.clicked() {
                            selected_container_edit = Some(SymbolLayoutFieldContainerEdit {
                                kind: SymbolLayoutFieldContainerKind::Pointer,
                                pointer_size,
                                ..SymbolLayoutFieldContainerEdit::default()
                            });
                            *should_close = true;
                        }
                    }
                },
            )
            .width(width)
            .height(Self::TOOLBAR_HEIGHT),
        );

        if let Some(selected_container_edit) = selected_container_edit {
            *container_edit = selected_container_edit;
        }
    }

    fn render_define_field_type_combo(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        field_draft: &mut SymbolLayoutFieldEditDraft,
        menu_id: &str,
        width: f32,
    ) {
        let type_options = Self::build_field_type_options(project_symbol_catalog);
        let selected_data_type_id = field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id()
            .to_string();
        let selected_type_option = type_options
            .iter()
            .find(|type_option| type_option.data_type_ref.get_data_type_id() == selected_data_type_id.as_str());
        let combo_label = selected_type_option
            .map(|type_option| type_option.label.clone())
            .unwrap_or_else(|| DataTypeToStringConverter::convert_data_type_to_string(&selected_data_type_id));
        let combo_icon = selected_type_option.and_then(|type_option| {
            (type_option.kind == SymbolLayoutFieldTypeOptionKind::BuiltIn)
                .then(|| DataTypeToIconConverter::convert_data_type_to_icon(type_option.data_type_ref.get_data_type_id(), &self.app_context.theme.icon_library))
        });
        let search_storage_id = Self::define_field_type_search_storage_id(menu_id);
        let popup_width = Self::define_field_type_popup_width(width);
        let built_in_type_item_width = Self::define_field_builtin_type_item_width(popup_width);

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                combo_label,
                menu_id,
                combo_icon,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    let mut search_text = popup_user_interface
                        .ctx()
                        .data_mut(|data| data.get_temp::<String>(search_storage_id).unwrap_or_default());

                    popup_user_interface.add_space(4.0);
                    let search_box_id = format!("symbol_layout_define_field_type_search_{}", menu_id);
                    popup_user_interface.add(
                        SearchBoxView::new(self.app_context.clone(), &mut search_text, "Search types", &search_box_id)
                            .width((popup_width - 8.0).max(1.0))
                            .height(Self::TOOLBAR_HEIGHT),
                    );
                    popup_user_interface.add_space(4.0);
                    popup_user_interface
                        .ctx()
                        .data_mut(|data| data.insert_temp(search_storage_id, search_text.clone()));

                    let filtered_type_options = Self::filter_field_type_options(&type_options, &search_text);

                    if filtered_type_options.is_empty() {
                        popup_user_interface.label(RichText::new("No matching types").color(self.app_context.theme.foreground_preview));
                        return;
                    }

                    let (built_in_type_options, symbol_layout_type_options): (Vec<_>, Vec<_>) = filtered_type_options
                        .into_iter()
                        .partition(|type_option| type_option.kind == SymbolLayoutFieldTypeOptionKind::BuiltIn);

                    ScrollArea::vertical()
                        .max_height(240.0)
                        .auto_shrink([false, false])
                        .show(popup_user_interface, |scroll_user_interface| {
                            if !built_in_type_options.is_empty() {
                                Grid::new(Id::new(("symbol_layout_define_field_builtin_type_grid", menu_id)))
                                    .spacing(vec2(Self::DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_SPACING, 0.0))
                                    .min_col_width(Self::DEFINE_FIELD_BUILT_IN_TYPE_ITEM_WIDTH)
                                    .show(scroll_user_interface, |grid_user_interface| {
                                        for (type_option_position, type_option) in built_in_type_options.iter().enumerate() {
                                            let data_type_id = type_option.data_type_ref.get_data_type_id();
                                            let row_icon = Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                                data_type_id,
                                                &self.app_context.theme.icon_library,
                                            ));
                                            let item_response = grid_user_interface.add(ComboBoxItemView::new(
                                                self.app_context.clone(),
                                                &type_option.label,
                                                row_icon,
                                                built_in_type_item_width,
                                            ));

                                            if item_response.clicked() {
                                                field_draft
                                                    .data_type_selection
                                                    .select_single_data_type(type_option.data_type_ref.clone());
                                                grid_user_interface
                                                    .ctx()
                                                    .data_mut(|data| data.insert_temp(search_storage_id, String::new()));
                                                *should_close = true;
                                            }

                                            if (type_option_position + 1) % Self::DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT == 0 {
                                                grid_user_interface.end_row();
                                            }
                                        }

                                        if built_in_type_options.len() % Self::DEFINE_FIELD_BUILT_IN_TYPE_COLUMN_COUNT != 0 {
                                            grid_user_interface.end_row();
                                        }
                                    });
                            }

                            if !built_in_type_options.is_empty() && !symbol_layout_type_options.is_empty() {
                                scroll_user_interface.separator();
                            }

                            for type_option in symbol_layout_type_options {
                                let item_response =
                                    scroll_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), &type_option.label, None, popup_width));

                                if item_response.clicked() {
                                    field_draft
                                        .data_type_selection
                                        .select_single_data_type(type_option.data_type_ref);
                                    scroll_user_interface
                                        .ctx()
                                        .data_mut(|data| data.insert_temp(search_storage_id, String::new()));
                                    *should_close = true;
                                }
                            }
                        });
                },
            )
            .width(width)
            .popup_width(popup_width)
            .height(Self::TOOLBAR_HEIGHT),
        );
    }

    fn render_layout_kind_combo(
        &self,
        user_interface: &mut Ui,
        layout_kind: &mut SymbolicLayoutKind,
        menu_id: &str,
    ) {
        let mut selected_layout_kind = None;
        let combo_width = Self::LAYOUT_KIND_COMBO_WIDTH.min(user_interface.available_width().max(1.0));

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                layout_kind.label(),
                menu_id,
                None,
                |popup_user_interface, should_close| {
                    for candidate_layout_kind in SymbolicLayoutKind::ALL {
                        let item_response = popup_user_interface.add(ComboBoxItemView::new(
                            self.app_context.clone(),
                            candidate_layout_kind.label(),
                            None,
                            combo_width,
                        ));

                        if item_response.clicked() {
                            selected_layout_kind = Some(candidate_layout_kind);
                            *should_close = true;
                        }
                    }
                },
            )
            .width(combo_width)
            .popup_width(combo_width)
            .height(Self::FIELD_ROW_HEIGHT),
        );

        if let Some(selected_layout_kind) = selected_layout_kind {
            *layout_kind = selected_layout_kind;
        }
    }

    fn layout_kind_from_label(label: &str) -> Option<SymbolicLayoutKind> {
        SymbolicLayoutKind::ALL
            .iter()
            .copied()
            .find(|layout_kind| layout_kind.label() == label.trim())
    }

    fn clear_struct_viewer_if_symbol_layout_focused(&self) {
        let is_symbol_layout_focused = self
            .struct_viewer_view_data
            .read("SymbolLayoutEditor check details focus")
            .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned())
            .is_some_and(|focus_target| matches!(focus_target, StructViewerFocusTarget::SymbolLayoutEditor { .. }));

        if is_symbol_layout_focused {
            StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
        }
    }

    fn focus_selected_layout_in_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_layout_id: Option<&str>,
    ) {
        let Some(selected_layout_id) = selected_layout_id else {
            self.clear_struct_viewer_if_symbol_layout_focused();
            return;
        };
        let Some(struct_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == selected_layout_id)
        else {
            self.clear_struct_viewer_if_symbol_layout_focused();
            return;
        };

        let details_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string(struct_layout_descriptor.get_struct_layout_id())
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_LAYOUT_ID.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(
                struct_layout_descriptor
                    .get_struct_layout_definition()
                    .get_layout_kind()
                    .label(),
            )
            .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_KIND.to_string(), false),
        ]);
        let selection_key = format!("layout|{}", struct_layout_descriptor.get_struct_layout_id());
        let edit_callback = Self::build_struct_viewer_layout_edit_callback(
            self.app_context.clone(),
            self.struct_viewer_view_data.clone(),
            struct_layout_descriptor.get_struct_layout_id().to_string(),
        );

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_struct,
            edit_callback,
            Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
        );
    }

    fn build_struct_viewer_layout_edit_callback(
        app_context: Arc<AppContext>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        layout_id: String,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        Arc::new(move |edited_field: ValuedStructField| {
            if edited_field.get_name() != StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_KIND {
                return;
            }

            let edited_text = StructViewerViewData::read_utf8_field_text(&edited_field);
            let Some(edited_layout_kind) = Self::layout_kind_from_label(&edited_text) else {
                return;
            };
            let Some(project_symbol_catalog) = Self::get_opened_project_symbol_catalog_from_context(&app_context) else {
                return;
            };

            let mut did_update_layout = false;
            let updated_struct_layout_descriptors = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .map(|struct_layout_descriptor| {
                    if struct_layout_descriptor.get_struct_layout_id() != layout_id {
                        return struct_layout_descriptor.clone();
                    }

                    let struct_layout_definition = struct_layout_descriptor.get_struct_layout_definition();
                    if struct_layout_definition.get_layout_kind() == edited_layout_kind {
                        return struct_layout_descriptor.clone();
                    }

                    did_update_layout = true;
                    squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor::new(
                        struct_layout_descriptor.get_struct_layout_id().to_string(),
                        SymbolicStructDefinition::new_with_layout_kind(
                            struct_layout_definition.get_symbol_namespace().to_string(),
                            edited_layout_kind,
                            struct_layout_definition.get_fields().to_vec(),
                        ),
                    )
                })
                .collect::<Vec<_>>();

            if !did_update_layout {
                return;
            }

            let mut updated_project_symbol_catalog = project_symbol_catalog;
            updated_project_symbol_catalog.set_struct_layout_descriptors(updated_struct_layout_descriptors);
            Self::persist_project_symbol_catalog_with_context(&app_context, updated_project_symbol_catalog.clone());

            let Some(updated_struct_layout_descriptor) = updated_project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
            else {
                return;
            };
            let details_struct = ValuedStruct::new_anonymous(vec![
                DataTypeStringUtf8::get_value_from_primitive_string(updated_struct_layout_descriptor.get_struct_layout_id())
                    .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_LAYOUT_ID.to_string(), false),
                DataTypeStringUtf8::get_value_from_primitive_string(
                    updated_struct_layout_descriptor
                        .get_struct_layout_definition()
                        .get_layout_kind()
                        .label(),
                )
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_KIND.to_string(), false),
            ]);
            let selection_key = format!("layout|{}", updated_struct_layout_descriptor.get_struct_layout_id());
            let edit_callback = Self::build_struct_viewer_layout_edit_callback(app_context.clone(), struct_viewer_view_data.clone(), layout_id.clone());

            StructViewerViewData::focus_valued_struct_with_focus_target(
                struct_viewer_view_data.clone(),
                app_context.engine_unprivileged_state.clone(),
                details_struct,
                edit_callback,
                Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
            );
        })
    }

    fn focus_field_in_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
        field_index: usize,
    ) {
        let Some(field_draft) = draft.field_drafts.get(field_index) else {
            self.clear_struct_viewer_if_symbol_layout_focused();
            return;
        };

        let details_struct = Self::build_field_details_struct(project_symbol_catalog, draft.layout_kind, field_draft);
        let selection_key = format!("field|{}|{}", draft.layout_id, field_index);
        let edit_callback = Self::build_struct_viewer_field_edit_callback(
            self.symbol_layout_editor_view_data.clone(),
            self.struct_viewer_view_data.clone(),
            self.app_context.clone(),
            field_index,
        );

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_struct,
            edit_callback,
            Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
        );
    }

    fn focus_unassigned_span_in_struct_viewer(
        &self,
        draft: &SymbolLayoutEditDraft,
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) {
        let details_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("UNASSIGNED").to_named_valued_struct_field(String::from("kind"), true),
            DataTypeStringUtf8::get_value_from_primitive_string(&draft.layout_id).to_named_valued_struct_field(String::from("layout"), true),
            DataTypeU64::get_value_from_primitive(offset_in_bytes).to_named_valued_struct_field(String::from("offset"), true),
            DataTypeU64::get_value_from_primitive(size_in_bytes).to_named_valued_struct_field(String::from("size"), true),
        ]);
        let selection_key = format!("unassigned|{}|{}|{}", draft.layout_id, offset_in_bytes, size_in_bytes);

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_struct,
            Arc::new(|_edited_field| {}),
            Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
        );
    }

    fn build_field_details_struct(
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_kind: SymbolicLayoutKind,
        field_draft: &SymbolLayoutFieldEditDraft,
    ) -> ValuedStruct {
        let element_type = SymbolLayoutEditorViewData::resolve_field_element_type(project_symbol_catalog, field_draft);
        if layout_kind.is_union() {
            return ValuedStruct::new_anonymous(vec![
                DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.field_name)
                    .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_NAME.to_string(), false),
                DataTypeStringUtf8::get_value_from_primitive_string(
                    field_draft
                        .data_type_selection
                        .visible_data_type()
                        .get_data_type_id(),
                )
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_SYMBOL_LAYOUT.to_string(), false),
            ]);
        }

        let mut fields = vec![
            DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.field_name)
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_NAME.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(element_type.label())
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_ELEMENT_TYPE.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(field_draft.container_edit.kind.label())
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_CONTAINER_KIND.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(if field_draft.is_hidden { "true" } else { "false" })
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_HIDDEN.to_string(), false),
        ];

        let element_type_field_name = match element_type {
            SymbolLayoutFieldElementType::BuiltInDataType => StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_DATA_TYPE,
            SymbolLayoutFieldElementType::SymbolLayout => StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_SYMBOL_LAYOUT,
        };
        fields.insert(
            2,
            DataTypeStringUtf8::get_value_from_primitive_string(
                field_draft
                    .data_type_selection
                    .visible_data_type()
                    .get_data_type_id(),
            )
            .to_named_valued_struct_field(element_type_field_name.to_string(), false),
        );

        if matches!(
            field_draft.container_edit.kind,
            SymbolLayoutFieldContainerKind::FixedArray | SymbolLayoutFieldContainerKind::FixedPointerArray
        ) {
            let length = field_draft
                .container_edit
                .fixed_array_length
                .trim()
                .parse::<u64>()
                .unwrap_or(1);
            fields.push(
                DataTypeU64::get_value_from_primitive(length)
                    .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_FIXED_ARRAY_LENGTH.to_string(), false),
            );
            fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.container_edit.display_count_resolver_id).to_named_valued_struct_field(
                    StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_DISPLAY_COUNT_RESOLVER.to_string(),
                    false,
                ),
            );
        }

        if matches!(
            field_draft.container_edit.kind,
            SymbolLayoutFieldContainerKind::DynamicArray | SymbolLayoutFieldContainerKind::DynamicPointerArray
        ) {
            fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.container_edit.dynamic_array_count_resolver_id)
                    .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_COUNT_RESOLVER.to_string(), false),
            );
            fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.container_edit.display_count_resolver_id).to_named_valued_struct_field(
                    StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_DISPLAY_COUNT_RESOLVER.to_string(),
                    false,
                ),
            );
        }

        if matches!(
            field_draft.container_edit.kind,
            SymbolLayoutFieldContainerKind::Pointer | SymbolLayoutFieldContainerKind::FixedPointerArray | SymbolLayoutFieldContainerKind::DynamicPointerArray
        ) {
            fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.container_edit.pointer_size.to_string())
                    .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_POINTER_SIZE.to_string(), false),
            );
        }

        fields.push(
            DataTypeStringUtf8::get_value_from_primitive_string(field_draft.offset_mode.label())
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_OFFSET_MODE.to_string(), false),
        );
        if field_draft.offset_mode == SymbolLayoutFieldOffsetMode::Static {
            let offset_in_bytes = SymbolLayoutFieldEditDraft::parse_static_offset_text(&field_draft.static_offset_in_bytes).unwrap_or(0);
            fields.push(
                DataTypeU64::get_value_from_primitive(offset_in_bytes)
                    .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_STATIC_OFFSET.to_string(), false),
            );
        }
        if field_draft.offset_mode == SymbolLayoutFieldOffsetMode::Resolver {
            fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.offset_resolver_id)
                    .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_OFFSET_RESOLVER.to_string(), false),
            );
        }

        ValuedStruct::new_anonymous(fields)
    }

    fn build_struct_viewer_field_edit_callback(
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        app_context: Arc<AppContext>,
        field_index: usize,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        Arc::new(move |edited_field: ValuedStructField| {
            let updated_draft = {
                let Some(mut view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor apply field details edit") else {
                    return;
                };
                let Some(mut draft) = view_data.get_draft().cloned() else {
                    return;
                };
                let Some(field_draft) = draft.field_drafts.get_mut(field_index) else {
                    return;
                };

                let project_symbol_catalog = Self::get_opened_project_symbol_catalog_from_context(&app_context).unwrap_or_default();
                Self::apply_field_details_edit(&project_symbol_catalog, field_draft, &edited_field);
                view_data.replace_draft(draft.clone());
                draft
            };

            let Some(updated_field_draft) = updated_draft.field_drafts.get(field_index) else {
                return;
            };
            let project_symbol_catalog = Self::get_opened_project_symbol_catalog_from_context(&app_context).unwrap_or_default();
            let details_struct = Self::build_field_details_struct(&project_symbol_catalog, updated_draft.layout_kind, updated_field_draft);
            let selection_key = format!("field|{}|{}", updated_draft.layout_id, field_index);
            let edit_callback = Self::build_struct_viewer_field_edit_callback(
                symbol_layout_editor_view_data.clone(),
                struct_viewer_view_data.clone(),
                app_context.clone(),
                field_index,
            );

            StructViewerViewData::focus_valued_struct_with_focus_target(
                struct_viewer_view_data.clone(),
                app_context.engine_unprivileged_state.clone(),
                details_struct,
                edit_callback,
                Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
            );
        })
    }

    fn apply_field_details_edit(
        project_symbol_catalog: &ProjectSymbolCatalog,
        field_draft: &mut SymbolLayoutFieldEditDraft,
        edited_field: &ValuedStructField,
    ) {
        let edited_text = StructViewerViewData::read_utf8_field_text(edited_field);

        match edited_field.get_name() {
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_NAME => {
                field_draft.field_name = edited_text;
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_ELEMENT_TYPE => {
                Self::apply_field_element_type_edit(project_symbol_catalog, field_draft, &edited_text);
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_DATA_TYPE => {
                field_draft
                    .data_type_selection
                    .replace_selected_data_types(vec![DataTypeRef::new(edited_text.trim())]);
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_SYMBOL_LAYOUT => {
                field_draft
                    .data_type_selection
                    .replace_selected_data_types(vec![DataTypeRef::new(edited_text.trim())]);
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_CONTAINER_KIND => {
                if let Some(container_kind) = Self::container_kind_from_label(&edited_text) {
                    field_draft.container_edit.kind = container_kind;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_HIDDEN => {
                field_draft.is_hidden = Self::parse_bool_text(&edited_text).unwrap_or(field_draft.is_hidden);
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_FIXED_ARRAY_LENGTH => {
                if let Some(length) = Self::read_u64_field_value(edited_field) {
                    field_draft.container_edit.fixed_array_length = length.max(1).to_string();
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_COUNT_RESOLVER => {
                field_draft.container_edit.dynamic_array_count_resolver_id = edited_text;
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_DISPLAY_COUNT_RESOLVER => {
                field_draft.container_edit.display_count_resolver_id = edited_text;
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_POINTER_SIZE => {
                if let Ok(pointer_size) = PointerScanPointerSize::from_str(edited_text.trim()) {
                    field_draft.container_edit.pointer_size = pointer_size;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_OFFSET_MODE => {
                if let Some(offset_mode) = Self::offset_mode_from_label(&edited_text) {
                    field_draft.offset_mode = offset_mode;
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_STATIC_OFFSET => {
                if let Some(offset_in_bytes) = Self::read_u64_field_value(edited_field) {
                    field_draft.static_offset_in_bytes = offset_in_bytes.to_string();
                }
            }
            StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_OFFSET_RESOLVER => {
                field_draft.offset_resolver_id = edited_text;
            }
            _ => {}
        }
    }

    fn apply_field_element_type_edit(
        project_symbol_catalog: &ProjectSymbolCatalog,
        field_draft: &mut SymbolLayoutFieldEditDraft,
        edited_text: &str,
    ) {
        let current_element_type = SymbolLayoutEditorViewData::resolve_field_element_type(project_symbol_catalog, field_draft);
        let selected_element_type = SymbolLayoutFieldElementType::ALL
            .iter()
            .copied()
            .find(|element_type| element_type.label() == edited_text.trim())
            .unwrap_or(current_element_type);

        if selected_element_type == current_element_type {
            return;
        }

        let next_data_type_ref = match selected_element_type {
            SymbolLayoutFieldElementType::BuiltInDataType => Some(DataTypeRef::new(DataTypeI32::DATA_TYPE_ID)),
            SymbolLayoutFieldElementType::SymbolLayout => {
                SymbolLayoutEditorViewData::first_symbol_layout_id(project_symbol_catalog).map(|struct_layout_id| DataTypeRef::new(&struct_layout_id))
            }
        };

        if let Some(next_data_type_ref) = next_data_type_ref {
            field_draft
                .data_type_selection
                .replace_selected_data_types(vec![next_data_type_ref]);
        }
    }

    fn container_kind_from_label(label: &str) -> Option<SymbolLayoutFieldContainerKind> {
        SymbolLayoutFieldContainerKind::ALL
            .iter()
            .copied()
            .find(|container_kind| container_kind.label() == label)
    }

    fn offset_mode_from_label(label: &str) -> Option<SymbolLayoutFieldOffsetMode> {
        SymbolLayoutFieldOffsetMode::ALL
            .iter()
            .copied()
            .find(|offset_mode| offset_mode.label() == label)
    }

    fn parse_bool_text(text: &str) -> Option<bool> {
        match text.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" | "hidden" => Some(true),
            "false" | "no" | "0" | "visible" => Some(false),
            _ => None,
        }
    }

    fn read_u64_field_value(valued_struct_field: &ValuedStructField) -> Option<u64> {
        let value_bytes = valued_struct_field.get_data_value()?.get_value_bytes();
        let value_bytes: [u8; 8] = value_bytes.as_slice().try_into().ok()?;

        Some(u64::from_le_bytes(value_bytes))
    }

    fn measure_text_width(
        user_interface: &Ui,
        text: &str,
        font_id: &eframe::egui::FontId,
        text_color: Color32,
    ) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        user_interface.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.to_string(), font_id.clone(), text_color)
                .size()
                .x
        })
    }

    fn truncate_text_to_width(
        user_interface: &Ui,
        text: &str,
        max_text_width: f32,
        font_id: &eframe::egui::FontId,
        text_color: Color32,
    ) -> String {
        if text.is_empty() || max_text_width <= 0.0 {
            return String::new();
        }

        let full_text_width = Self::measure_text_width(user_interface, text, font_id, text_color);
        if full_text_width <= max_text_width {
            return text.to_string();
        }

        let ellipsis = "...";
        let ellipsis_width = Self::measure_text_width(user_interface, ellipsis, font_id, text_color);
        if ellipsis_width > max_text_width {
            return String::new();
        }

        let mut truncated_text = text.to_string();
        while !truncated_text.is_empty() {
            truncated_text.pop();
            let candidate_text = format!("{}{}", truncated_text, ellipsis);
            let candidate_width = Self::measure_text_width(user_interface, &candidate_text, font_id, text_color);
            if candidate_width <= max_text_width {
                return candidate_text;
            }
        }

        String::new()
    }

    fn render_symbol_layout_row(
        &self,
        user_interface: &mut Ui,
        layout_id: &str,
        layout_kind: SymbolicLayoutKind,
        field_count: usize,
        usage_count: usize,
        is_selected: bool,
    ) -> Option<SymbolLayoutRowAction> {
        let theme = &self.app_context.theme;
        let (row_rect, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::LIST_ROW_HEIGHT), Sense::click());
        let mut row_action = None;

        if is_selected {
            user_interface
                .painter()
                .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface
                .painter()
                .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
        }

        StateLayer {
            bounds_min: row_rect.min,
            bounds_max: row_rect.max,
            enabled: true,
            pressed: row_response.is_pointer_button_down_on(),
            has_hover: row_response.hovered(),
            has_focus: false,
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        user_interface.painter().text(
            pos2(row_rect.min.x + 8.0, row_rect.center().y),
            Align2::LEFT_CENTER,
            layout_id,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(row_rect)
                .layout(Layout::right_to_left(Align::Center)),
        );
        row_user_interface.set_clip_rect(row_rect);

        let rename_response = self.render_flat_icon_button(
            &mut row_user_interface,
            &theme.icon_library.icon_handle_common_edit,
            "Rename this symbol layout.",
            false,
        );
        if rename_response.clicked() {
            row_action = Some(SymbolLayoutRowAction::Rename);
        }

        row_user_interface.add_space(Self::FIELD_INPUT_SPACING);
        let entry_count_label = if layout_kind.is_union() { "variants" } else { "fields" };
        row_user_interface.label(
            RichText::new(format!(
                "{} | {} {} | {} uses",
                layout_kind.label(),
                field_count,
                entry_count_label,
                usage_count
            ))
            .color(if is_selected { theme.foreground } else { theme.foreground_preview }),
        );

        if row_response.double_clicked() && row_action.is_none() {
            row_action = Some(SymbolLayoutRowAction::Open);
        } else if row_response.clicked() && row_action.is_none() {
            row_action = Some(SymbolLayoutRowAction::Select);
        }

        row_action
    }

    fn render_list_header(
        &self,
        user_interface: &mut Ui,
    ) {
        let theme = &self.app_context.theme;
        let (header_rect, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::LIST_ROW_HEIGHT), Sense::hover());

        user_interface
            .painter()
            .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);
        user_interface.painter().text(
            pos2(header_rect.min.x + 8.0, header_rect.center().y),
            Align2::LEFT_CENTER,
            "Symbol Layout",
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground_preview,
        );
        user_interface.painter().text(
            pos2(header_rect.max.x - 8.0, header_rect.center().y),
            Align2::RIGHT_CENTER,
            "Kind | Entries | Uses",
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground_preview,
        );
    }

    fn render_list_toolbar(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        is_take_over_active: bool,
    ) {
        let theme = &self.app_context.theme;
        let (toolbar_rect, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::TOOLBAR_HEIGHT), Sense::empty());

        user_interface
            .painter()
            .rect_filled(toolbar_rect, CornerRadius::ZERO, theme.background_primary);

        let mut toolbar_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(toolbar_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        toolbar_user_interface.set_clip_rect(toolbar_rect);

        let new_layout_response = self.render_flat_icon_button(
            &mut toolbar_user_interface,
            &theme.icon_library.icon_handle_common_add,
            "Create a new reusable symbol layout.",
            is_take_over_active,
        );
        if new_layout_response.clicked() {
            SymbolLayoutEditorViewData::begin_create_symbol_layout(
                self.symbol_layout_editor_view_data.clone(),
                project_symbol_catalog,
                self.default_data_type_ref(),
            );
        }
    }

    fn render_filter_text_box(
        &self,
        user_interface: &mut Ui,
        filter_text: &str,
    ) {
        let mut edited_filter_text = filter_text.to_string();
        user_interface.add(
            SearchBoxView::new(
                self.app_context.clone(),
                &mut edited_filter_text,
                "Filter symbol layouts...",
                "symbol_layout_editor_filter_text",
            )
            .width(user_interface.available_width())
            .height(Self::FIELD_ROW_HEIGHT),
        );
        if edited_filter_text != filter_text {
            SymbolLayoutEditorViewData::set_filter_text(self.symbol_layout_editor_view_data.clone(), edited_filter_text);
        }
    }

    fn render_list_panel(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_layout_id: Option<&str>,
        filter_text: &str,
        is_take_over_active: bool,
    ) {
        self.render_list_toolbar(user_interface, project_symbol_catalog, is_take_over_active);

        self.render_filter_text_box(user_interface, filter_text);

        self.render_list_header(user_interface);
        ScrollArea::vertical()
            .id_salt("symbol_layout_editor_layout_list")
            .auto_shrink([false, false])
            .show(user_interface, |user_interface| {
                for struct_layout_descriptor in project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .filter(|struct_layout_descriptor| SymbolLayoutEditorViewData::layout_matches_filter(struct_layout_descriptor, filter_text))
                {
                    let struct_layout_id = struct_layout_descriptor.get_struct_layout_id();
                    let usage_count = SymbolLayoutEditorViewData::count_symbol_claim_usages(project_symbol_catalog, struct_layout_id);
                    let field_count = struct_layout_descriptor
                        .get_struct_layout_definition()
                        .get_fields()
                        .len();
                    let row_action = self.render_symbol_layout_row(
                        user_interface,
                        struct_layout_id,
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_layout_kind(),
                        field_count,
                        usage_count,
                        selected_layout_id == Some(struct_layout_id),
                    );
                    match row_action {
                        Some(SymbolLayoutRowAction::Select) => {
                            SymbolLayoutEditorViewData::select_symbol_layout(self.symbol_layout_editor_view_data.clone(), Some(struct_layout_id.to_string()));
                            self.focus_selected_layout_in_struct_viewer(project_symbol_catalog, Some(struct_layout_id));
                        }
                        Some(SymbolLayoutRowAction::Open) if !is_take_over_active => {
                            SymbolLayoutEditorViewData::begin_open_symbol_layout(
                                self.symbol_layout_editor_view_data.clone(),
                                project_symbol_catalog,
                                struct_layout_id,
                            );
                        }
                        Some(SymbolLayoutRowAction::Rename) if !is_take_over_active => {
                            SymbolLayoutEditorViewData::begin_rename_symbol_layout(
                                self.symbol_layout_editor_view_data.clone(),
                                project_symbol_catalog,
                                struct_layout_id,
                            );
                        }
                        _ => {}
                    }
                }

                if project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .is_empty()
                {
                    user_interface.label(RichText::new("No symbol layouts yet.").color(self.app_context.theme.foreground_preview));
                }
            });
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

    fn render_field_editor_section(
        &self,
        user_interface: &mut Ui,
        layout_kind: SymbolicLayoutKind,
        field_draft: &mut SymbolLayoutFieldEditDraft,
        field_index: usize,
        is_selected: bool,
        can_move_up: bool,
        can_move_down: bool,
    ) -> Option<SymbolLayoutFieldRowAction> {
        let theme = &self.app_context.theme;
        let mut pending_field_row_action = None;

        let (row_rect, row_response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), Self::LIST_ROW_HEIGHT), Sense::click());
        if is_selected {
            user_interface
                .painter()
                .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface
                .painter()
                .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
        }

        StateLayer {
            bounds_min: row_rect.min,
            bounds_max: row_rect.max,
            enabled: true,
            pressed: row_response.is_pointer_button_down_on(),
            has_hover: row_response.hovered(),
            has_focus: row_response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        let button_area_width = Self::ICON_BUTTON_WIDTH * 2.0;
        let button_area_left = (row_rect.max.x - button_area_width).max(row_rect.min.x);
        let mut button_min_x = button_area_left;
        let mut render_next_button = |icon_handle: &eframe::egui::TextureHandle, tooltip_text: &str, is_disabled: bool| -> Response {
            let button_rect = Rect::from_min_size(pos2(button_min_x, row_rect.min.y), vec2(Self::ICON_BUTTON_WIDTH, Self::LIST_ROW_HEIGHT));
            button_min_x += Self::ICON_BUTTON_WIDTH;

            self.render_flat_icon_button_at(user_interface, button_rect, icon_handle, tooltip_text, is_disabled)
        };

        let entry_name = if layout_kind.is_union() { "variant" } else { "field" };
        let move_up_response = render_next_button(
            &theme.icon_library.icon_handle_navigation_up_arrow_small,
            &format!("Move this {} up.", entry_name),
            !can_move_up,
        );
        if move_up_response.clicked() {
            pending_field_row_action = Some(SymbolLayoutFieldRowAction::MoveUp);
        }

        let move_down_response = render_next_button(
            &theme.icon_library.icon_handle_navigation_down_arrow_small,
            &format!("Move this {} down.", entry_name),
            !can_move_down,
        );
        if move_down_response.clicked() {
            pending_field_row_action = Some(SymbolLayoutFieldRowAction::MoveDown);
        }

        if row_response.secondary_clicked() {
            let context_menu_position = row_response
                .interact_pointer_pos()
                .unwrap_or_else(|| row_rect.left_bottom());
            SymbolLayoutEditorViewData::show_field_context_menu(self.symbol_layout_editor_view_data.clone(), field_index, context_menu_position);
        }

        if row_response.clicked() {
            SymbolLayoutEditorViewData::hide_field_context_menu(self.symbol_layout_editor_view_data.clone());
        }

        let trimmed_field_name = field_draft.field_name.trim();
        let field_name = if trimmed_field_name.is_empty() {
            String::from("<unnamed>")
        } else {
            trimmed_field_name.to_string()
        };
        let data_type_ref = field_draft.data_type_selection.visible_data_type();
        let data_type_icon = DataTypeToIconConverter::convert_data_type_to_icon(data_type_ref.get_data_type_id(), &theme.icon_library);
        let icon_size = vec2(Self::FIELD_ROW_ICON_SIZE, Self::FIELD_ROW_ICON_SIZE);
        let icon_rect = Rect::from_min_size(
            pos2(row_rect.min.x + Self::FIELD_ROW_LEFT_PADDING, row_rect.center().y - icon_size.y * 0.5),
            icon_size,
        );
        IconDraw::draw_sized_tinted(user_interface, icon_rect.center(), icon_size, &data_type_icon, Color32::WHITE);

        let preview_text = Self::format_field_data_type_preview(field_draft);
        let preview_right = button_area_left - Self::FIELD_ROW_LEFT_PADDING;
        let label_position = pos2(icon_rect.max.x + Self::FIELD_ROW_ICON_GAP, row_rect.center().y);
        let label_max_width = (preview_right - label_position.x).max(0.0);
        let label_text = Self::truncate_text_to_width(
            user_interface,
            &field_name,
            label_max_width,
            &theme.font_library.font_noto_sans.font_normal,
            theme.foreground,
        );
        let label_width = Self::measure_text_width(user_interface, &label_text, &theme.font_library.font_noto_sans.font_normal, theme.foreground);
        let preview_max_width = (preview_right - label_position.x - label_width - Self::FIELD_ROW_PREVIEW_GAP).max(0.0);
        let preview_text = Self::truncate_text_to_width(
            user_interface,
            &preview_text,
            preview_max_width,
            &theme.font_library.font_noto_sans.font_small,
            theme.foreground_preview,
        );
        user_interface.painter().text(
            label_position,
            Align2::LEFT_CENTER,
            label_text,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        if !preview_text.is_empty() {
            user_interface.painter().text(
                pos2(preview_right, row_rect.center().y),
                Align2::RIGHT_CENTER,
                preview_text,
                theme.font_library.font_noto_sans.font_small.clone(),
                theme.foreground_preview,
            );
        }

        if row_response.clicked() && pending_field_row_action.is_none() {
            pending_field_row_action = Some(SymbolLayoutFieldRowAction::SelectField);
        }

        pending_field_row_action
    }

    fn render_add_entry_button(
        &self,
        user_interface: &mut Ui,
        tooltip_text: &str,
    ) -> bool {
        user_interface
            .horizontal(|user_interface| {
                self.render_flat_icon_button(user_interface, &self.app_context.theme.icon_library.icon_handle_common_add, tooltip_text, false)
                    .clicked()
            })
            .inner
    }

    fn render_union_variant_preview_row(
        &self,
        user_interface: &mut Ui,
        label_text: &str,
        preview_text: &str,
    ) {
        let theme = &self.app_context.theme;
        let (row_rect, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), Self::LIST_ROW_HEIGHT), Sense::hover());
        let label_position = pos2(row_rect.min.x + Self::FIELD_ROW_LEFT_PADDING, row_rect.center().y);
        let preview_position = pos2(row_rect.max.x - Self::FIELD_ROW_LEFT_PADDING, row_rect.center().y);

        user_interface.painter().text(
            label_position,
            Align2::LEFT_CENTER,
            Self::truncate_text_to_width(
                user_interface,
                label_text,
                (row_rect.width() * 0.6).max(0.0),
                &theme.font_library.font_noto_sans.font_normal,
                theme.foreground,
            ),
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );
        user_interface.painter().text(
            preview_position,
            Align2::RIGHT_CENTER,
            Self::truncate_text_to_width(
                user_interface,
                preview_text,
                (row_rect.width() * 0.35).max(0.0),
                &theme.font_library.font_noto_sans.font_small,
                theme.foreground_preview,
            ),
            theme.font_library.font_noto_sans.font_small.clone(),
            theme.foreground_preview,
        );
    }

    fn render_union_variant_layout_rows(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_index: usize,
        variant_field_draft: &SymbolLayoutFieldEditDraft,
        selected_unassigned_span: Option<&SymbolLayoutUnassignedSelection>,
    ) -> Option<(String, SymbolLayoutUnassignedRowContext, SymbolLayoutUnassignedRowAction)> {
        let variant_draft = Self::create_union_variant_layout_draft(project_symbol_catalog, union_draft, variant_index, variant_field_draft);
        let variant_layout_id = variant_draft.layout_id.as_str();

        let Some((layout_size_in_bytes, mut field_spans)) = self.resolve_draft_field_spans(project_symbol_catalog, &variant_draft) else {
            self.render_union_variant_preview_row(user_interface, "UNASSIGNED", "variant layout unresolved");
            return None;
        };
        let unassigned_split_offsets = self
            .symbol_layout_editor_view_data
            .read("SymbolLayoutEditor variant unassigned split offsets")
            .map(|symbol_layout_editor_view_data| symbol_layout_editor_view_data.get_unassigned_split_offsets_for_layout(Some(variant_layout_id)))
            .unwrap_or_default();
        let mut pending_unassigned_row_action = None;
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
                for unassigned_row_context in Self::build_unassigned_row_contexts(
                    next_visible_offset,
                    unassigned_size,
                    &unassigned_split_offsets,
                    previous_visible_field,
                    move_down_field,
                ) {
                    let is_selected = selected_unassigned_span.is_some_and(|selected_unassigned_span| {
                        selected_unassigned_span.matches(
                            Some(variant_layout_id),
                            unassigned_row_context.offset_in_bytes,
                            unassigned_row_context.size_in_bytes,
                        )
                    });
                    if let Some(unassigned_row_action) = self.render_unassigned_layout_row(
                        user_interface,
                        Some(variant_layout_id),
                        unassigned_row_context.clone(),
                        true,
                        false,
                        is_selected,
                    ) {
                        pending_unassigned_row_action = Some((variant_layout_id.to_string(), unassigned_row_context, unassigned_row_action));
                    }
                }
            }

            if let Some(field_draft) = variant_draft.field_drafts.get(field_span.field_position) {
                self.render_union_variant_preview_row(
                    user_interface,
                    field_draft.field_name.trim(),
                    &Self::format_field_data_type_preview(field_draft),
                );
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
            for unassigned_row_context in
                Self::build_unassigned_row_contexts(next_visible_offset, unassigned_size, &unassigned_split_offsets, previous_visible_field, None)
            {
                let is_selected = selected_unassigned_span.is_some_and(|selected_unassigned_span| {
                    selected_unassigned_span.matches(
                        Some(variant_layout_id),
                        unassigned_row_context.offset_in_bytes,
                        unassigned_row_context.size_in_bytes,
                    )
                });
                if let Some(unassigned_row_action) = self.render_unassigned_layout_row(
                    user_interface,
                    Some(variant_layout_id),
                    unassigned_row_context.clone(),
                    true,
                    false,
                    is_selected,
                ) {
                    pending_unassigned_row_action = Some((variant_layout_id.to_string(), unassigned_row_context, unassigned_row_action));
                }
            }
        }

        pending_unassigned_row_action
    }

    fn render_field_context_menu(
        &self,
        user_interface: &mut Ui,
        layout_kind: SymbolicLayoutKind,
        context_menu_target: &SymbolLayoutFieldContextMenuTarget,
        field_count: usize,
    ) -> Option<SymbolLayoutFieldRowAction> {
        let theme = &self.app_context.theme;
        let field_index = context_menu_target.get_field_index();
        let can_remove_field = field_count > 1;
        let can_move_up = field_index > 0;
        let can_move_down = field_index + 1 < field_count;
        let mut open = true;
        let mut pending_field_row_action = None;
        let entry_name = if layout_kind.is_union() { "variant" } else { "field" };

        ContextMenu::new(
            self.app_context.clone(),
            "symbol_layout_field_context_menu",
            context_menu_target.get_position(),
            |user_interface, should_close| {
                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            &format!("Move {} up", entry_name),
                            "symbol_layout_field_ctx_move_up",
                            &None,
                            Self::FIELD_CONTEXT_MENU_WIDTH,
                        )
                        .icon(theme.icon_library.icon_handle_navigation_up_arrow_small.clone())
                        .disabled(!can_move_up),
                    )
                    .clicked()
                {
                    pending_field_row_action = Some(SymbolLayoutFieldRowAction::MoveUp);
                    *should_close = true;
                }

                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            &format!("Move {} down", entry_name),
                            "symbol_layout_field_ctx_move_down",
                            &None,
                            Self::FIELD_CONTEXT_MENU_WIDTH,
                        )
                        .icon(
                            theme
                                .icon_library
                                .icon_handle_navigation_down_arrow_small
                                .clone(),
                        )
                        .disabled(!can_move_down),
                    )
                    .clicked()
                {
                    pending_field_row_action = Some(SymbolLayoutFieldRowAction::MoveDown);
                    *should_close = true;
                }

                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            &format!("Insert new {} below", entry_name),
                            "symbol_layout_field_ctx_insert_below",
                            &None,
                            Self::FIELD_CONTEXT_MENU_WIDTH,
                        )
                        .icon(theme.icon_library.icon_handle_common_add.clone()),
                    )
                    .clicked()
                {
                    pending_field_row_action = Some(SymbolLayoutFieldRowAction::InsertAfter);
                    *should_close = true;
                }

                user_interface.separator();

                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            "Delete",
                            "symbol_layout_field_ctx_delete",
                            &None,
                            Self::FIELD_CONTEXT_MENU_WIDTH,
                        )
                        .icon(theme.icon_library.icon_handle_common_delete.clone())
                        .icon_background(theme.background_control_danger, theme.background_control_danger_dark)
                        .disabled(!can_remove_field),
                    )
                    .clicked()
                {
                    pending_field_row_action = Some(SymbolLayoutFieldRowAction::RequestRemoveFieldConfirmation);
                    *should_close = true;
                }
            },
        )
        .width(Self::FIELD_CONTEXT_MENU_WIDTH)
        .corner_radius(8)
        .show(user_interface, &mut open);

        if !open {
            SymbolLayoutEditorViewData::hide_field_context_menu(self.symbol_layout_editor_view_data.clone());
        }

        pending_field_row_action
    }

    fn render_layout_size_editor(
        &self,
        user_interface: &mut Ui,
        draft: &mut SymbolLayoutEditDraft,
    ) {
        self.render_u64_data_value_box(
            user_interface,
            &mut draft.size_text,
            &mut draft.size_format,
            "size",
            "symbol_layout_editor_layout_size",
            user_interface.available_width().max(1.0),
            Self::FIELD_ROW_HEIGHT,
        );
    }

    fn resolve_draft_field_spans(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
    ) -> Option<(u64, Vec<SymbolLayoutFieldSpan>)> {
        let struct_layout_descriptor = SymbolLayoutEditorViewData::build_symbol_layout_descriptor(project_symbol_catalog, draft).ok()?;
        let symbolic_struct_definition = struct_layout_descriptor.get_struct_layout_definition();
        let layout_size_in_bytes = symbolic_struct_definition
            .get_declared_size_in_bytes()
            .unwrap_or_else(|| {
                SymbolLayoutEditorViewData::resolve_symbolic_struct_size_in_bytes(
                    project_symbol_catalog,
                    symbolic_struct_definition,
                    &mut std::collections::HashSet::new(),
                )
            });
        let mut next_sequential_offset = 0_u64;
        let mut field_spans = Vec::with_capacity(symbolic_struct_definition.get_fields().len());

        for (field_position, symbolic_field_definition) in symbolic_struct_definition.get_fields().iter().enumerate() {
            let field_offset = match symbolic_field_definition.get_offset_resolution() {
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                    if symbolic_struct_definition.get_layout_kind().is_union() =>
                {
                    0
                }
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
            };
            let field_size_in_bytes = SymbolLayoutEditorViewData::resolve_symbolic_field_size_in_bytes(
                project_symbol_catalog,
                symbolic_field_definition,
                &mut std::collections::HashSet::new(),
            );

            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
            field_spans.push(SymbolLayoutFieldSpan {
                field_position,
                offset_in_bytes: field_offset,
                size_in_bytes: field_size_in_bytes,
            });
        }

        Some((layout_size_in_bytes, field_spans))
    }

    fn render_unassigned_layout_row(
        &self,
        user_interface: &mut Ui,
        layout_id: Option<&str>,
        row_context: SymbolLayoutUnassignedRowContext,
        can_show_context_menu: bool,
        can_define_field: bool,
        is_selected: bool,
    ) -> Option<SymbolLayoutUnassignedRowAction> {
        if row_context.size_in_bytes == 0 {
            return None;
        }

        let theme = &self.app_context.theme;
        let can_move_up = row_context.move_up_field.is_some() || row_context.move_up_unassigned_span.is_some();
        let can_move_down = row_context.move_down_field.is_some() || row_context.move_down_unassigned_span.is_some();
        let row_sense = if can_show_context_menu || can_move_up || can_move_down {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (row_rect, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::FIELD_ROW_HEIGHT), row_sense);
        let mut pending_unassigned_row_action = None;

        if is_selected {
            user_interface
                .painter()
                .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface
                .painter()
                .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
        }

        StateLayer {
            bounds_min: row_rect.min,
            bounds_max: row_rect.max,
            enabled: true,
            pressed: row_response.is_pointer_button_down_on(),
            has_hover: row_response.hovered(),
            has_focus: row_response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        let button_area_width = Self::ICON_BUTTON_WIDTH * 2.0;
        let button_area_left = (row_rect.max.x - button_area_width).max(row_rect.min.x);
        let mut button_min_x = button_area_left;
        let mut render_next_button = |icon_handle: &eframe::egui::TextureHandle, tooltip_text: &str, is_disabled: bool| -> Response {
            let button_rect = Rect::from_min_size(pos2(button_min_x, row_rect.min.y), vec2(Self::ICON_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT));
            button_min_x += Self::ICON_BUTTON_WIDTH;

            self.render_flat_icon_button_at(user_interface, button_rect, icon_handle, tooltip_text, is_disabled)
        };

        let move_up_response = render_next_button(
            &theme.icon_library.icon_handle_navigation_up_arrow_small,
            "Move this unassigned span up.",
            !can_move_up,
        );
        if move_up_response.clicked() {
            pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::MoveUp);
        }

        let move_down_response = render_next_button(
            &theme.icon_library.icon_handle_navigation_down_arrow_small,
            "Move this unassigned span down.",
            !can_move_down,
        );
        if move_down_response.clicked() {
            pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::MoveDown);
        }

        if can_show_context_menu && row_response.secondary_clicked() {
            let position = row_response
                .interact_pointer_pos()
                .unwrap_or_else(|| pos2(row_rect.left(), row_rect.center().y));
            SymbolLayoutEditorViewData::show_unassigned_context_menu_for_layout(
                self.symbol_layout_editor_view_data.clone(),
                layout_id.map(str::to_string),
                row_context.offset_in_bytes,
                row_context.size_in_bytes,
                position,
                row_context.merge_above_span.clone(),
                row_context.merge_below_span.clone(),
            );
            if pending_unassigned_row_action.is_none() {
                pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::SelectSpan);
            }
        } else if can_show_context_menu && row_response.clicked() && pending_unassigned_row_action.is_none() {
            pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::SelectSpan);
        } else if row_response.clicked() && pending_unassigned_row_action.is_none() {
            SymbolLayoutEditorViewData::hide_unassigned_context_menu(self.symbol_layout_editor_view_data.clone());
            SymbolLayoutEditorViewData::hide_field_context_menu(self.symbol_layout_editor_view_data.clone());
        }

        let left_text = format!("UNASSIGNED[{}]", row_context.size_in_bytes);
        let right_text = format!("0x{:X}", row_context.offset_in_bytes);
        let label_position = pos2(row_rect.min.x + Self::FIELD_ROW_LEFT_PADDING, row_rect.center().y);
        let right_text_x = button_area_left - Self::FIELD_ROW_LEFT_PADDING;
        let left_max_width = (right_text_x - label_position.x).max(0.0);
        let left_color = if can_define_field { theme.foreground } else { theme.foreground_preview };
        let left_text = Self::truncate_text_to_width(
            user_interface,
            &left_text,
            left_max_width,
            &theme.font_library.font_noto_sans.font_normal,
            left_color,
        );
        let left_width = Self::measure_text_width(user_interface, &left_text, &theme.font_library.font_noto_sans.font_normal, left_color);
        let right_max_width = (right_text_x - label_position.x - left_width - Self::FIELD_ROW_PREVIEW_GAP).max(0.0);
        let right_text = Self::truncate_text_to_width(
            user_interface,
            &right_text,
            right_max_width,
            &theme.font_library.font_noto_sans.font_small,
            theme.foreground_preview,
        );

        user_interface.painter().text(
            label_position,
            Align2::LEFT_CENTER,
            left_text,
            theme.font_library.font_noto_sans.font_normal.clone(),
            left_color,
        );
        if !right_text.is_empty() {
            user_interface.painter().text(
                pos2(right_text_x, row_rect.center().y),
                Align2::RIGHT_CENTER,
                right_text,
                theme.font_library.font_noto_sans.font_small.clone(),
                theme.foreground_preview,
            );
        }

        pending_unassigned_row_action
    }

    fn render_unassigned_context_menu(
        &self,
        user_interface: &mut Ui,
        context_menu_target: &SymbolLayoutUnassignedContextMenuTarget,
        can_define_field: bool,
    ) -> Option<SymbolLayoutUnassignedRowAction> {
        let theme = &self.app_context.theme;
        let mut open = true;
        let mut pending_unassigned_row_action = None;

        ContextMenu::new(
            self.app_context.clone(),
            "symbol_layout_unassigned_context_menu",
            context_menu_target.get_position(),
            |user_interface, should_close| {
                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            "Split Range",
                            "symbol_layout_unassigned_ctx_split_range",
                            &None,
                            Self::FIELD_CONTEXT_MENU_WIDTH,
                        )
                        .icon(theme.icon_library.icon_handle_common_add.clone())
                        .disabled(context_menu_target.get_size_in_bytes() < 2),
                    )
                    .clicked()
                {
                    pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::SplitRange);
                    *should_close = true;
                }

                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            "Merge with Above",
                            "symbol_layout_unassigned_ctx_merge_above",
                            &None,
                            Self::FIELD_CONTEXT_MENU_WIDTH,
                        )
                        .disabled(context_menu_target.get_merge_above_span().is_none()),
                    )
                    .clicked()
                {
                    pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::MergeAbove);
                    *should_close = true;
                }

                if user_interface
                    .add(
                        ToolbarMenuItemView::new(
                            self.app_context.clone(),
                            "Merge with Below",
                            "symbol_layout_unassigned_ctx_merge_below",
                            &None,
                            Self::FIELD_CONTEXT_MENU_WIDTH,
                        )
                        .disabled(context_menu_target.get_merge_below_span().is_none()),
                    )
                    .clicked()
                {
                    pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::MergeBelow);
                    *should_close = true;
                }

                if can_define_field {
                    user_interface.separator();

                    if user_interface
                        .add(
                            ToolbarMenuItemView::new(
                                self.app_context.clone(),
                                "Define Field...",
                                "symbol_layout_unassigned_ctx_define_field_at",
                                &None,
                                Self::FIELD_CONTEXT_MENU_WIDTH,
                            )
                            .icon(theme.icon_library.icon_handle_common_add.clone()),
                        )
                        .clicked()
                    {
                        pending_unassigned_row_action = Some(SymbolLayoutUnassignedRowAction::DefineField);
                        *should_close = true;
                    }
                }
            },
        )
        .width(Self::FIELD_CONTEXT_MENU_WIDTH)
        .corner_radius(8)
        .show(user_interface, &mut open);

        if !open {
            SymbolLayoutEditorViewData::hide_unassigned_context_menu(self.symbol_layout_editor_view_data.clone());
        }

        pending_unassigned_row_action
    }

    fn render_field_rows(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &mut SymbolLayoutEditDraft,
        selected_field_index: Option<usize>,
        selected_unassigned_span: Option<&SymbolLayoutUnassignedSelection>,
    ) {
        let field_count = draft.field_drafts.len();
        let layout_kind = draft.layout_kind;
        let mut pending_field_row_action = None;
        let mut pending_unassigned_row_action: Option<(Option<String>, SymbolLayoutUnassignedRowContext, SymbolLayoutUnassignedRowAction)> = None;
        let field_spans = self.resolve_draft_field_spans(project_symbol_catalog, draft);
        let field_spans_by_position = field_spans
            .as_ref()
            .map(|(_layout_size_in_bytes, field_spans)| {
                field_spans
                    .iter()
                    .map(|field_span| (field_span.field_position, *field_span))
                    .collect::<std::collections::HashMap<usize, SymbolLayoutFieldSpan>>()
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
                let variant_title = draft
                    .field_drafts
                    .get(field_index)
                    .map(|field_draft| {
                        let trimmed_field_name = field_draft.field_name.trim();
                        if trimmed_field_name.is_empty() {
                            format!("Variant {}", field_index + 1)
                        } else {
                            trimmed_field_name.to_string()
                        }
                    })
                    .unwrap_or_else(|| format!("Variant {}", field_index + 1));
                let union_draft_preview = draft.clone();

                user_interface.add(
                    GroupBox::new_from_theme(&self.app_context.theme, &variant_title, |user_interface| {
                        let Some(field_draft) = draft.field_drafts.get_mut(field_index) else {
                            return;
                        };
                        if let Some(field_row_action) = self.render_field_editor_section(
                            user_interface,
                            layout_kind,
                            field_draft,
                            field_index,
                            selected_field_index == Some(field_index),
                            field_index > 0,
                            field_index + 1 < field_count,
                        ) {
                            pending_field_row_action = Some((field_index, field_row_action));
                        }
                        let variant_field_preview_draft = field_draft.clone();
                        user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                        if let Some((variant_layout_id, unassigned_row_context, unassigned_row_action)) = self.render_union_variant_layout_rows(
                            user_interface,
                            project_symbol_catalog,
                            &union_draft_preview,
                            field_index,
                            &variant_field_preview_draft,
                            selected_unassigned_span,
                        ) {
                            pending_unassigned_row_action = Some((Some(variant_layout_id), unassigned_row_context, unassigned_row_action));
                        }
                        user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                        if self.render_add_entry_button(user_interface, "Add a new field to this union variant.") {
                            pending_field_row_action = Some((field_index, SymbolLayoutFieldRowAction::InsertFieldIntoVariant));
                        }
                    })
                    .desired_width(user_interface.available_width()),
                );

                if field_index + 1 < field_count {
                    user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                }
            }
        } else {
            for field_index in field_render_indices {
                let Some(field_draft) = draft.field_drafts.get_mut(field_index) else {
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
                        for unassigned_row_context in Self::build_unassigned_row_contexts(
                            next_visible_offset,
                            unassigned_size,
                            &unassigned_split_offsets,
                            previous_visible_field,
                            move_down_field,
                        ) {
                            let is_selected = selected_unassigned_span.is_some_and(|selected_unassigned_span| {
                                selected_unassigned_span.matches(None, unassigned_row_context.offset_in_bytes, unassigned_row_context.size_in_bytes)
                            });
                            if let Some(unassigned_row_action) =
                                self.render_unassigned_layout_row(user_interface, None, unassigned_row_context.clone(), true, true, is_selected)
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
                        Self::can_move_struct_field_up(field_spans, &unassigned_split_offsets, field_index),
                        Self::can_move_struct_field_down(field_spans, *layout_size_in_bytes, &unassigned_split_offsets, field_index),
                    )
                } else {
                    (false, false)
                };
                if let Some(field_row_action) = self.render_field_editor_section(
                    user_interface,
                    layout_kind,
                    field_draft,
                    field_index,
                    selected_field_index == Some(field_index),
                    can_move_up,
                    can_move_down,
                ) {
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
                Self::build_unassigned_row_contexts(next_visible_offset, unassigned_size, &unassigned_split_offsets, move_up_field, None)
            {
                let is_selected = selected_unassigned_span.is_some_and(|selected_unassigned_span| {
                    selected_unassigned_span.matches(None, unassigned_row_context.offset_in_bytes, unassigned_row_context.size_in_bytes)
                });
                if let Some(unassigned_row_action) =
                    self.render_unassigned_layout_row(user_interface, None, unassigned_row_context.clone(), true, true, is_selected)
                {
                    pending_unassigned_row_action = Some((None, unassigned_row_context, unassigned_row_action));
                }
            }
        }

        let (field_context_menu_target, unassigned_context_menu_target) = self
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
            && field_context_menu_target.get_field_index() < field_count
            && let Some(field_row_action) = self.render_field_context_menu(user_interface, draft.layout_kind, &field_context_menu_target, field_count)
        {
            pending_field_row_action = Some((field_context_menu_target.get_field_index(), field_row_action));
        }

        if let Some(unassigned_context_menu_target) = unassigned_context_menu_target
            && let Some(unassigned_row_action) = self.render_unassigned_context_menu(
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
            let mut target_variant_draft = target_layout_id
                .as_deref()
                .map(|target_layout_id| Self::create_union_variant_layout_draft_for_id(project_symbol_catalog, draft, target_layout_id));
            let mut persist_target_variant_draft = false;
            match unassigned_row_action {
                SymbolLayoutUnassignedRowAction::SelectSpan => {
                    SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                        self.symbol_layout_editor_view_data.clone(),
                        target_layout_id.clone(),
                        unassigned_row_context.offset_in_bytes,
                        unassigned_row_context.size_in_bytes,
                    );
                    let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                    self.focus_unassigned_span_in_struct_viewer(focus_draft, unassigned_row_context.offset_in_bytes, unassigned_row_context.size_in_bytes);
                }
                SymbolLayoutUnassignedRowAction::DefineField => {
                    if target_layout_id.is_some() {
                        log::warn!("Ignoring Define Field action for nested union variant unassigned span.");
                        return;
                    }
                    let mut field_draft = self.create_field_draft_for_unassigned_span(
                        project_symbol_catalog,
                        draft.layout_kind,
                        &draft.layout_id,
                        0,
                        unassigned_row_context.offset_in_bytes,
                    );
                    field_draft.field_name = format!("field_{:08X}", unassigned_row_context.offset_in_bytes);
                    field_draft.field_name = Self::build_unique_field_name(draft, &field_draft.field_name);
                    SymbolLayoutEditorViewData::begin_define_field_from_unassigned_span(
                        self.symbol_layout_editor_view_data.clone(),
                        draft.layout_id.clone(),
                        unassigned_row_context.offset_in_bytes,
                        unassigned_row_context.size_in_bytes,
                        self.default_data_type_ref(),
                    );
                    SymbolLayoutEditorViewData::replace_define_field_draft(self.symbol_layout_editor_view_data.clone(), field_draft);
                }
                SymbolLayoutUnassignedRowAction::MoveUp => {
                    let updated_unassigned_selection = if let Some(target_variant_draft) = target_variant_draft.as_mut() {
                        Self::move_unassigned_span_up(target_variant_draft, unassigned_row_context.clone())
                    } else {
                        Self::move_unassigned_span_up(draft, unassigned_row_context.clone())
                    };

                    if let Some(updated_unassigned_selection) = updated_unassigned_selection {
                        persist_target_variant_draft = target_layout_id.is_some();
                        SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            target_layout_id.clone(),
                            updated_unassigned_selection.get_offset_in_bytes(),
                            updated_unassigned_selection.get_size_in_bytes(),
                        );
                        let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                        self.focus_unassigned_span_in_struct_viewer(
                            focus_draft,
                            updated_unassigned_selection.get_offset_in_bytes(),
                            updated_unassigned_selection.get_size_in_bytes(),
                        );
                    } else if let Some(move_up_unassigned_span) = unassigned_row_context.move_up_unassigned_span.as_ref() {
                        let old_split_offset = unassigned_row_context.offset_in_bytes;
                        let new_split_offset = move_up_unassigned_span
                            .get_offset_in_bytes()
                            .saturating_add(unassigned_row_context.size_in_bytes);
                        SymbolLayoutEditorViewData::move_unassigned_split_offset_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            target_layout_id.clone(),
                            old_split_offset,
                            new_split_offset,
                        );
                        SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            target_layout_id.clone(),
                            move_up_unassigned_span.get_offset_in_bytes(),
                            unassigned_row_context.size_in_bytes,
                        );
                        let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                        self.focus_unassigned_span_in_struct_viewer(
                            focus_draft,
                            move_up_unassigned_span.get_offset_in_bytes(),
                            unassigned_row_context.size_in_bytes,
                        );
                    }
                }
                SymbolLayoutUnassignedRowAction::MoveDown => {
                    let updated_unassigned_selection = if let Some(target_variant_draft) = target_variant_draft.as_mut() {
                        Self::move_unassigned_span_down(target_variant_draft, unassigned_row_context.clone())
                    } else {
                        Self::move_unassigned_span_down(draft, unassigned_row_context.clone())
                    };

                    if let Some(updated_unassigned_selection) = updated_unassigned_selection {
                        persist_target_variant_draft = target_layout_id.is_some();
                        SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            target_layout_id.clone(),
                            updated_unassigned_selection.get_offset_in_bytes(),
                            updated_unassigned_selection.get_size_in_bytes(),
                        );
                        let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                        self.focus_unassigned_span_in_struct_viewer(
                            focus_draft,
                            updated_unassigned_selection.get_offset_in_bytes(),
                            updated_unassigned_selection.get_size_in_bytes(),
                        );
                    } else if let Some(move_down_unassigned_span) = unassigned_row_context.move_down_unassigned_span.as_ref() {
                        let old_split_offset = unassigned_row_context
                            .offset_in_bytes
                            .saturating_add(unassigned_row_context.size_in_bytes);
                        let new_unassigned_offset = unassigned_row_context
                            .offset_in_bytes
                            .saturating_add(move_down_unassigned_span.get_size_in_bytes());
                        SymbolLayoutEditorViewData::move_unassigned_split_offset_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            target_layout_id.clone(),
                            old_split_offset,
                            new_unassigned_offset,
                        );
                        SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            target_layout_id.clone(),
                            new_unassigned_offset,
                            unassigned_row_context.size_in_bytes,
                        );
                        let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                        self.focus_unassigned_span_in_struct_viewer(focus_draft, new_unassigned_offset, unassigned_row_context.size_in_bytes);
                    }
                }
                SymbolLayoutUnassignedRowAction::SplitRange => {
                    if let Some(updated_unassigned_selection) = SymbolLayoutEditorViewData::split_unassigned_span_for_layout(
                        self.symbol_layout_editor_view_data.clone(),
                        target_layout_id.clone(),
                        unassigned_row_context.offset_in_bytes,
                        unassigned_row_context.size_in_bytes,
                    ) {
                        let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                        self.focus_unassigned_span_in_struct_viewer(
                            focus_draft,
                            updated_unassigned_selection.get_offset_in_bytes(),
                            updated_unassigned_selection.get_size_in_bytes(),
                        );
                    }
                }
                SymbolLayoutUnassignedRowAction::MergeAbove => {
                    if let Some(merge_above_span) = unassigned_row_context.merge_above_span.as_ref() {
                        SymbolLayoutEditorViewData::remove_unassigned_split_offset_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            target_layout_id.clone(),
                            unassigned_row_context.offset_in_bytes,
                        );
                        SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            target_layout_id.clone(),
                            merge_above_span.get_offset_in_bytes(),
                            merge_above_span.get_size_in_bytes(),
                        );
                        let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                        self.focus_unassigned_span_in_struct_viewer(focus_draft, merge_above_span.get_offset_in_bytes(), merge_above_span.get_size_in_bytes());
                    }
                }
                SymbolLayoutUnassignedRowAction::MergeBelow => {
                    if let Some(merge_below_span) = unassigned_row_context.merge_below_span.as_ref() {
                        SymbolLayoutEditorViewData::remove_unassigned_split_offset_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            target_layout_id.clone(),
                            unassigned_row_context
                                .offset_in_bytes
                                .saturating_add(unassigned_row_context.size_in_bytes),
                        );
                        SymbolLayoutEditorViewData::select_unassigned_span_for_layout(
                            self.symbol_layout_editor_view_data.clone(),
                            target_layout_id.clone(),
                            merge_below_span.get_offset_in_bytes(),
                            merge_below_span.get_size_in_bytes(),
                        );
                        let focus_draft = target_variant_draft.as_ref().unwrap_or(draft);
                        self.focus_unassigned_span_in_struct_viewer(focus_draft, merge_below_span.get_offset_in_bytes(), merge_below_span.get_size_in_bytes());
                    }
                }
            }

            if persist_target_variant_draft && let Some(target_variant_draft) = target_variant_draft.as_ref() {
                self.persist_variant_layout_draft(project_symbol_catalog, target_variant_draft);
            }
        }

        if let Some((field_index, field_row_action)) = pending_field_row_action {
            let mut field_index_to_focus = None;
            match field_row_action {
                SymbolLayoutFieldRowAction::InsertAfter => {
                    let insert_index = field_index.saturating_add(1).min(draft.field_drafts.len());
                    let mut field_draft = self.create_field_draft_for_layout_kind(project_symbol_catalog, draft.layout_kind, &draft.layout_id, insert_index);
                    field_draft.field_name = Self::build_unique_field_name(draft, &field_draft.field_name);
                    draft.field_drafts.insert(insert_index, field_draft);
                    field_index_to_focus = Some(insert_index);
                }
                SymbolLayoutFieldRowAction::InsertFieldIntoVariant => {
                    if self.append_field_to_variant_layout(project_symbol_catalog, draft, field_index) {
                        field_index_to_focus = Some(field_index);
                    }
                }
                SymbolLayoutFieldRowAction::RequestRemoveFieldConfirmation => {
                    SymbolLayoutEditorViewData::request_field_delete_confirmation(
                        self.symbol_layout_editor_view_data.clone(),
                        draft.layout_id.clone(),
                        field_index,
                    );
                    field_index_to_focus = Some(field_index);
                }
                SymbolLayoutFieldRowAction::MoveUp => {
                    if !draft.layout_kind.is_union() {
                        if let Some((_layout_size_in_bytes, field_spans)) = field_spans.as_ref()
                            && Self::move_struct_field_up(draft, field_spans, &unassigned_split_offsets, field_index)
                        {
                            field_index_to_focus = Some(field_index);
                        }
                    } else if field_index > 0 {
                        draft.field_drafts.swap(field_index, field_index - 1);
                        field_index_to_focus = Some(field_index - 1);
                    }
                }
                SymbolLayoutFieldRowAction::MoveDown => {
                    if !draft.layout_kind.is_union() {
                        if let Some((layout_size_in_bytes, field_spans)) = field_spans.as_ref()
                            && Self::move_struct_field_down(draft, field_spans, *layout_size_in_bytes, &unassigned_split_offsets, field_index)
                        {
                            field_index_to_focus = Some(field_index);
                        }
                    } else if field_index + 1 < draft.field_drafts.len() {
                        draft.field_drafts.swap(field_index, field_index + 1);
                        field_index_to_focus = Some(field_index + 1);
                    }
                }
                SymbolLayoutFieldRowAction::SelectField => {
                    field_index_to_focus = Some(field_index);
                }
            }

            if let Some(field_index_to_focus) = field_index_to_focus {
                SymbolLayoutEditorViewData::select_field(self.symbol_layout_editor_view_data.clone(), field_index_to_focus);
                self.focus_field_in_struct_viewer(project_symbol_catalog, draft, field_index_to_focus);
            }
        }
    }

    fn render_symbol_layout_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        take_over_title: &str,
        baseline_draft: Option<&SymbolLayoutEditDraft>,
        draft: Option<&SymbolLayoutEditDraft>,
        selected_field_index: Option<usize>,
        selected_unassigned_span: Option<&SymbolLayoutUnassignedSelection>,
        show_layout_name_editor: bool,
    ) {
        let Some(draft) = draft else {
            return;
        };
        let baseline_draft = baseline_draft.unwrap_or(draft);

        let mut edited_draft = draft.clone();
        let validation_result = SymbolLayoutEditorViewData::build_symbol_layout_descriptor(project_symbol_catalog, &edited_draft);
        let usage_count = edited_draft
            .original_layout_id
            .as_deref()
            .map(|selected_layout_id| SymbolLayoutEditorViewData::count_symbol_claim_usages(project_symbol_catalog, selected_layout_id))
            .unwrap_or(0);
        let has_unsaved_changes = edited_draft != *baseline_draft;
        let is_creating_new_layout = edited_draft.original_layout_id.is_none();
        let is_union_layout = edited_draft.layout_kind.is_union();
        let can_save = validation_result.is_ok() && has_unsaved_changes;
        let header_action_width = if is_union_layout { Self::ICON_BUTTON_WIDTH } else { 0.0 };
        let mut should_cancel_take_over = false;
        let mut should_save_draft = false;
        let should_append_field = Cell::new(false);

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
                                    self.normalize_union_field_drafts(project_symbol_catalog, &mut edited_draft);
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
                                    project_symbol_catalog,
                                    &mut edited_draft,
                                    selected_field_index,
                                    selected_unassigned_span,
                                );
                                if !is_union_layout {
                                    user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                                    if self.render_add_entry_button(user_interface, "Add a new field entry.") {
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
                                    project_symbol_catalog,
                                    &mut edited_draft,
                                    selected_field_index,
                                    selected_unassigned_span,
                                );
                                if !is_union_layout {
                                    user_interface.add_space(Self::TAKE_OVER_GROUPBOX_SPACING);
                                    if self.render_add_entry_button(user_interface, "Add a new field entry.") {
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
            let mut field_draft =
                self.create_field_draft_for_layout_kind(project_symbol_catalog, edited_draft.layout_kind, &edited_draft.layout_id, field_index_to_focus);
            field_draft.field_name = Self::build_unique_field_name(&edited_draft, &field_draft.field_name);
            edited_draft.field_drafts.push(field_draft);
            SymbolLayoutEditorViewData::select_field(self.symbol_layout_editor_view_data.clone(), field_index_to_focus);
            self.focus_field_in_struct_viewer(project_symbol_catalog, &edited_draft, field_index_to_focus);
        }

        if should_cancel_take_over {
            SymbolLayoutEditorViewData::cancel_take_over_state(self.symbol_layout_editor_view_data.clone());
            self.clear_struct_viewer_if_symbol_layout_focused();
            return;
        }

        if should_save_draft {
            match SymbolLayoutEditorViewData::apply_draft_to_catalog(project_symbol_catalog, &edited_draft) {
                Ok(updated_project_symbol_catalog) => {
                    self.persist_project_symbol_catalog(updated_project_symbol_catalog.clone());
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

    fn render_define_field_from_unassigned_take_over(
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
            let insert_index = Self::field_insert_index_for_offset(&field_spans, updated_draft.field_drafts.len(), field_offset_in_bytes);

            updated_draft.field_drafts.insert(insert_index, new_field_draft);
            SymbolLayoutEditorViewData::update_draft(self.symbol_layout_editor_view_data.clone(), updated_draft.clone());
            SymbolLayoutEditorViewData::return_to_define_field_source(self.symbol_layout_editor_view_data.clone(), return_state.clone());
            SymbolLayoutEditorViewData::select_field(self.symbol_layout_editor_view_data.clone(), insert_index);
            self.focus_field_in_struct_viewer(project_symbol_catalog, &updated_draft, insert_index);
            return;
        }

        SymbolLayoutEditorViewData::replace_define_field_draft(self.symbol_layout_editor_view_data.clone(), edited_define_field_draft);
    }

    fn render_delete_confirmation_take_over(
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

    fn render_field_delete_confirmation_take_over(
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

#[cfg(test)]
mod tests {
    use super::{
        SymbolLayoutEditorView, SymbolLayoutFieldContainerKind, SymbolLayoutFieldEditDraft, SymbolLayoutFieldSpan, SymbolLayoutUnassignedAdjacentField,
        SymbolLayoutUnassignedRowContext,
    };
    use crate::views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData;
    use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{SymbolLayoutEditDraft, SymbolLayoutFieldOffsetMode};
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::{built_in_types::u32::data_type_u32::DataTypeU32, data_type_ref::DataTypeRef},
        data_values::anonymous_value_string_format::AnonymousValueStringFormat,
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::project_symbol_catalog::ProjectSymbolCatalog,
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

        assert_eq!(SymbolLayoutEditorView::format_field_data_type_preview(&field_draft), "u16[4]");
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

        assert_eq!(SymbolLayoutEditorView::field_insert_index_for_offset(&field_spans, 2, 8), 1);
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

        let updated_unassigned_selection = SymbolLayoutEditorView::move_unassigned_span_up(&mut draft, row_context).expect("Expected span to move.");

        assert_eq!(updated_unassigned_selection.get_offset_in_bytes(), 0);
        assert_eq!(updated_unassigned_selection.get_size_in_bytes(), 12);
        assert_eq!(draft.field_drafts[0].offset_mode, SymbolLayoutFieldOffsetMode::Static);
        assert_eq!(draft.field_drafts[0].static_offset_in_bytes, "12");
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

        let updated_unassigned_selection = SymbolLayoutEditorView::move_unassigned_span_down(&mut draft, row_context).expect("Expected span to move.");

        assert_eq!(updated_unassigned_selection.get_offset_in_bytes(), 8);
        assert_eq!(updated_unassigned_selection.get_size_in_bytes(), 12);
        assert_eq!(draft.field_drafts[1].offset_mode, SymbolLayoutFieldOffsetMode::Static);
        assert_eq!(draft.field_drafts[1].static_offset_in_bytes, "4");
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

        assert!(SymbolLayoutEditorView::move_struct_field_down(
            &mut draft,
            &field_spans,
            32,
            &BTreeSet::new(),
            0
        ));

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

        assert!(SymbolLayoutEditorView::move_struct_field_down(&mut draft, &field_spans, 32, &split_offsets, 0));

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

        assert!(SymbolLayoutEditorView::move_struct_field_down(
            &mut draft,
            &field_spans,
            32,
            &BTreeSet::new(),
            0
        ));

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

        assert!(SymbolLayoutEditorView::move_struct_field_up(&mut draft, &field_spans, &split_offsets, 0));

        assert_eq!(draft.field_drafts[0].field_name, "field_a");
        assert_eq!(draft.field_drafts[0].static_offset_in_bytes, "4");
        assert_eq!(draft.field_drafts[1].field_name, "field_b");
        assert_eq!(draft.field_drafts[1].static_offset_in_bytes, "16");
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

        assert!(SymbolLayoutEditorView::move_struct_field_down(&mut draft, &field_spans, 8, &BTreeSet::new(), 0));

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

        assert_eq!(SymbolLayoutEditorView::resolve_first_unassigned_offset(&field_spans, 24), Some(4));
    }

    #[test]
    fn resolve_first_unassigned_offset_finds_tail_gap() {
        let field_spans = [SymbolLayoutFieldSpan {
            field_position: 0,
            offset_in_bytes: 0,
            size_in_bytes: 4,
        }];

        assert_eq!(SymbolLayoutEditorView::resolve_first_unassigned_offset(&field_spans, 16), Some(4));
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

        let variant_draft = SymbolLayoutEditorView::create_union_variant_layout_draft(&project_symbol_catalog, &union_draft, 0, &variant_field_draft);

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

        let variant_draft = SymbolLayoutEditorView::create_union_variant_layout_draft(&project_symbol_catalog, &union_draft, 0, &variant_field_draft);

        assert_eq!(variant_draft.original_layout_id.as_deref(), Some("actor.state.alive"));
        assert_eq!(variant_draft.layout_id, "actor.state.alive");
        assert_eq!(variant_draft.size_text, "64");
    }

    #[test]
    fn build_unassigned_row_contexts_splits_contiguous_unassigned_rows() {
        let split_offsets = BTreeSet::from([4, 12]);
        let row_contexts = SymbolLayoutEditorView::build_unassigned_row_contexts(0, 16, &split_offsets, None, None);

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
        field_draft.offset_mode = super::SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = String::from("0x10");

        assert_eq!(
            SymbolLayoutEditorView::validate_define_field_draft(&project_symbol_catalog, &field_draft, 0x100, 0x40),
            Ok((0x110, 4))
        );
    }

    #[test]
    fn validate_define_field_draft_rejects_field_that_crosses_unassigned_span() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new(DataTypeU32::DATA_TYPE_ID));

        field_draft.field_name = String::from("too_wide");
        field_draft.offset_mode = super::SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = String::from("0x3E");

        assert!(SymbolLayoutEditorView::validate_define_field_draft(&project_symbol_catalog, &field_draft, 0x100, 0x40).is_err());
    }

    #[test]
    fn format_field_data_type_preview_includes_hidden_marker() {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("u8"));

        field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::FixedArray;
        field_draft.container_edit.fixed_array_length = String::from("12");
        field_draft.is_hidden = true;

        assert_eq!(SymbolLayoutEditorView::format_field_data_type_preview(&field_draft), "u8[12] hidden");
    }

    #[test]
    fn format_field_data_type_preview_includes_fixed_array_display_resolver() {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("u64"));

        field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::FixedArray;
        field_draft.container_edit.fixed_array_length = String::from("1024");
        field_draft.container_edit.display_count_resolver_id = String::from("entity.count");

        assert_eq!(
            SymbolLayoutEditorView::format_field_data_type_preview(&field_draft),
            "u64[1024] display resolver(entity.count)"
        );
    }

    #[test]
    fn format_field_data_type_preview_includes_pointer_size() {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("u32"));

        field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::Pointer;
        field_draft.container_edit.pointer_size = PointerScanPointerSize::Pointer64;

        assert_eq!(SymbolLayoutEditorView::format_field_data_type_preview(&field_draft), "u32*(u64)");
    }

    #[test]
    fn format_field_data_type_preview_includes_fixed_pointer_array() {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("Entity"));

        field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::FixedPointerArray;
        field_draft.container_edit.pointer_size = PointerScanPointerSize::Pointer64;
        field_draft.container_edit.fixed_array_length = String::from("1024");
        field_draft.container_edit.display_count_resolver_id = String::from("entity.count");

        assert_eq!(
            SymbolLayoutEditorView::format_field_data_type_preview(&field_draft),
            "Entity*(u64)[1024] display resolver(entity.count)"
        );
    }

    #[test]
    fn format_field_data_type_preview_includes_dynamic_array_resolver() {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new("game.item"));

        field_draft.container_edit.kind = SymbolLayoutFieldContainerKind::DynamicArray;
        field_draft.container_edit.dynamic_array_count_resolver_id = String::from("inventory.count");

        assert_eq!(
            SymbolLayoutEditorView::format_field_data_type_preview(&field_draft),
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

        let builtin_details_struct =
            SymbolLayoutEditorView::build_field_details_struct(&project_symbol_catalog, SymbolicLayoutKind::Struct, &builtin_field_draft);
        let symbol_layout_details_struct =
            SymbolLayoutEditorView::build_field_details_struct(&project_symbol_catalog, SymbolicLayoutKind::Struct, &symbol_layout_field_draft);

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

        let details_struct = SymbolLayoutEditorView::build_field_details_struct(&project_symbol_catalog, SymbolicLayoutKind::Union, &variant_field_draft);

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
            details_struct
                .get_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_OFFSET_MODE)
                .is_none()
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
        let (selected_layout_id, filter_text, take_over_state, baseline_draft, draft, selected_field_index, selected_unassigned_span, define_field_draft) =
            self.symbol_layout_editor_view_data
                .read("SymbolLayoutEditor view")
                .map(|symbol_layout_editor_view_data| {
                    (
                        symbol_layout_editor_view_data
                            .get_selected_layout_id()
                            .map(str::to_string),
                        symbol_layout_editor_view_data.get_filter_text().to_string(),
                        symbol_layout_editor_view_data.get_take_over_state().cloned(),
                        symbol_layout_editor_view_data.get_baseline_draft().cloned(),
                        symbol_layout_editor_view_data.get_draft().cloned(),
                        symbol_layout_editor_view_data.get_selected_field_index(),
                        symbol_layout_editor_view_data
                            .get_selected_unassigned_span()
                            .cloned(),
                        symbol_layout_editor_view_data.get_define_field_draft().cloned(),
                    )
                })
                .unwrap_or((None, String::new(), None, None, None, None, None, None));
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
                self.clear_struct_viewer_if_symbol_layout_focused();
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
            self.focus_selected_layout_in_struct_viewer(&project_symbol_catalog, next_layout_id.as_deref());
        }

        if !is_take_over_active && can_handle_window_shortcuts && user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown)) {
            let next_layout_id = SymbolLayoutEditorViewData::navigate_symbol_layout_selection(
                self.symbol_layout_editor_view_data.clone(),
                &project_symbol_catalog,
                ListNavigationDirection::Down,
            );
            self.focus_selected_layout_in_struct_viewer(&project_symbol_catalog, next_layout_id.as_deref());
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
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            selected_field_index,
                            selected_unassigned_span.as_ref(),
                            true,
                        );
                    }
                    Some(SymbolLayoutEditorTakeOverState::RenameSymbolLayout { .. }) => {
                        self.render_symbol_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "Rename Symbol Layout",
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            selected_field_index,
                            selected_unassigned_span.as_ref(),
                            true,
                        );
                    }
                    Some(SymbolLayoutEditorTakeOverState::OpenSymbolLayout { .. }) => {
                        self.render_symbol_layout_take_over(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            "Edit Symbol Layout",
                            baseline_draft.as_ref(),
                            draft.as_ref(),
                            selected_field_index,
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
                        self.render_list_panel(
                            &mut content_user_interface,
                            &project_symbol_catalog,
                            selected_layout_id.as_deref(),
                            &filter_text,
                            false,
                        );
                    }
                }
            })
            .response
    }
}
