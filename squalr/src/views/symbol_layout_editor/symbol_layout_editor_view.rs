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
    SymbolLayoutEditDraft, SymbolLayoutEditorTakeOverState, SymbolLayoutEditorViewData, SymbolLayoutFieldEditDraft, SymbolLayoutFieldOffsetMode,
};
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::SymbolLayoutFieldContainerKind;
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
use squalr_engine_api::structures::{
    data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
    projects::{
        project_symbol_catalog::ProjectSymbolCatalog,
        symbol_layouts::symbol_layout_draft_ops::{SymbolLayoutDraftOps, SymbolLayoutFieldSpan},
    },
    structs::{
        symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_struct_definition::SymbolicLayoutKind,
    },
};
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
}

#[cfg(test)]
mod tests {
    use super::details::symbol_layout_details_focus::build_field_details_struct;
    use super::rows::symbol_layout_field_row_action::grow_draft_size_to_fit_fields;
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
                            self.default_data_type_ref(),
                            false,
                        )
                        .show(&mut content_user_interface);
                    }
                }
            })
            .response
    }
}
