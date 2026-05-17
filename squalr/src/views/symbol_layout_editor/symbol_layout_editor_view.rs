mod rows;
mod takeovers;
mod toolbars;

use crate::app_context::AppContext;
use crate::ui::converters::{data_type_to_icon_converter::DataTypeToIconConverter, data_type_to_string_converter::DataTypeToStringConverter};
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::list_navigation::ListNavigationDirection;
use crate::ui::widgets::controls::{
    button::Button as ThemeButton,
    combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
    context_menu::context_menu::ContextMenu,
    data_value_box::data_value_box_view::DataValueBoxView,
    list_header::ListHeaderView,
    search_box::SearchBoxView,
    toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
};
use crate::views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData};
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
    SymbolLayoutEditDraft, SymbolLayoutEditorTakeOverState, SymbolLayoutEditorViewData, SymbolLayoutFieldContextMenuTarget, SymbolLayoutFieldEditDraft,
    SymbolLayoutFieldElementType, SymbolLayoutFieldOffsetMode, SymbolLayoutUnassignedContextMenuTarget,
};
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::{SymbolLayoutFieldContainerEdit, SymbolLayoutFieldContainerKind};
use eframe::egui::{Align, Direction, Grid, Id, Key, Layout, Response, RichText, ScrollArea, Ui, Widget, vec2};
use epaint::{Color32, CornerRadius};
use rows::{
    symbol_layout_field_row_action::SymbolLayoutFieldRowAction, symbol_layout_field_row_view::SymbolLayoutFieldRowView,
    symbol_layout_row_view::SymbolLayoutRowView, symbol_layout_unassigned_row_view::SymbolLayoutUnassignedRowView,
    union_variant_preview_row_view::UnionVariantPreviewRowView,
};
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
use squalr_engine_api::structures::{
    data_types::{
        built_in_types::{i32::data_type_i32::DataTypeI32, string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
        data_type_ref::DataTypeRef,
    },
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    details::DetailsEdit,
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    projects::{
        project_symbol_catalog::ProjectSymbolCatalog,
        symbol_layouts::symbol_layout_details::{
            SymbolLayoutDetails, SymbolLayoutDetailsEditOperation, SymbolLayoutDetailsFieldElementKind, SymbolLayoutFieldDetails,
        },
        symbol_layouts::symbol_layout_draft_ops::{
            SymbolLayoutDraftOps, SymbolLayoutFieldSpan, SymbolLayoutUnassignedAdjacentField, SymbolLayoutUnassignedRowContext, SymbolLayoutUnassignedSelection,
        },
    },
    structs::{
        symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
    },
};
use std::{collections::BTreeSet, str::FromStr, sync::Arc};
use toolbars::symbol_layout_list_toolbar_view::SymbolLayoutListToolbarView;

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
        self.clear_struct_viewer_if_symbol_layout_focused();
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

    fn append_field_to_variant_layout(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &mut SymbolLayoutEditDraft,
        variant_index: usize,
    ) -> Option<(SymbolLayoutEditDraft, usize)> {
        let Some(variant_field_draft) = union_draft.field_drafts.get(variant_index) else {
            return None;
        };
        let mut variant_draft = self.create_union_variant_layout_draft_with_pending(project_symbol_catalog, union_draft, variant_index, variant_field_draft);

        let Some((layout_size_in_bytes, field_spans)) = self.resolve_draft_field_spans(project_symbol_catalog, &variant_draft) else {
            return None;
        };
        let Some(field_offset_in_bytes) = SymbolLayoutDraftOps::resolve_tail_unassigned_offset(&field_spans, layout_size_in_bytes) else {
            return None;
        };

        let field_position = variant_draft.field_drafts.len();
        let mut field_draft =
            self.create_field_draft_for_layout_kind(project_symbol_catalog, SymbolicLayoutKind::Struct, &variant_draft.layout_id, field_position);
        field_draft.field_name = format!("field_{:08X}", field_offset_in_bytes);
        field_draft.field_name = SymbolLayoutDraftOps::build_unique_field_name(&variant_draft, &field_draft.field_name);
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

        self.cache_variant_layout_draft(&variant_draft)
            .then_some((variant_draft, field_position))
    }

    fn resolve_variant_tail_unassigned_offset(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_index: usize,
        variant_field_draft: &SymbolLayoutFieldEditDraft,
    ) -> Option<u64> {
        let variant_draft = self.create_union_variant_layout_draft_with_pending(project_symbol_catalog, union_draft, variant_index, variant_field_draft);
        let (layout_size_in_bytes, field_spans) = self.resolve_draft_field_spans(project_symbol_catalog, &variant_draft)?;

        SymbolLayoutDraftOps::resolve_tail_unassigned_offset(&field_spans, layout_size_in_bytes)
    }

    fn resolve_draft_tail_unassigned_offset(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
    ) -> Option<u64> {
        let (layout_size_in_bytes, field_spans) = self.resolve_draft_field_spans(project_symbol_catalog, draft)?;

        SymbolLayoutDraftOps::resolve_tail_unassigned_offset(&field_spans, layout_size_in_bytes)
    }

    fn create_union_variant_layout_draft_with_pending(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_index: usize,
        variant_field_draft: &SymbolLayoutFieldEditDraft,
    ) -> SymbolLayoutEditDraft {
        let variant_layout_id = variant_field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id();
        if let Some(mut pending_variant_draft) = self.read_pending_variant_layout_draft(variant_layout_id) {
            pending_variant_draft.size_text = union_draft.size_text.clone();
            pending_variant_draft.size_format = union_draft.size_format;

            return pending_variant_draft;
        }
        if let Some(variant_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == variant_layout_id)
        {
            return self.create_union_variant_layout_draft_for_id_with_pending(
                project_symbol_catalog,
                union_draft,
                variant_layout_descriptor.get_struct_layout_id(),
            );
        }

        let variant_layout_id = Self::build_union_variant_layout_id(project_symbol_catalog, union_draft, variant_index);
        if let Some(mut pending_variant_draft) = self.read_pending_variant_layout_draft(&variant_layout_id) {
            pending_variant_draft.size_text = union_draft.size_text.clone();
            pending_variant_draft.size_format = union_draft.size_format;

            return pending_variant_draft;
        }

        Self::create_virtual_union_variant_layout_draft(union_draft, variant_layout_id)
    }

    #[cfg(test)]
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

    fn create_union_variant_layout_draft_for_id_with_pending(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_layout_id: &str,
    ) -> SymbolLayoutEditDraft {
        if let Some(mut pending_variant_draft) = self.read_pending_variant_layout_draft(variant_layout_id) {
            pending_variant_draft.size_text = union_draft.size_text.clone();
            pending_variant_draft.size_format = union_draft.size_format;

            return pending_variant_draft;
        }

        Self::create_union_variant_layout_draft_for_id(project_symbol_catalog, union_draft, variant_layout_id)
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

    fn read_pending_variant_layout_draft(
        &self,
        variant_layout_id: &str,
    ) -> Option<SymbolLayoutEditDraft> {
        self.symbol_layout_editor_view_data
            .read("SymbolLayoutEditor read pending variant draft")
            .and_then(|symbol_layout_editor_view_data| {
                symbol_layout_editor_view_data
                    .get_pending_variant_draft(variant_layout_id)
                    .cloned()
            })
    }

    fn cache_variant_layout_draft(
        &self,
        variant_draft: &SymbolLayoutEditDraft,
    ) -> bool {
        let Some(mut symbol_layout_editor_view_data) = self
            .symbol_layout_editor_view_data
            .write("SymbolLayoutEditor cache pending variant draft")
        else {
            return false;
        };

        symbol_layout_editor_view_data.replace_pending_variant_draft(variant_draft.clone());
        true
    }

    fn pending_variant_drafts_for_union_from_view_data(
        symbol_layout_editor_view_data: &SymbolLayoutEditorViewData,
        union_draft: Option<&SymbolLayoutEditDraft>,
    ) -> Vec<(SymbolLayoutEditDraft, BTreeSet<u64>)> {
        let Some(union_draft) = union_draft.filter(|union_draft| union_draft.layout_kind.is_union()) else {
            return Vec::new();
        };
        let referenced_variant_layout_ids = union_draft
            .field_drafts
            .iter()
            .map(|field_draft| {
                field_draft
                    .data_type_selection
                    .visible_data_type()
                    .get_data_type_id()
                    .to_string()
            })
            .collect::<BTreeSet<_>>();

        symbol_layout_editor_view_data
            .get_pending_variant_drafts_with_split_offsets()
            .into_iter()
            .filter(|(variant_draft, _unassigned_split_offsets)| referenced_variant_layout_ids.contains(&variant_draft.layout_id))
            .collect()
    }

    fn pending_variant_drafts_for_union(
        &self,
        union_draft: Option<&SymbolLayoutEditDraft>,
    ) -> Vec<(SymbolLayoutEditDraft, BTreeSet<u64>)> {
        self.symbol_layout_editor_view_data
            .read("SymbolLayoutEditor read pending variant drafts")
            .map(|symbol_layout_editor_view_data| Self::pending_variant_drafts_for_union_from_view_data(&symbol_layout_editor_view_data, union_draft))
            .unwrap_or_default()
    }

    fn build_effective_project_symbol_catalog_from_pending_drafts(
        project_symbol_catalog: &ProjectSymbolCatalog,
        pending_variant_drafts: &[(SymbolLayoutEditDraft, BTreeSet<u64>)],
    ) -> ProjectSymbolCatalog {
        let mut effective_project_symbol_catalog = project_symbol_catalog.clone();
        let mut struct_layout_descriptors = effective_project_symbol_catalog
            .get_struct_layout_descriptors()
            .to_vec();

        for (variant_draft, unassigned_split_offsets) in pending_variant_drafts {
            let Ok(variant_descriptor) = SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(
                project_symbol_catalog,
                variant_draft,
                unassigned_split_offsets,
            ) else {
                continue;
            };
            let variant_layout_id = variant_descriptor.get_struct_layout_id().to_string();

            struct_layout_descriptors.retain(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() != variant_layout_id);
            struct_layout_descriptors.push(variant_descriptor);
        }

        effective_project_symbol_catalog.set_struct_layout_descriptors(struct_layout_descriptors);
        effective_project_symbol_catalog
    }

    fn build_effective_project_symbol_catalog_from_view_data(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        focused_variant_layout_id: Option<&str>,
    ) -> ProjectSymbolCatalog {
        let pending_variant_drafts = symbol_layout_editor_view_data
            .read("SymbolLayoutEditor build effective catalog")
            .map(|symbol_layout_editor_view_data| {
                let union_draft = symbol_layout_editor_view_data.get_draft();
                let mut pending_variant_drafts = Self::pending_variant_drafts_for_union_from_view_data(&symbol_layout_editor_view_data, union_draft);

                if let Some(focused_variant_layout_id) = focused_variant_layout_id
                    && !pending_variant_drafts
                        .iter()
                        .any(|(variant_draft, _unassigned_split_offsets)| variant_draft.layout_id == focused_variant_layout_id)
                    && let Some(focused_variant_draft) = symbol_layout_editor_view_data.get_pending_variant_draft(focused_variant_layout_id)
                {
                    pending_variant_drafts.push((
                        focused_variant_draft.clone(),
                        symbol_layout_editor_view_data.get_unassigned_split_offsets_for_layout(Some(focused_variant_layout_id)),
                    ));
                }

                pending_variant_drafts
            })
            .unwrap_or_default();

        Self::build_effective_project_symbol_catalog_from_pending_drafts(project_symbol_catalog, &pending_variant_drafts)
    }

    fn build_pending_variant_layout_descriptors(
        project_symbol_catalog: &ProjectSymbolCatalog,
        pending_variant_drafts: &[(SymbolLayoutEditDraft, BTreeSet<u64>)],
    ) -> Result<Vec<(Option<String>, StructLayoutDescriptor)>, String> {
        pending_variant_drafts
            .iter()
            .map(|(variant_draft, unassigned_split_offsets)| {
                SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(
                    project_symbol_catalog,
                    variant_draft,
                    unassigned_split_offsets,
                )
                .map(|struct_layout_descriptor| (variant_draft.original_layout_id.clone(), struct_layout_descriptor))
                .map_err(|error| format!("Variant layout `{}`: {}", variant_draft.layout_id, error))
            })
            .collect()
    }

    fn persist_variant_layout_draft(
        &self,
        variant_draft: &SymbolLayoutEditDraft,
    ) -> bool {
        self.cache_variant_layout_draft(variant_draft)
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
        .with_active_when_resolver(field_draft.to_active_when_resolver()))
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
            Some(DataTypeToIconConverter::convert_data_type_or_symbol_layout_to_icon(
                type_option.data_type_ref.get_data_type_id(),
                type_option.kind == SymbolLayoutFieldTypeOptionKind::SymbolLayout,
                &self.app_context.theme.icon_library,
            ))
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
                                let item_response = scroll_user_interface.add(ComboBoxItemView::new(
                                    self.app_context.clone(),
                                    &type_option.label,
                                    Some(DataTypeToIconConverter::convert_symbol_layout_to_icon(&self.app_context.theme.icon_library)),
                                    popup_width,
                                ));

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

        let details_projection = SymbolLayoutDetails::build_layout_projection(
            struct_layout_descriptor.get_struct_layout_id(),
            struct_layout_descriptor
                .get_struct_layout_definition()
                .get_layout_kind(),
        );
        let selection_key = format!("layout|{}", struct_layout_descriptor.get_struct_layout_id());
        let edit_callback = Self::build_struct_viewer_layout_edit_callback(
            self.app_context.clone(),
            self.struct_viewer_view_data.clone(),
            struct_layout_descriptor.get_struct_layout_id().to_string(),
        );

        StructViewerViewData::focus_details_projection_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_projection,
            edit_callback,
            Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
        );
    }

    fn build_struct_viewer_layout_edit_callback(
        app_context: Arc<AppContext>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        layout_id: String,
    ) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
        Arc::new(move |details_edit: DetailsEdit| {
            let SymbolLayoutDetailsEditOperation::UpdateLayoutKind(edited_layout_kind) = SymbolLayoutDetails::plan_edit(&details_edit) else {
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
                    StructLayoutDescriptor::new(
                        struct_layout_descriptor.get_struct_layout_id().to_string(),
                        SymbolicStructDefinition::new_with_layout_kind(
                            struct_layout_definition.get_symbol_namespace().to_string(),
                            edited_layout_kind,
                            struct_layout_definition.get_fields().to_vec(),
                        )
                        .with_declared_size_in_bytes(struct_layout_definition.get_declared_size_in_bytes()),
                    )
                })
                .collect::<Vec<_>>();

            if !did_update_layout {
                return;
            }

            let mut updated_project_symbol_catalog = project_symbol_catalog;
            updated_project_symbol_catalog.set_struct_layout_descriptors(updated_struct_layout_descriptors);

            let Some(updated_struct_layout_descriptor) = updated_project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
            else {
                return;
            };
            Self::persist_symbol_layout_descriptor_with_context(&app_context, Some(layout_id.clone()), updated_struct_layout_descriptor);
            let details_projection = SymbolLayoutDetails::build_layout_projection(
                updated_struct_layout_descriptor.get_struct_layout_id(),
                updated_struct_layout_descriptor
                    .get_struct_layout_definition()
                    .get_layout_kind(),
            );
            let selection_key = format!("layout|{}", updated_struct_layout_descriptor.get_struct_layout_id());
            let edit_callback = Self::build_struct_viewer_layout_edit_callback(app_context.clone(), struct_viewer_view_data.clone(), layout_id.clone());

            StructViewerViewData::focus_details_projection_with_focus_target(
                struct_viewer_view_data.clone(),
                app_context.engine_unprivileged_state.clone(),
                details_projection,
                edit_callback,
                Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
            );
        })
    }

    fn focus_unassigned_span_in_struct_viewer(
        &self,
        draft: &SymbolLayoutEditDraft,
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) {
        let details_projection = SymbolLayoutDetails::build_unassigned_projection(&draft.layout_id, offset_in_bytes, size_in_bytes);
        let selection_key = format!("unassigned|{}|{}|{}", draft.layout_id, offset_in_bytes, size_in_bytes);

        StructViewerViewData::focus_details_projection_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_projection,
            Arc::new(|_details_edit| {}),
            Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
        );
    }

    fn build_field_details(
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_kind: SymbolicLayoutKind,
        field_draft: &SymbolLayoutFieldEditDraft,
    ) -> SymbolLayoutFieldDetails {
        let element_type = SymbolLayoutEditorViewData::resolve_field_element_type(project_symbol_catalog, field_draft);
        let fixed_array_length = matches!(
            field_draft.container_edit.kind,
            SymbolLayoutFieldContainerKind::FixedArray | SymbolLayoutFieldContainerKind::FixedPointerArray
        )
        .then(|| {
            field_draft
                .container_edit
                .fixed_array_length
                .trim()
                .parse::<u64>()
                .unwrap_or(1)
                .max(1)
        });
        let uses_count_resolver = matches!(
            field_draft.container_edit.kind,
            SymbolLayoutFieldContainerKind::DynamicArray | SymbolLayoutFieldContainerKind::DynamicPointerArray
        );
        let uses_display_count_resolver = matches!(
            field_draft.container_edit.kind,
            SymbolLayoutFieldContainerKind::FixedArray
                | SymbolLayoutFieldContainerKind::FixedPointerArray
                | SymbolLayoutFieldContainerKind::DynamicArray
                | SymbolLayoutFieldContainerKind::DynamicPointerArray
        );
        let uses_pointer_size = matches!(
            field_draft.container_edit.kind,
            SymbolLayoutFieldContainerKind::Pointer | SymbolLayoutFieldContainerKind::FixedPointerArray | SymbolLayoutFieldContainerKind::DynamicPointerArray
        );

        SymbolLayoutFieldDetails {
            field_name: field_draft.field_name.clone(),
            data_type_id: field_draft
                .data_type_selection
                .visible_data_type()
                .get_data_type_id()
                .to_string(),
            element_kind: match element_type {
                SymbolLayoutFieldElementType::BuiltInDataType => SymbolLayoutDetailsFieldElementKind::BuiltInDataType,
                SymbolLayoutFieldElementType::SymbolLayout => SymbolLayoutDetailsFieldElementKind::SymbolLayout,
            },
            container_kind_label: field_draft.container_edit.kind.label().to_string(),
            fixed_array_length,
            count_resolver_id: uses_count_resolver.then(|| {
                field_draft
                    .container_edit
                    .dynamic_array_count_resolver_id
                    .clone()
            }),
            display_count_resolver_id: uses_display_count_resolver.then(|| field_draft.container_edit.display_count_resolver_id.clone()),
            active_when_resolver_id: (layout_kind.is_union() || !field_draft.active_when_resolver_id.is_empty())
                .then(|| field_draft.active_when_resolver_id.clone()),
            pointer_size_label: uses_pointer_size.then(|| field_draft.container_edit.pointer_size.to_string()),
            offset_resolver_id: (field_draft.offset_mode == SymbolLayoutFieldOffsetMode::Resolver).then(|| field_draft.offset_resolver_id.clone()),
        }
    }

    #[cfg(test)]
    fn build_field_details_struct(
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_kind: SymbolicLayoutKind,
        field_draft: &SymbolLayoutFieldEditDraft,
    ) -> squalr_engine_api::structures::structs::valued_struct::ValuedStruct {
        use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;

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
                DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.active_when_resolver_id)
                    .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_ACTIVE_WHEN_RESOLVER.to_string(), false),
            ]);
        }

        let mut fields = vec![
            DataTypeStringUtf8::get_value_from_primitive_string(&field_draft.field_name)
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_NAME.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(element_type.label())
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_ELEMENT_TYPE.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(field_draft.container_edit.kind.label())
                .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_SYMBOL_LAYOUT_FIELD_CONTAINER_KIND.to_string(), false),
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
    ) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
        Arc::new(move |details_edit: DetailsEdit| {
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
                Self::apply_field_details_operation(&project_symbol_catalog, field_draft, SymbolLayoutDetails::plan_edit(&details_edit));
                Self::grow_draft_size_to_fit_fields(&project_symbol_catalog, &mut draft);
                view_data.replace_draft(draft.clone());
                draft
            };

            let Some(updated_field_draft) = updated_draft.field_drafts.get(field_index) else {
                return;
            };
            let project_symbol_catalog = Self::get_opened_project_symbol_catalog_from_context(&app_context).unwrap_or_default();
            let details_projection = SymbolLayoutDetails::build_field_projection(
                &updated_draft.layout_id,
                field_index,
                updated_draft.layout_kind,
                &Self::build_field_details(&project_symbol_catalog, updated_draft.layout_kind, updated_field_draft),
            );
            let selection_key = format!("field|{}|{}", updated_draft.layout_id, field_index);
            let edit_callback = Self::build_struct_viewer_field_edit_callback(
                symbol_layout_editor_view_data.clone(),
                struct_viewer_view_data.clone(),
                app_context.clone(),
                field_index,
            );

            StructViewerViewData::focus_details_projection_with_focus_target(
                struct_viewer_view_data.clone(),
                app_context.engine_unprivileged_state.clone(),
                details_projection,
                edit_callback,
                Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
            );
        })
    }

    fn build_variant_field_edit_callback(
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        app_context: Arc<AppContext>,
        variant_layout_id: String,
        field_index: usize,
    ) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
        Arc::new(move |details_edit: DetailsEdit| {
            let project_symbol_catalog = Self::get_opened_project_symbol_catalog_from_context(&app_context).unwrap_or_default();
            let updated_variant_draft = {
                let Some(mut view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor apply variant field details edit") else {
                    return;
                };
                let Some(union_draft) = view_data.get_draft().cloned() else {
                    return;
                };
                let mut variant_draft = view_data
                    .get_pending_variant_draft(&variant_layout_id)
                    .cloned()
                    .unwrap_or_else(|| Self::create_union_variant_layout_draft_for_id(&project_symbol_catalog, &union_draft, &variant_layout_id));
                let Some(field_draft) = variant_draft.field_drafts.get_mut(field_index) else {
                    return;
                };

                Self::apply_field_details_operation(&project_symbol_catalog, field_draft, SymbolLayoutDetails::plan_edit(&details_edit));
                Self::grow_draft_size_to_fit_fields(&project_symbol_catalog, &mut variant_draft);
                view_data.replace_pending_variant_draft(variant_draft.clone());
                variant_draft
            };
            SymbolLayoutEditorViewData::select_field_for_layout(symbol_layout_editor_view_data.clone(), Some(variant_layout_id.clone()), field_index);

            let updated_project_symbol_catalog = Self::build_effective_project_symbol_catalog_from_view_data(
                &project_symbol_catalog,
                symbol_layout_editor_view_data.clone(),
                Some(&variant_layout_id),
            );
            let details_projection = updated_variant_draft
                .field_drafts
                .get(field_index)
                .map(|field_draft| {
                    SymbolLayoutDetails::build_field_projection(
                        &updated_variant_draft.layout_id,
                        field_index,
                        SymbolicLayoutKind::Struct,
                        &Self::build_field_details(&updated_project_symbol_catalog, SymbolicLayoutKind::Struct, field_draft),
                    )
                });
            let Some(details_projection) = details_projection else {
                return;
            };
            let selection_key = format!("field|{}|{}", variant_layout_id, field_index);
            let edit_callback = Self::build_variant_field_edit_callback(
                symbol_layout_editor_view_data.clone(),
                struct_viewer_view_data.clone(),
                app_context.clone(),
                variant_layout_id.clone(),
                field_index,
            );

            StructViewerViewData::focus_details_projection_with_focus_target(
                struct_viewer_view_data.clone(),
                app_context.engine_unprivileged_state.clone(),
                details_projection,
                edit_callback,
                Some(StructViewerFocusTarget::SymbolLayoutEditor { selection_key }),
            );
        })
    }

    fn apply_field_details_operation(
        project_symbol_catalog: &ProjectSymbolCatalog,
        field_draft: &mut SymbolLayoutFieldEditDraft,
        edit_operation: SymbolLayoutDetailsEditOperation,
    ) {
        match edit_operation {
            SymbolLayoutDetailsEditOperation::UpdateFieldName(field_name) => {
                field_draft.field_name = field_name;
            }
            SymbolLayoutDetailsEditOperation::UpdateFieldElementKind(element_kind) => {
                Self::apply_field_element_type_edit(project_symbol_catalog, field_draft, element_kind.label());
            }
            SymbolLayoutDetailsEditOperation::UpdateFieldDataType(data_type_id) | SymbolLayoutDetailsEditOperation::UpdateFieldSymbolLayout(data_type_id) => {
                field_draft
                    .data_type_selection
                    .replace_selected_data_types(vec![DataTypeRef::new(data_type_id.trim())]);
            }
            SymbolLayoutDetailsEditOperation::UpdateFieldContainerKind(container_kind_label) => {
                if let Some(container_kind) = Self::container_kind_from_label(&container_kind_label) {
                    field_draft.container_edit.kind = container_kind;
                }
            }
            SymbolLayoutDetailsEditOperation::UpdateFieldFixedArrayLength(length) => {
                field_draft.container_edit.fixed_array_length = length.max(1).to_string();
            }
            SymbolLayoutDetailsEditOperation::UpdateFieldCountResolver(count_resolver_id) => {
                field_draft.container_edit.dynamic_array_count_resolver_id = count_resolver_id;
            }
            SymbolLayoutDetailsEditOperation::UpdateFieldDisplayCountResolver(display_count_resolver_id) => {
                field_draft.container_edit.display_count_resolver_id = display_count_resolver_id;
            }
            SymbolLayoutDetailsEditOperation::UpdateFieldActiveWhenResolver(active_when_resolver_id) => {
                field_draft.active_when_resolver_id = active_when_resolver_id;
            }
            SymbolLayoutDetailsEditOperation::UpdateFieldPointerSize(pointer_size_label) => {
                if let Ok(pointer_size) = PointerScanPointerSize::from_str(pointer_size_label.trim()) {
                    field_draft.container_edit.pointer_size = pointer_size;
                }
            }
            SymbolLayoutDetailsEditOperation::UpdateFieldOffsetResolver(offset_resolver_id) => {
                field_draft.offset_resolver_id = offset_resolver_id;
            }
            SymbolLayoutDetailsEditOperation::UpdateLayoutKind(_) | SymbolLayoutDetailsEditOperation::NoOp | SymbolLayoutDetailsEditOperation::Reject(_) => {}
        }
    }

    fn grow_draft_size_to_fit_fields(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &mut SymbolLayoutEditDraft,
    ) {
        let Ok(declared_size_in_bytes) = SymbolLayoutEditorViewData::parse_layout_size_text(&draft.size_text, draft.size_format) else {
            return;
        };
        let mut next_sequential_offset = 0_u64;

        for field_draft in &draft.field_drafts {
            let Ok(symbolic_field_definition) = Self::build_symbolic_field_definition_from_draft(field_draft) else {
                continue;
            };
            let field_offset = match symbolic_field_definition.get_offset_resolution() {
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) if draft.layout_kind.is_union() => 0,
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
            };
            let field_size_in_bytes = SymbolLayoutEditorViewData::resolve_symbolic_field_size_in_bytes(
                project_symbol_catalog,
                &symbolic_field_definition,
                &mut std::collections::HashSet::new(),
            );

            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
        }

        if next_sequential_offset > declared_size_in_bytes {
            draft.size_text = Self::format_layout_size(next_sequential_offset, draft.size_format);
        }
    }

    fn format_layout_size(
        size_in_bytes: u64,
        size_format: AnonymousValueStringFormat,
    ) -> String {
        match size_format {
            AnonymousValueStringFormat::Binary => format!("{:b}", size_in_bytes),
            AnonymousValueStringFormat::Hexadecimal | AnonymousValueStringFormat::Address => format!("{:X}", size_in_bytes),
            _ => size_in_bytes.to_string(),
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
        user_interface.add(
            SymbolLayoutListToolbarView::new(
                self.app_context.clone(),
                self.symbol_layout_editor_view_data.clone(),
                project_symbol_catalog,
                self.default_data_type_ref(),
                is_take_over_active,
            )
            .height(Self::TOOLBAR_HEIGHT)
            .icon_button_size(Self::ICON_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT),
        );

        self.render_filter_text_box(user_interface, filter_text);

        user_interface.add(
            ListHeaderView::new(self.app_context.clone(), "Symbol Layout", "Kind | Entries | Uses")
                .height(Self::LIST_ROW_HEIGHT)
                .horizontal_padding(8.0),
        );
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
                    let row_action = SymbolLayoutRowView::new(
                        self.app_context.clone(),
                        struct_layout_id,
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_layout_kind(),
                        field_count,
                        usage_count,
                        selected_layout_id == Some(struct_layout_id),
                    )
                    .show(user_interface);
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

    fn render_centered_add_entry_button(
        &self,
        user_interface: &mut Ui,
        tooltip_text: &str,
        is_enabled: bool,
    ) -> bool {
        let button_size = vec2(Self::TAKE_OVER_ACTION_BUTTON_WIDTH, Self::FIELD_ROW_HEIGHT);

        user_interface
            .horizontal(|user_interface| {
                let theme = &self.app_context.theme;
                let leading_button_margin = (user_interface.available_width() - button_size.x).max(0.0) * 0.5;
                user_interface.add_space(leading_button_margin);

                let button_response = user_interface.add_sized(
                    button_size,
                    ThemeButton::new_from_theme(theme)
                        .with_tooltip_text(tooltip_text)
                        .corner_radius(CornerRadius::same(Self::FIELD_ADD_BUTTON_CORNER_RADIUS))
                        .background_color(theme.background_control_secondary)
                        .border_color(theme.background_control_secondary_dark)
                        .border_width(1.0)
                        .disabled(!is_enabled),
                );

                IconDraw::draw_tinted(
                    user_interface,
                    button_response.rect,
                    &theme.icon_library.icon_handle_common_add,
                    if is_enabled { theme.foreground } else { theme.foreground_preview },
                );

                is_enabled && button_response.clicked()
            })
            .inner
    }

    fn render_union_variant_child_row<R>(
        user_interface: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> R {
        user_interface
            .horizontal(|user_interface| {
                user_interface.spacing_mut().item_spacing.x = 0.0;
                user_interface.add_space(Self::UNION_VARIANT_CHILD_INDENT);
                user_interface
                    .allocate_ui_with_layout(vec2(user_interface.available_width().max(1.0), 0.0), Layout::top_down(Align::Min), add_contents)
                    .inner
            })
            .inner
    }

    fn render_union_variant_layout_rows(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        union_draft: &SymbolLayoutEditDraft,
        variant_index: usize,
        variant_field_draft: &SymbolLayoutFieldEditDraft,
        selected_field_layout_id: Option<&str>,
        selected_field_index: Option<usize>,
        selected_unassigned_span: Option<&SymbolLayoutUnassignedSelection>,
    ) -> Option<SymbolLayoutVariantLayoutRowAction> {
        let mut variant_draft = self.create_union_variant_layout_draft_with_pending(project_symbol_catalog, union_draft, variant_index, variant_field_draft);
        let variant_layout_id = variant_draft.layout_id.clone();

        let Some((layout_size_in_bytes, mut field_spans)) = self.resolve_draft_field_spans(project_symbol_catalog, &variant_draft) else {
            Self::render_union_variant_child_row(user_interface, |user_interface| {
                UnionVariantPreviewRowView::new(self.app_context.clone(), "UNASSIGNED", "variant layout unresolved").show(user_interface);
            });
            return None;
        };
        let unassigned_split_offsets = self
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
                    let is_selected = selected_unassigned_span.is_some_and(|selected_unassigned_span| {
                        selected_unassigned_span.matches(
                            Some(variant_layout_id.as_str()),
                            unassigned_row_context.offset_in_bytes,
                            unassigned_row_context.size_in_bytes,
                        )
                    });
                    let unassigned_row_action = Self::render_union_variant_child_row(user_interface, |user_interface| {
                        SymbolLayoutUnassignedRowView::new(
                            self.app_context.clone(),
                            self.symbol_layout_editor_view_data.clone(),
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
            let is_selected = selected_field_layout_id == Some(variant_layout_id.as_str()) && selected_field_index == Some(field_span.field_position);
            if let Some(field_draft) = variant_draft.field_drafts.get_mut(field_span.field_position) {
                let field_row_action = Self::render_union_variant_child_row(user_interface, |user_interface| {
                    SymbolLayoutFieldRowView::new(
                        self.app_context.clone(),
                        self.symbol_layout_editor_view_data.clone(),
                        project_symbol_catalog,
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
                let is_selected = selected_unassigned_span.is_some_and(|selected_unassigned_span| {
                    selected_unassigned_span.matches(
                        Some(variant_layout_id.as_str()),
                        unassigned_row_context.offset_in_bytes,
                        unassigned_row_context.size_in_bytes,
                    )
                });
                let unassigned_row_action = Self::render_union_variant_child_row(user_interface, |user_interface| {
                    SymbolLayoutUnassignedRowView::new(
                        self.app_context.clone(),
                        self.symbol_layout_editor_view_data.clone(),
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
            && let Some(field_row_action) = self.render_field_context_menu(
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

    fn render_field_context_menu(
        &self,
        user_interface: &mut Ui,
        layout_kind: SymbolicLayoutKind,
        context_menu_target: &SymbolLayoutFieldContextMenuTarget,
        field_count: usize,
        can_delete_final_field: bool,
    ) -> Option<SymbolLayoutFieldRowAction> {
        let theme = &self.app_context.theme;
        let field_index = context_menu_target.get_field_index();
        let can_remove_field = can_delete_final_field || field_count > 1;
        let can_move_up = field_index > 0;
        let can_move_down = field_index + 1 < field_count;
        let mut open = true;
        let mut pending_field_row_action = None;
        let entry_name = if layout_kind.is_union() { "variant" } else { "field" };
        let context_menu_id = context_menu_target
            .get_layout_id()
            .map(|layout_id| format!("symbol_layout_field_context_menu_{}", layout_id))
            .unwrap_or_else(|| String::from("symbol_layout_field_context_menu"));

        ContextMenu::new(
            self.app_context.clone(),
            &context_menu_id,
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
        let mut visible_field_position = 0_usize;

        for symbolic_field_definition in symbolic_struct_definition.get_fields() {
            if symbolic_field_definition.is_unassigned() {
                next_sequential_offset = next_sequential_offset.saturating_add(
                    symbolic_field_definition
                        .get_unassigned_size_in_bytes()
                        .unwrap_or(0),
                );
                continue;
            }

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
                field_position: visible_field_position,
                offset_in_bytes: field_offset,
                size_in_bytes: field_size_in_bytes,
            });
            visible_field_position = visible_field_position.saturating_add(1);
        }

        Some((layout_size_in_bytes, field_spans))
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
        selected_field_layout_id: Option<&str>,
        selected_unassigned_span: Option<&SymbolLayoutUnassignedSelection>,
    ) {
        let field_count = draft.field_drafts.len();
        let layout_kind = draft.layout_kind;
        let mut pending_field_row_action = None;
        let mut pending_variant_field_row_action = None;
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
                let union_draft_preview = draft.clone();
                let Some(field_draft) = draft.field_drafts.get_mut(field_index) else {
                    continue;
                };

                if let Some(field_row_action) = SymbolLayoutFieldRowView::new(
                    self.app_context.clone(),
                    self.symbol_layout_editor_view_data.clone(),
                    project_symbol_catalog,
                    layout_kind,
                    field_draft,
                    field_index,
                    selected_field_layout_id.is_none() && selected_field_index == Some(field_index),
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
                if let Some(variant_layout_action) = self.render_union_variant_layout_rows(
                    user_interface,
                    project_symbol_catalog,
                    &union_draft_preview,
                    field_index,
                    &variant_field_preview_draft,
                    selected_field_layout_id,
                    selected_field_index,
                    selected_unassigned_span,
                ) {
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

                let variant_tail_unassigned_offset =
                    self.resolve_variant_tail_unassigned_offset(project_symbol_catalog, &union_draft_preview, field_index, &variant_field_preview_draft);
                if Self::render_union_variant_child_row(user_interface, |user_interface| {
                    self.render_centered_add_entry_button(
                        user_interface,
                        "Add a new field to this union variant.",
                        variant_tail_unassigned_offset.is_some(),
                    )
                }) {
                    pending_field_row_action = Some((field_index, SymbolLayoutFieldRowAction::InsertFieldIntoVariant));
                }

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
                        for unassigned_row_context in SymbolLayoutDraftOps::build_unassigned_row_contexts(
                            next_visible_offset,
                            unassigned_size,
                            &unassigned_split_offsets,
                            previous_visible_field,
                            move_down_field,
                        ) {
                            let is_selected = selected_unassigned_span.is_some_and(|selected_unassigned_span| {
                                selected_unassigned_span.matches(None, unassigned_row_context.offset_in_bytes, unassigned_row_context.size_in_bytes)
                            });
                            if let Some(unassigned_row_action) = SymbolLayoutUnassignedRowView::new(
                                self.app_context.clone(),
                                self.symbol_layout_editor_view_data.clone(),
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
                    self.app_context.clone(),
                    self.symbol_layout_editor_view_data.clone(),
                    project_symbol_catalog,
                    layout_kind,
                    field_draft,
                    field_index,
                    selected_field_layout_id.is_none() && selected_field_index == Some(field_index),
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
                let is_selected = selected_unassigned_span.is_some_and(|selected_unassigned_span| {
                    selected_unassigned_span.matches(None, unassigned_row_context.offset_in_bytes, unassigned_row_context.size_in_bytes)
                });
                if let Some(unassigned_row_action) = SymbolLayoutUnassignedRowView::new(
                    self.app_context.clone(),
                    self.symbol_layout_editor_view_data.clone(),
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
            && let Some(field_row_action) = self.render_field_context_menu(user_interface, draft.layout_kind, &field_context_menu_target, field_count, false)
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
                .map(|target_layout_id| self.create_union_variant_layout_draft_for_id_with_pending(project_symbol_catalog, draft, target_layout_id));
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
                    field_draft.field_name = SymbolLayoutDraftOps::build_unique_field_name(draft, &field_draft.field_name);
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
                        SymbolLayoutDraftOps::move_unassigned_span_up(target_variant_draft, unassigned_row_context.clone())
                    } else {
                        SymbolLayoutDraftOps::move_unassigned_span_up(draft, unassigned_row_context.clone())
                    };

                    if let Some(updated_unassigned_selection) = updated_unassigned_selection {
                        persist_target_variant_draft = target_layout_id.is_some();
                        if let Some(split_offset_in_bytes) = SymbolLayoutDraftOps::split_offset_to_preserve_unassigned_move_up(&updated_unassigned_selection) {
                            SymbolLayoutEditorViewData::insert_unassigned_split_offset_for_layout(
                                self.symbol_layout_editor_view_data.clone(),
                                target_layout_id.clone(),
                                split_offset_in_bytes,
                            );
                        }
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
                        SymbolLayoutDraftOps::move_unassigned_span_down(target_variant_draft, unassigned_row_context.clone())
                    } else {
                        SymbolLayoutDraftOps::move_unassigned_span_down(draft, unassigned_row_context.clone())
                    };

                    if let Some(updated_unassigned_selection) = updated_unassigned_selection {
                        persist_target_variant_draft = target_layout_id.is_some();
                        if let Some(split_offset_in_bytes) = SymbolLayoutDraftOps::split_offset_to_preserve_unassigned_move_down(&updated_unassigned_selection)
                        {
                            SymbolLayoutEditorViewData::insert_unassigned_split_offset_for_layout(
                                self.symbol_layout_editor_view_data.clone(),
                                target_layout_id.clone(),
                                split_offset_in_bytes,
                            );
                        }
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
                self.persist_variant_layout_draft(target_variant_draft);
            }
        }

        if let Some((variant_layout_id, field_index, field_row_action)) = pending_variant_field_row_action {
            field_row_action.apply_to_variant_layout(self, project_symbol_catalog, draft, variant_layout_id, field_index);
        }

        if let Some((field_index, field_row_action)) = pending_field_row_action {
            field_row_action.apply_to_layout_draft(
                self,
                project_symbol_catalog,
                draft,
                field_index,
                field_spans.as_ref(),
                &unassigned_split_offsets,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::rows::symbol_layout_field_row_view::SymbolLayoutFieldRowView;
    use super::{SymbolLayoutEditorView, SymbolLayoutFieldContainerKind, SymbolLayoutFieldEditDraft};
    use crate::views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData;
    use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{
        SymbolLayoutEditDraft, SymbolLayoutEditorViewData, SymbolLayoutFieldOffsetMode,
    };
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
        let details_struct = SymbolLayoutEditorView::build_field_details_struct(&ProjectSymbolCatalog::default(), SymbolicLayoutKind::Struct, &field_draft);

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

        SymbolLayoutEditorView::grow_draft_size_to_fit_fields(&ProjectSymbolCatalog::default(), &mut draft);

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

        let effective_project_symbol_catalog =
            SymbolLayoutEditorView::build_effective_project_symbol_catalog_from_pending_drafts(&project_symbol_catalog, &[(variant_draft, BTreeSet::new())]);

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
