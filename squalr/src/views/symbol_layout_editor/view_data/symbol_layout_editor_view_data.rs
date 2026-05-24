use crate::ui::list_navigation::{ListNavigationDirection, resolve_next_index};
use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::SymbolLayoutFieldContainerEdit;
use epaint::Pos2;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::{
    data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
    data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    projects::{
        project_symbol_catalog::ProjectSymbolCatalog,
        symbol_layouts::{
            symbol_layout_descriptor_builder::{SymbolLayoutDescriptorBuildTarget, SymbolLayoutDescriptorBuilder, SymbolLayoutDescriptorFieldBuildTarget},
            symbol_layout_draft_ops::{SymbolLayoutDraftMutationTarget, SymbolLayoutUnassignedSelection},
        },
    },
    structs::{
        symbolic_field_definition::{SymbolicFieldCountResolution, SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_resolver_definition::SymbolicResolverRef,
        symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
    },
};
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SymbolLayoutFieldOffsetMode {
    #[default]
    Sequential,
    Static,
    Resolver,
}

impl SymbolLayoutFieldOffsetMode {
    pub const ALL: [Self; 3] = [Self::Sequential, Self::Static, Self::Resolver];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Sequential => "Sequential",
            Self::Static => "Static",
            Self::Resolver => "Resolver",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SymbolLayoutFieldElementType {
    #[default]
    BuiltInDataType,
    SymbolLayout,
}

impl SymbolLayoutFieldElementType {
    pub const ALL: [Self; 2] = [Self::BuiltInDataType, Self::SymbolLayout];

    pub fn label(&self) -> &'static str {
        match self {
            Self::BuiltInDataType => "Data Type",
            Self::SymbolLayout => "Symbol Layout",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolLayoutFieldEditDraft {
    pub field_name: String,
    pub data_type_selection: DataTypeSelection,
    pub container_edit: SymbolLayoutFieldContainerEdit,
    pub active_when_resolver_id: String,
    pub display_format: Option<AnonymousValueStringFormat>,
    pub offset_mode: SymbolLayoutFieldOffsetMode,
    pub static_offset_in_bytes: String,
    pub offset_resolver_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolLayoutEditDraft {
    pub original_layout_id: Option<String>,
    pub layout_id: String,
    pub layout_kind: SymbolicLayoutKind,
    pub size_text: String,
    pub size_format: AnonymousValueStringFormat,
    pub field_drafts: Vec<SymbolLayoutFieldEditDraft>,
}

impl SymbolLayoutDraftMutationTarget for SymbolLayoutEditDraft {
    fn get_layout_kind(&self) -> SymbolicLayoutKind {
        self.layout_kind
    }

    fn get_field_count(&self) -> usize {
        self.field_drafts.len()
    }

    fn get_field_name(
        &self,
        field_position: usize,
    ) -> Option<&str> {
        self.field_drafts
            .get(field_position)
            .map(|field_draft| field_draft.field_name.as_str())
    }

    fn set_field_static_offset(
        &mut self,
        field_position: usize,
        offset_in_bytes: u64,
    ) -> bool {
        let Some(field_draft) = self.field_drafts.get_mut(field_position) else {
            return false;
        };

        field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = offset_in_bytes.to_string();
        true
    }
}

impl SymbolLayoutDescriptorBuildTarget for SymbolLayoutEditDraft {
    type Field = SymbolLayoutFieldEditDraft;

    fn get_original_layout_id(&self) -> Option<&str> {
        self.original_layout_id.as_deref()
    }

    fn get_layout_id(&self) -> &str {
        &self.layout_id
    }

    fn get_layout_kind(&self) -> SymbolicLayoutKind {
        self.layout_kind
    }

    fn get_size_text(&self) -> &str {
        &self.size_text
    }

    fn get_size_format(&self) -> AnonymousValueStringFormat {
        self.size_format
    }

    fn get_field_count(&self) -> usize {
        self.field_drafts.len()
    }

    fn get_field(
        &self,
        field_position: usize,
    ) -> Option<&Self::Field> {
        self.field_drafts.get(field_position)
    }
}

impl SymbolLayoutDescriptorFieldBuildTarget for SymbolLayoutFieldEditDraft {
    fn get_field_name(&self) -> &str {
        &self.field_name
    }

    fn get_data_type_id(&self) -> &str {
        self.data_type_selection.visible_data_type().get_data_type_id()
    }

    fn to_container_type(&self) -> Result<ContainerType, String> {
        self.container_edit.to_container_type()
    }

    fn to_count_resolution(&self) -> Result<SymbolicFieldCountResolution, String> {
        self.container_edit.to_count_resolution()
    }

    fn to_display_count_resolution(&self) -> Result<SymbolicFieldCountResolution, String> {
        self.container_edit.to_display_count_resolution()
    }

    fn to_offset_resolution(&self) -> Result<SymbolicFieldOffsetResolution, String> {
        SymbolLayoutFieldEditDraft::to_offset_resolution(self)
    }

    fn to_active_when_resolver(&self) -> Option<SymbolicResolverRef> {
        SymbolLayoutFieldEditDraft::to_active_when_resolver(self)
    }

    fn to_display_format(&self) -> Option<AnonymousValueStringFormat> {
        self.display_format
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolLayoutDefineFieldReturnState {
    CreateSymbolLayout,
    RenameSymbolLayout { layout_id: String },
    OpenSymbolLayout { layout_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolLayoutEditorTakeOverState {
    CreateSymbolLayout,
    RenameSymbolLayout {
        layout_id: String,
    },
    OpenSymbolLayout {
        layout_id: String,
    },
    DefineFieldFromUnassignedSpan {
        layout_id: String,
        offset: u64,
        size: u64,
        return_state: SymbolLayoutDefineFieldReturnState,
    },
    DeleteConfirmation {
        layout_id: String,
    },
    UnassignFieldConfirmation {
        layout_id: String,
        field_index: usize,
        return_state: SymbolLayoutDefineFieldReturnState,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolLayoutFieldContextMenuTarget {
    layout_id: Option<String>,
    field_index: usize,
    position: Pos2,
}

impl SymbolLayoutFieldContextMenuTarget {
    pub fn new(
        field_index: usize,
        position: Pos2,
    ) -> Self {
        Self {
            layout_id: None,
            field_index,
            position,
        }
    }

    pub fn with_layout_id(
        mut self,
        layout_id: String,
    ) -> Self {
        self.layout_id = Some(layout_id);
        self
    }

    pub fn get_layout_id(&self) -> Option<&str> {
        self.layout_id.as_deref()
    }

    pub fn get_field_index(&self) -> usize {
        self.field_index
    }

    pub fn get_position(&self) -> Pos2 {
        self.position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolLayoutUnassignedContextMenuTarget {
    layout_id: Option<String>,
    offset_in_bytes: u64,
    size_in_bytes: u64,
    position: Pos2,
    merge_above_span: Option<SymbolLayoutUnassignedSelection>,
    merge_below_span: Option<SymbolLayoutUnassignedSelection>,
}

impl SymbolLayoutUnassignedContextMenuTarget {
    pub fn new(
        offset_in_bytes: u64,
        size_in_bytes: u64,
        position: Pos2,
    ) -> Self {
        Self {
            layout_id: None,
            offset_in_bytes,
            size_in_bytes,
            position,
            merge_above_span: None,
            merge_below_span: None,
        }
    }

    pub fn with_merge_spans(
        mut self,
        merge_above_span: Option<SymbolLayoutUnassignedSelection>,
        merge_below_span: Option<SymbolLayoutUnassignedSelection>,
    ) -> Self {
        self.merge_above_span = merge_above_span;
        self.merge_below_span = merge_below_span;
        self
    }

    pub fn with_layout_id(
        mut self,
        layout_id: String,
    ) -> Self {
        self.layout_id = Some(layout_id);
        self
    }

    pub fn get_layout_id(&self) -> Option<&str> {
        self.layout_id.as_deref()
    }

    pub fn get_offset_in_bytes(&self) -> u64 {
        self.offset_in_bytes
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.size_in_bytes
    }

    pub fn get_position(&self) -> Pos2 {
        self.position
    }

    pub fn get_merge_above_span(&self) -> Option<&SymbolLayoutUnassignedSelection> {
        self.merge_above_span.as_ref()
    }

    pub fn get_merge_below_span(&self) -> Option<&SymbolLayoutUnassignedSelection> {
        self.merge_below_span.as_ref()
    }
}

#[derive(Clone, Default)]
pub struct SymbolLayoutEditorViewData {
    selected_layout_id: Option<String>,
    filter_text: String,
    take_over_state: Option<SymbolLayoutEditorTakeOverState>,
    baseline_project_symbol_catalog: Option<ProjectSymbolCatalog>,
    baseline_draft: Option<SymbolLayoutEditDraft>,
    draft: Option<SymbolLayoutEditDraft>,
    pending_variant_drafts: BTreeMap<String, SymbolLayoutEditDraft>,
    selected_field_layout_id: Option<String>,
    selected_field_index: Option<usize>,
    selected_unassigned_span: Option<SymbolLayoutUnassignedSelection>,
    define_field_draft: Option<SymbolLayoutFieldEditDraft>,
    field_context_menu_target: Option<SymbolLayoutFieldContextMenuTarget>,
    unassigned_context_menu_target: Option<SymbolLayoutUnassignedContextMenuTarget>,
    unassigned_split_offsets: BTreeSet<u64>,
    unassigned_split_offsets_by_layout: BTreeMap<String, BTreeSet<u64>>,
}

impl SymbolLayoutEditorViewData {
    pub fn new() -> Self {
        Self {
            selected_layout_id: None,
            filter_text: String::new(),
            take_over_state: None,
            baseline_project_symbol_catalog: None,
            baseline_draft: None,
            draft: None,
            pending_variant_drafts: BTreeMap::new(),
            selected_field_layout_id: None,
            selected_field_index: None,
            selected_unassigned_span: None,
            define_field_draft: None,
            field_context_menu_target: None,
            unassigned_context_menu_target: None,
            unassigned_split_offsets: BTreeSet::new(),
            unassigned_split_offsets_by_layout: BTreeMap::new(),
        }
    }

    pub fn get_selected_layout_id(&self) -> Option<&str> {
        self.selected_layout_id.as_deref()
    }

    pub fn get_filter_text(&self) -> &str {
        &self.filter_text
    }

    pub fn get_take_over_state(&self) -> Option<&SymbolLayoutEditorTakeOverState> {
        self.take_over_state.as_ref()
    }

    pub fn get_draft(&self) -> Option<&SymbolLayoutEditDraft> {
        self.draft.as_ref()
    }

    pub fn get_baseline_draft(&self) -> Option<&SymbolLayoutEditDraft> {
        self.baseline_draft.as_ref()
    }

    pub fn get_baseline_project_symbol_catalog(&self) -> Option<&ProjectSymbolCatalog> {
        self.baseline_project_symbol_catalog.as_ref()
    }

    pub fn get_pending_variant_draft(
        &self,
        layout_id: &str,
    ) -> Option<&SymbolLayoutEditDraft> {
        self.pending_variant_drafts.get(layout_id)
    }

    pub fn get_pending_variant_drafts_with_split_offsets(&self) -> Vec<(SymbolLayoutEditDraft, BTreeSet<u64>)> {
        self.pending_variant_drafts
            .iter()
            .map(|(layout_id, draft)| {
                (
                    draft.clone(),
                    self.unassigned_split_offsets_by_layout
                        .get(layout_id)
                        .cloned()
                        .unwrap_or_default(),
                )
            })
            .collect()
    }

    pub fn replace_pending_variant_draft(
        &mut self,
        variant_draft: SymbolLayoutEditDraft,
    ) {
        self.pending_variant_drafts
            .insert(variant_draft.layout_id.clone(), variant_draft);
    }

    pub fn get_selected_field_index(&self) -> Option<usize> {
        self.selected_field_index
    }

    pub fn get_selected_field_layout_id(&self) -> Option<&str> {
        self.selected_field_layout_id.as_deref()
    }

    pub fn get_selected_unassigned_span(&self) -> Option<&SymbolLayoutUnassignedSelection> {
        self.selected_unassigned_span.as_ref()
    }

    pub fn get_define_field_draft(&self) -> Option<&SymbolLayoutFieldEditDraft> {
        self.define_field_draft.as_ref()
    }

    pub fn get_field_context_menu_target(&self) -> Option<&SymbolLayoutFieldContextMenuTarget> {
        self.field_context_menu_target.as_ref()
    }

    pub fn get_unassigned_context_menu_target(&self) -> Option<&SymbolLayoutUnassignedContextMenuTarget> {
        self.unassigned_context_menu_target.as_ref()
    }

    pub fn get_unassigned_split_offsets(&self) -> &BTreeSet<u64> {
        &self.unassigned_split_offsets
    }

    pub fn get_unassigned_split_offsets_for_layout(
        &self,
        layout_id: Option<&str>,
    ) -> BTreeSet<u64> {
        match layout_id {
            Some(layout_id) => self
                .unassigned_split_offsets_by_layout
                .get(layout_id)
                .cloned()
                .unwrap_or_default(),
            None => self.unassigned_split_offsets.clone(),
        }
    }

    fn get_unassigned_split_offsets_mut(
        &mut self,
        layout_id: Option<&str>,
    ) -> &mut BTreeSet<u64> {
        match layout_id {
            Some(layout_id) => self
                .unassigned_split_offsets_by_layout
                .entry(layout_id.to_string())
                .or_default(),
            None => &mut self.unassigned_split_offsets,
        }
    }

    fn prune_unassigned_split_offsets(
        &mut self,
        layout_size_in_bytes: u64,
    ) {
        self.unassigned_split_offsets
            .retain(|split_offset| *split_offset > 0 && *split_offset < layout_size_in_bytes);
        self.unassigned_split_offsets_by_layout
            .retain(|_layout_id, split_offsets| {
                split_offsets.retain(|split_offset| *split_offset > 0 && *split_offset < layout_size_in_bytes);
                !split_offsets.is_empty()
            });
    }

    fn clear_unassigned_split_offsets(&mut self) {
        self.unassigned_split_offsets.clear();
        self.unassigned_split_offsets_by_layout.clear();
    }

    fn clear_pending_variant_drafts(&mut self) {
        self.pending_variant_drafts.clear();
    }

    pub fn clear_pending_variant_drafts_for_take_over(symbol_layout_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor clear pending variant drafts") {
            symbol_layout_editor_view_data.clear_pending_variant_drafts();
        }
    }

    fn collect_unassigned_split_offsets_from_descriptor(
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_descriptor: &StructLayoutDescriptor,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> BTreeSet<u64> {
        let mut split_offsets = BTreeSet::new();
        let mut next_sequential_offset = 0_u64;
        let mut previous_was_unassigned = false;

        for symbolic_field_definition in struct_layout_descriptor
            .get_struct_layout_definition()
            .get_fields()
        {
            if symbolic_field_definition.is_unassigned() {
                if previous_was_unassigned && next_sequential_offset > 0 {
                    split_offsets.insert(next_sequential_offset);
                }
                next_sequential_offset = next_sequential_offset.saturating_add(
                    symbolic_field_definition
                        .get_unassigned_size_in_bytes()
                        .unwrap_or(0),
                );
                previous_was_unassigned = true;
                continue;
            }

            previous_was_unassigned = false;
            let field_offset = match symbolic_field_definition.get_offset_resolution() {
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                    if struct_layout_descriptor
                        .get_struct_layout_definition()
                        .get_layout_kind()
                        .is_union() =>
                {
                    0
                }
                SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => next_sequential_offset,
            };
            let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(
                project_symbol_catalog,
                symbolic_field_definition,
                &mut HashSet::new(),
                resolve_data_type_size_in_bytes,
            );
            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
        }

        split_offsets
    }

    pub fn set_filter_text(
        symbol_layout_editor_view_data: Dependency<Self>,
        filter_text: String,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor set filter text") {
            symbol_layout_editor_view_data.filter_text = filter_text;
        }
    }

    pub fn update_draft(
        symbol_layout_editor_view_data: Dependency<Self>,
        draft: SymbolLayoutEditDraft,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor update draft") {
            symbol_layout_editor_view_data.replace_draft(draft);
        }
    }

    pub fn replace_draft(
        &mut self,
        draft: SymbolLayoutEditDraft,
    ) {
        if self
            .selected_field_index
            .is_some_and(|field_index| self.selected_field_layout_id.is_none() && field_index >= draft.field_drafts.len())
        {
            self.selected_field_layout_id = None;
            self.selected_field_index = None;
        }
        self.selected_unassigned_span = self
            .selected_unassigned_span
            .take()
            .filter(|selected_unassigned_span| {
                Self::parse_layout_size_text(&draft.size_text, draft.size_format).is_ok_and(|layout_size_in_bytes| {
                    selected_unassigned_span.get_offset_in_bytes() < layout_size_in_bytes
                        && selected_unassigned_span
                            .get_offset_in_bytes()
                            .saturating_add(selected_unassigned_span.get_size_in_bytes())
                            <= layout_size_in_bytes
                })
            });
        if self
            .field_context_menu_target
            .as_ref()
            .is_some_and(|context_menu_target| {
                context_menu_target.get_layout_id().is_none() && context_menu_target.get_field_index() >= draft.field_drafts.len()
            })
        {
            self.field_context_menu_target = None;
        }
        if self
            .unassigned_context_menu_target
            .as_ref()
            .is_some_and(|context_menu_target| {
                !matches!(
                    Self::parse_layout_size_text(&draft.size_text, draft.size_format),
                    Ok(layout_size_in_bytes) if context_menu_target.get_offset_in_bytes() < layout_size_in_bytes
                )
            })
        {
            self.unassigned_context_menu_target = None;
        }
        if let Ok(layout_size_in_bytes) = Self::parse_layout_size_text(&draft.size_text, draft.size_format) {
            self.prune_unassigned_split_offsets(layout_size_in_bytes);
        } else {
            self.clear_unassigned_split_offsets();
        }
        self.draft = Some(draft);
    }

    pub fn split_unassigned_span_for_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: Option<String>,
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) -> Option<SymbolLayoutUnassignedSelection> {
        if size_in_bytes < 2 {
            return None;
        }

        let split_offset_in_bytes = offset_in_bytes.checked_add(size_in_bytes / 2)?;
        let mut symbol_layout_editor_view_data = symbol_layout_editor_view_data.write("SymbolLayoutEditor split unassigned span")?;

        symbol_layout_editor_view_data
            .get_unassigned_split_offsets_mut(layout_id.as_deref())
            .insert(split_offset_in_bytes);
        symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        symbol_layout_editor_view_data.selected_field_index = None;
        symbol_layout_editor_view_data.selected_field_layout_id = None;
        symbol_layout_editor_view_data.selected_unassigned_span = Some(match layout_id {
            Some(layout_id) => {
                SymbolLayoutUnassignedSelection::new_for_layout(layout_id, offset_in_bytes, split_offset_in_bytes.saturating_sub(offset_in_bytes))
            }
            None => SymbolLayoutUnassignedSelection::new(offset_in_bytes, split_offset_in_bytes.saturating_sub(offset_in_bytes)),
        });
        symbol_layout_editor_view_data.selected_unassigned_span.clone()
    }

    pub fn remove_unassigned_split_offset_for_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: Option<String>,
        split_offset_in_bytes: u64,
    ) -> bool {
        let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor remove unassigned split offset") else {
            return false;
        };

        symbol_layout_editor_view_data
            .get_unassigned_split_offsets_mut(layout_id.as_deref())
            .remove(&split_offset_in_bytes)
    }

    pub fn move_unassigned_split_offset_for_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: Option<String>,
        old_split_offset_in_bytes: u64,
        new_split_offset_in_bytes: u64,
    ) -> bool {
        let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor move unassigned split offset") else {
            return false;
        };
        let split_offsets = symbol_layout_editor_view_data.get_unassigned_split_offsets_mut(layout_id.as_deref());

        if !split_offsets.remove(&old_split_offset_in_bytes) {
            return false;
        }

        split_offsets.insert(new_split_offset_in_bytes);
        true
    }

    pub fn insert_unassigned_split_offset_for_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: Option<String>,
        split_offset_in_bytes: u64,
    ) -> bool {
        if split_offset_in_bytes == 0 {
            return false;
        }

        let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor insert unassigned split offset") else {
            return false;
        };

        symbol_layout_editor_view_data
            .get_unassigned_split_offsets_mut(layout_id.as_deref())
            .insert(split_offset_in_bytes)
    }

    pub fn select_symbol_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        selected_layout_id: Option<String>,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor select symbol layout") {
            symbol_layout_editor_view_data.selected_layout_id = selected_layout_id;
        }
    }

    pub fn navigate_symbol_layout_selection(
        symbol_layout_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        direction: ListNavigationDirection,
    ) -> Option<String> {
        let mut symbol_layout_editor_view_data = symbol_layout_editor_view_data.write("SymbolLayoutEditor navigate symbol layout selection")?;
        let visible_layout_ids = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .filter(|struct_layout_descriptor| Self::layout_matches_filter(struct_layout_descriptor, &symbol_layout_editor_view_data.filter_text))
            .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id().to_string())
            .collect::<Vec<String>>();
        let selected_layout_index = symbol_layout_editor_view_data
            .selected_layout_id
            .as_ref()
            .and_then(|selected_layout_id| {
                visible_layout_ids
                    .iter()
                    .position(|visible_layout_id| visible_layout_id == selected_layout_id)
            });
        let next_selection_index = resolve_next_index(selected_layout_index, visible_layout_ids.len(), direction)?;
        let next_layout_id = visible_layout_ids.get(next_selection_index)?.clone();

        symbol_layout_editor_view_data.selected_layout_id = Some(next_layout_id.clone());

        Some(next_layout_id)
    }

    pub fn begin_create_symbol_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        default_data_type_ref: DataTypeRef,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor begin create symbol layout") {
            symbol_layout_editor_view_data.selected_layout_id = None;
            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::CreateSymbolLayout);
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = None;
            symbol_layout_editor_view_data.define_field_draft = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
            symbol_layout_editor_view_data.clear_unassigned_split_offsets();
            symbol_layout_editor_view_data.clear_pending_variant_drafts();
            symbol_layout_editor_view_data.baseline_project_symbol_catalog = Some(project_symbol_catalog.clone());
            let baseline_draft = Self::create_default_new_draft(project_symbol_catalog, default_data_type_ref, resolve_data_type_size_in_bytes);
            symbol_layout_editor_view_data.baseline_draft = Some(baseline_draft.clone());
            symbol_layout_editor_view_data.draft = Some(baseline_draft);
        }
    }

    pub fn begin_rename_symbol_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor begin rename symbol layout") {
            symbol_layout_editor_view_data.selected_layout_id = Some(layout_id.to_string());
            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::RenameSymbolLayout {
                layout_id: layout_id.to_string(),
            });
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = None;
            symbol_layout_editor_view_data.define_field_draft = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
            symbol_layout_editor_view_data.clear_unassigned_split_offsets();
            symbol_layout_editor_view_data.clear_pending_variant_drafts();
            symbol_layout_editor_view_data.baseline_project_symbol_catalog = Some(project_symbol_catalog.clone());
            let selected_struct_layout_descriptor = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id);
            symbol_layout_editor_view_data.baseline_draft = selected_struct_layout_descriptor.map(|struct_layout_descriptor| {
                Self::create_draft_from_descriptor_with_catalog(project_symbol_catalog, struct_layout_descriptor, resolve_data_type_size_in_bytes)
            });
            if let Some(struct_layout_descriptor) = selected_struct_layout_descriptor {
                symbol_layout_editor_view_data.unassigned_split_offsets =
                    Self::collect_unassigned_split_offsets_from_descriptor(project_symbol_catalog, struct_layout_descriptor, resolve_data_type_size_in_bytes);
            }
            symbol_layout_editor_view_data.draft = symbol_layout_editor_view_data.baseline_draft.clone();
        }
    }

    pub fn begin_open_symbol_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor begin open symbol layout") {
            symbol_layout_editor_view_data.selected_layout_id = Some(layout_id.to_string());
            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::OpenSymbolLayout {
                layout_id: layout_id.to_string(),
            });
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = None;
            symbol_layout_editor_view_data.define_field_draft = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
            symbol_layout_editor_view_data.clear_unassigned_split_offsets();
            symbol_layout_editor_view_data.clear_pending_variant_drafts();
            symbol_layout_editor_view_data.baseline_project_symbol_catalog = Some(project_symbol_catalog.clone());
            let selected_struct_layout_descriptor = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id);
            symbol_layout_editor_view_data.baseline_draft = selected_struct_layout_descriptor.map(|struct_layout_descriptor| {
                Self::create_draft_from_descriptor_with_catalog(project_symbol_catalog, struct_layout_descriptor, resolve_data_type_size_in_bytes)
            });
            if let Some(struct_layout_descriptor) = selected_struct_layout_descriptor {
                symbol_layout_editor_view_data.unassigned_split_offsets =
                    Self::collect_unassigned_split_offsets_from_descriptor(project_symbol_catalog, struct_layout_descriptor, resolve_data_type_size_in_bytes);
            }
            symbol_layout_editor_view_data.draft = symbol_layout_editor_view_data.baseline_draft.clone();
        }
    }

    pub fn request_delete_confirmation(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: String,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor request delete confirmation") {
            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::DeleteConfirmation { layout_id });
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = None;
            symbol_layout_editor_view_data.define_field_draft = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        }
    }

    pub fn request_field_unassign_confirmation(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: String,
        field_index: usize,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor request field unassign confirmation") {
            let return_state = match symbol_layout_editor_view_data.take_over_state.as_ref() {
                Some(SymbolLayoutEditorTakeOverState::CreateSymbolLayout) => SymbolLayoutDefineFieldReturnState::CreateSymbolLayout,
                Some(SymbolLayoutEditorTakeOverState::RenameSymbolLayout { layout_id }) => {
                    SymbolLayoutDefineFieldReturnState::RenameSymbolLayout { layout_id: layout_id.clone() }
                }
                Some(SymbolLayoutEditorTakeOverState::OpenSymbolLayout { layout_id })
                | Some(SymbolLayoutEditorTakeOverState::DeleteConfirmation { layout_id })
                | Some(SymbolLayoutEditorTakeOverState::DefineFieldFromUnassignedSpan { layout_id, .. })
                | Some(SymbolLayoutEditorTakeOverState::UnassignFieldConfirmation { layout_id, .. }) => {
                    SymbolLayoutDefineFieldReturnState::OpenSymbolLayout { layout_id: layout_id.clone() }
                }
                None => SymbolLayoutDefineFieldReturnState::OpenSymbolLayout { layout_id: layout_id.clone() },
            };

            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::UnassignFieldConfirmation {
                layout_id,
                field_index,
                return_state,
            });
            symbol_layout_editor_view_data.selected_field_index = Some(field_index);
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = None;
            symbol_layout_editor_view_data.define_field_draft = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        }
    }

    pub fn return_to_open_symbol_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: String,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor return to open symbol layout") {
            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::OpenSymbolLayout { layout_id });
            symbol_layout_editor_view_data.define_field_draft = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        }
    }

    pub fn return_to_define_field_source(
        symbol_layout_editor_view_data: Dependency<Self>,
        return_state: SymbolLayoutDefineFieldReturnState,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor return to define field source") {
            symbol_layout_editor_view_data.take_over_state = Some(match return_state {
                SymbolLayoutDefineFieldReturnState::CreateSymbolLayout => SymbolLayoutEditorTakeOverState::CreateSymbolLayout,
                SymbolLayoutDefineFieldReturnState::RenameSymbolLayout { layout_id } => SymbolLayoutEditorTakeOverState::RenameSymbolLayout { layout_id },
                SymbolLayoutDefineFieldReturnState::OpenSymbolLayout { layout_id } => SymbolLayoutEditorTakeOverState::OpenSymbolLayout { layout_id },
            });
            symbol_layout_editor_view_data.define_field_draft = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        }
    }

    pub fn begin_define_field_from_unassigned_span(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: String,
        offset: u64,
        size: u64,
        default_data_type_ref: DataTypeRef,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor begin define field from unassigned span") {
            let mut field_draft = SymbolLayoutFieldEditDraft::new(default_data_type_ref);
            field_draft.field_name = format!("field_{:08X}", offset);
            field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
            field_draft.static_offset_in_bytes = String::from("0");

            let return_state = match symbol_layout_editor_view_data.take_over_state.as_ref() {
                Some(SymbolLayoutEditorTakeOverState::CreateSymbolLayout) => SymbolLayoutDefineFieldReturnState::CreateSymbolLayout,
                Some(SymbolLayoutEditorTakeOverState::RenameSymbolLayout { layout_id }) => {
                    SymbolLayoutDefineFieldReturnState::RenameSymbolLayout { layout_id: layout_id.clone() }
                }
                Some(SymbolLayoutEditorTakeOverState::OpenSymbolLayout { layout_id })
                | Some(SymbolLayoutEditorTakeOverState::UnassignFieldConfirmation { layout_id, .. })
                | Some(SymbolLayoutEditorTakeOverState::DeleteConfirmation { layout_id })
                | Some(SymbolLayoutEditorTakeOverState::DefineFieldFromUnassignedSpan { layout_id, .. }) => {
                    SymbolLayoutDefineFieldReturnState::OpenSymbolLayout { layout_id: layout_id.clone() }
                }
                None => SymbolLayoutDefineFieldReturnState::OpenSymbolLayout { layout_id: layout_id.clone() },
            };

            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::DefineFieldFromUnassignedSpan {
                layout_id,
                offset,
                size,
                return_state,
            });
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = Some(SymbolLayoutUnassignedSelection::new(offset, size));
            symbol_layout_editor_view_data.define_field_draft = Some(field_draft);
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        }
    }

    pub fn replace_define_field_draft(
        symbol_layout_editor_view_data: Dependency<Self>,
        field_draft: SymbolLayoutFieldEditDraft,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor replace define field draft") {
            symbol_layout_editor_view_data.define_field_draft = Some(field_draft);
        }
    }

    pub fn show_field_context_menu(
        symbol_layout_editor_view_data: Dependency<Self>,
        field_index: usize,
        position: Pos2,
    ) {
        Self::show_field_context_menu_for_layout(symbol_layout_editor_view_data, None, field_index, position);
    }

    pub fn show_field_context_menu_for_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: Option<String>,
        field_index: usize,
        position: Pos2,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor show field context menu") {
            let field_context_menu_target = SymbolLayoutFieldContextMenuTarget::new(field_index, position);
            let field_context_menu_target = match layout_id.as_ref() {
                Some(layout_id) => field_context_menu_target.with_layout_id(layout_id.clone()),
                None => field_context_menu_target,
            };
            symbol_layout_editor_view_data.field_context_menu_target = Some(field_context_menu_target);
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
            symbol_layout_editor_view_data.selected_field_layout_id = layout_id;
            symbol_layout_editor_view_data.selected_field_index = Some(field_index);
            symbol_layout_editor_view_data.selected_unassigned_span = None;
        }
    }

    pub fn hide_field_context_menu(symbol_layout_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor hide field context menu") {
            symbol_layout_editor_view_data.field_context_menu_target = None;
        }
    }

    pub fn show_unassigned_context_menu_for_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: Option<String>,
        offset_in_bytes: u64,
        size_in_bytes: u64,
        position: Pos2,
        merge_above_span: Option<SymbolLayoutUnassignedSelection>,
        merge_below_span: Option<SymbolLayoutUnassignedSelection>,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor show unassigned context menu") {
            let context_menu_target =
                SymbolLayoutUnassignedContextMenuTarget::new(offset_in_bytes, size_in_bytes, position).with_merge_spans(merge_above_span, merge_below_span);
            let context_menu_target = match layout_id.as_ref() {
                Some(layout_id) => context_menu_target.with_layout_id(layout_id.clone()),
                None => context_menu_target,
            };

            symbol_layout_editor_view_data.unassigned_context_menu_target = Some(context_menu_target);
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = Some(match layout_id {
                Some(layout_id) => SymbolLayoutUnassignedSelection::new_for_layout(layout_id, offset_in_bytes, size_in_bytes),
                None => SymbolLayoutUnassignedSelection::new(offset_in_bytes, size_in_bytes),
            });
        }
    }

    pub fn hide_unassigned_context_menu(symbol_layout_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor hide unassigned context menu") {
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        }
    }

    pub fn select_field(
        symbol_layout_editor_view_data: Dependency<Self>,
        field_index: usize,
    ) {
        Self::select_field_for_layout(symbol_layout_editor_view_data, None, field_index);
    }

    pub fn select_field_for_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: Option<String>,
        field_index: usize,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor select field") {
            symbol_layout_editor_view_data.selected_field_layout_id = layout_id;
            symbol_layout_editor_view_data.selected_field_index = Some(field_index);
            symbol_layout_editor_view_data.selected_unassigned_span = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        }
    }

    pub fn select_unassigned_span_for_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: Option<String>,
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor select unassigned span") {
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = Some(match layout_id {
                Some(layout_id) => SymbolLayoutUnassignedSelection::new_for_layout(layout_id, offset_in_bytes, size_in_bytes),
                None => SymbolLayoutUnassignedSelection::new(offset_in_bytes, size_in_bytes),
            });
            symbol_layout_editor_view_data.field_context_menu_target = None;
        }
    }

    pub fn clear_field_selection(symbol_layout_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor clear field selection") {
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        }
    }

    pub fn cancel_take_over_state(symbol_layout_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor cancel take over state") {
            symbol_layout_editor_view_data.take_over_state = None;
            symbol_layout_editor_view_data.baseline_project_symbol_catalog = None;
            symbol_layout_editor_view_data.baseline_draft = None;
            symbol_layout_editor_view_data.draft = None;
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = None;
            symbol_layout_editor_view_data.define_field_draft = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
            symbol_layout_editor_view_data.clear_unassigned_split_offsets();
            symbol_layout_editor_view_data.clear_pending_variant_drafts();
        }
    }

    pub fn synchronize(
        symbol_layout_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor synchronize") else {
            return;
        };

        let next_selected_layout_id = symbol_layout_editor_view_data
            .selected_layout_id
            .as_ref()
            .filter(|selected_layout_id| {
                project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == selected_layout_id.as_str())
            })
            .cloned()
            .or_else(|| {
                project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .first()
                    .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id().to_string())
            });

        symbol_layout_editor_view_data.selected_layout_id = next_selected_layout_id.clone();

        let should_clear_take_over_state = match symbol_layout_editor_view_data.take_over_state.as_ref() {
            Some(SymbolLayoutEditorTakeOverState::CreateSymbolLayout) => false,
            Some(SymbolLayoutEditorTakeOverState::DefineFieldFromUnassignedSpan {
                return_state: SymbolLayoutDefineFieldReturnState::CreateSymbolLayout,
                ..
            }) => false,
            Some(
                SymbolLayoutEditorTakeOverState::RenameSymbolLayout { layout_id }
                | SymbolLayoutEditorTakeOverState::OpenSymbolLayout { layout_id }
                | SymbolLayoutEditorTakeOverState::DeleteConfirmation { layout_id }
                | SymbolLayoutEditorTakeOverState::UnassignFieldConfirmation { layout_id, .. },
            ) => !project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id),
            Some(SymbolLayoutEditorTakeOverState::DefineFieldFromUnassignedSpan { layout_id, .. }) => !project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id),
            None => false,
        };

        if should_clear_take_over_state {
            symbol_layout_editor_view_data.take_over_state = None;
            symbol_layout_editor_view_data.baseline_project_symbol_catalog = None;
            symbol_layout_editor_view_data.baseline_draft = None;
            symbol_layout_editor_view_data.draft = None;
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
            symbol_layout_editor_view_data.selected_unassigned_span = None;
            symbol_layout_editor_view_data.define_field_draft = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
            symbol_layout_editor_view_data.clear_unassigned_split_offsets();
            symbol_layout_editor_view_data.clear_pending_variant_drafts();
        }

        let stale_field_unassign_layout_id = match symbol_layout_editor_view_data.take_over_state.as_ref() {
            Some(SymbolLayoutEditorTakeOverState::UnassignFieldConfirmation { layout_id, field_index, .. })
                if symbol_layout_editor_view_data
                    .draft
                    .as_ref()
                    .is_none_or(|draft| *field_index >= draft.field_drafts.len()) =>
            {
                Some(layout_id.clone())
            }
            _ => None,
        };

        if let Some(layout_id) = stale_field_unassign_layout_id {
            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::OpenSymbolLayout { layout_id });
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        }

        if symbol_layout_editor_view_data
            .selected_field_index
            .is_some_and(|field_index| {
                symbol_layout_editor_view_data
                    .selected_field_layout_id
                    .is_none()
                    && symbol_layout_editor_view_data
                        .draft
                        .as_ref()
                        .is_none_or(|draft| field_index >= draft.field_drafts.len())
            })
        {
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.selected_field_layout_id = None;
        }

        if symbol_layout_editor_view_data
            .field_context_menu_target
            .as_ref()
            .is_some_and(|context_menu_target| {
                context_menu_target.get_layout_id().is_none()
                    && symbol_layout_editor_view_data
                        .draft
                        .as_ref()
                        .is_none_or(|draft| context_menu_target.get_field_index() >= draft.field_drafts.len())
            })
        {
            symbol_layout_editor_view_data.field_context_menu_target = None;
        }
        if symbol_layout_editor_view_data
            .unassigned_context_menu_target
            .as_ref()
            .is_some_and(|context_menu_target| {
                symbol_layout_editor_view_data
                    .draft
                    .as_ref()
                    .is_none_or(|draft| {
                        !matches!(
                            Self::parse_layout_size_text(&draft.size_text, draft.size_format),
                            Ok(layout_size_in_bytes) if context_menu_target.get_offset_in_bytes() < layout_size_in_bytes
                        )
                    })
            })
        {
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
        }
    }

    pub fn unassign_field_from_draft(
        draft: &mut SymbolLayoutEditDraft,
        field_index: usize,
    ) -> bool {
        if field_index >= draft.field_drafts.len() {
            return false;
        }

        draft.field_drafts.remove(field_index);

        true
    }

    pub fn layout_matches_filter(
        struct_layout_descriptor: &StructLayoutDescriptor,
        filter_text: &str,
    ) -> bool {
        let trimmed_filter_text = filter_text.trim();
        if trimmed_filter_text.is_empty() {
            return true;
        }

        let normalized_filter_text = trimmed_filter_text.to_ascii_lowercase();
        if struct_layout_descriptor
            .get_struct_layout_id()
            .to_ascii_lowercase()
            .contains(&normalized_filter_text)
        {
            return true;
        }

        struct_layout_descriptor
            .get_struct_layout_definition()
            .get_fields()
            .iter()
            .any(|symbolic_field_definition| {
                symbolic_field_definition
                    .to_string()
                    .to_ascii_lowercase()
                    .contains(&normalized_filter_text)
            })
    }

    pub fn count_symbol_claim_usages(
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_id: &str,
    ) -> usize {
        let symbol_claim_usage_count = project_symbol_catalog
            .get_symbol_claims()
            .iter()
            .filter(|symbol_claim| symbol_claim.get_struct_layout_id() == struct_layout_id)
            .count();
        let module_field_usage_count = project_symbol_catalog
            .get_symbol_modules()
            .iter()
            .flat_map(|symbol_module| symbol_module.get_fields())
            .filter(|module_field| module_field.get_struct_layout_id() == struct_layout_id)
            .count();
        let struct_field_usage_count = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .flat_map(|struct_layout_descriptor| {
                struct_layout_descriptor
                    .get_struct_layout_definition()
                    .get_fields()
            })
            .filter(|symbolic_field_definition| symbolic_field_definition.get_data_type_ref().get_data_type_id() == struct_layout_id)
            .count();

        symbol_claim_usage_count
            .saturating_add(module_field_usage_count)
            .saturating_add(struct_field_usage_count)
    }

    pub fn resolve_field_element_type(
        project_symbol_catalog: &ProjectSymbolCatalog,
        field_draft: &SymbolLayoutFieldEditDraft,
    ) -> SymbolLayoutFieldElementType {
        let data_type_id = field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id();

        if project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == data_type_id)
        {
            SymbolLayoutFieldElementType::SymbolLayout
        } else {
            SymbolLayoutFieldElementType::BuiltInDataType
        }
    }

    pub fn first_symbol_layout_id(project_symbol_catalog: &ProjectSymbolCatalog) -> Option<String> {
        project_symbol_catalog
            .get_struct_layout_descriptors()
            .first()
            .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id().to_string())
    }

    pub fn create_draft_from_descriptor(
        struct_layout_descriptor: &StructLayoutDescriptor,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> SymbolLayoutEditDraft {
        Self::create_draft_from_descriptor_with_catalog(&ProjectSymbolCatalog::default(), struct_layout_descriptor, resolve_data_type_size_in_bytes)
    }

    pub fn create_draft_from_descriptor_with_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_descriptor: &StructLayoutDescriptor,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> SymbolLayoutEditDraft {
        let size_in_bytes = Self::resolve_symbolic_struct_size_in_bytes(
            project_symbol_catalog,
            struct_layout_descriptor.get_struct_layout_definition(),
            &mut HashSet::new(),
            resolve_data_type_size_in_bytes,
        );

        SymbolLayoutEditDraft {
            original_layout_id: Some(struct_layout_descriptor.get_struct_layout_id().to_string()),
            layout_id: struct_layout_descriptor.get_struct_layout_id().to_string(),
            layout_kind: struct_layout_descriptor
                .get_struct_layout_definition()
                .get_layout_kind(),
            size_text: size_in_bytes.to_string(),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: struct_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .iter()
                .scan(0_u64, |next_sequential_offset, symbolic_field_definition| {
                    if symbolic_field_definition.is_unassigned() {
                        *next_sequential_offset = next_sequential_offset.saturating_add(
                            symbolic_field_definition
                                .get_unassigned_size_in_bytes()
                                .unwrap_or(0),
                        );
                        return Some(None);
                    }

                    let field_offset = match symbolic_field_definition.get_offset_resolution() {
                        SymbolicFieldOffsetResolution::Static(offset_in_bytes) => *offset_in_bytes,
                        SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_)
                            if struct_layout_descriptor
                                .get_struct_layout_definition()
                                .get_layout_kind()
                                .is_union() =>
                        {
                            0
                        }
                        SymbolicFieldOffsetResolution::Sequential | SymbolicFieldOffsetResolution::Resolver(_) => *next_sequential_offset,
                    };
                    let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(
                        project_symbol_catalog,
                        symbolic_field_definition,
                        &mut HashSet::new(),
                        resolve_data_type_size_in_bytes,
                    );
                    *next_sequential_offset = (*next_sequential_offset).max(field_offset.saturating_add(field_size_in_bytes));

                    let mut field_draft = SymbolLayoutFieldEditDraft::from_symbolic_field_definition(symbolic_field_definition);
                    if field_draft.offset_mode == SymbolLayoutFieldOffsetMode::Sequential && field_offset > 0 {
                        field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
                        field_draft.static_offset_in_bytes = format!("0x{:X}", field_offset);
                    }

                    Some(Some(field_draft))
                })
                .flatten()
                .collect(),
        }
    }

    pub fn create_default_new_draft(
        project_symbol_catalog: &ProjectSymbolCatalog,
        default_data_type_ref: DataTypeRef,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> SymbolLayoutEditDraft {
        let mut suffix_index = 1_u64;
        let mut proposed_layout_id = String::from("new.struct");
        while project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == proposed_layout_id)
        {
            suffix_index = suffix_index.saturating_add(1);
            proposed_layout_id = format!("new.struct{}", suffix_index);
        }
        let default_size_in_bytes = resolve_data_type_size_in_bytes(&default_data_type_ref).unwrap_or(1);

        let mut field_draft = SymbolLayoutFieldEditDraft::new(default_data_type_ref);
        field_draft.field_name = String::from("field_1");

        SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: proposed_layout_id,
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: default_size_in_bytes.to_string(),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![field_draft],
        }
    }

    pub fn build_symbol_layout_descriptor(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> Result<StructLayoutDescriptor, String> {
        SymbolLayoutDescriptorBuilder::build_symbol_layout_descriptor(project_symbol_catalog, draft, resolve_data_type_size_in_bytes)
    }

    pub fn build_symbol_layout_descriptor_with_unassigned_split_offsets(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
        unassigned_split_offsets: &BTreeSet<u64>,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> Result<StructLayoutDescriptor, String> {
        SymbolLayoutDescriptorBuilder::build_symbol_layout_descriptor_with_unassigned_split_offsets(
            project_symbol_catalog,
            draft,
            unassigned_split_offsets,
            resolve_data_type_size_in_bytes,
        )
    }

    pub fn parse_layout_size_text(
        size_text: &str,
        size_format: AnonymousValueStringFormat,
    ) -> Result<u64, String> {
        SymbolLayoutDescriptorBuilder::parse_layout_size_text(size_text, size_format)
    }

    pub fn resolve_symbolic_struct_size_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_struct_definition: &SymbolicStructDefinition,
        visited_struct_layout_ids: &mut HashSet<String>,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> u64 {
        SymbolLayoutDescriptorBuilder::resolve_symbolic_struct_size_in_bytes(
            project_symbol_catalog,
            symbolic_struct_definition,
            visited_struct_layout_ids,
            resolve_data_type_size_in_bytes,
        )
    }

    pub fn resolve_symbolic_struct_field_span_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_struct_definition: &SymbolicStructDefinition,
        visited_struct_layout_ids: &mut HashSet<String>,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> u64 {
        SymbolLayoutDescriptorBuilder::resolve_symbolic_struct_field_span_in_bytes(
            project_symbol_catalog,
            symbolic_struct_definition,
            visited_struct_layout_ids,
            resolve_data_type_size_in_bytes,
        )
    }

    pub fn resolve_symbolic_field_size_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_field_definition: &SymbolicFieldDefinition,
        visited_struct_layout_ids: &mut HashSet<String>,
        resolve_data_type_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> u64 {
        SymbolLayoutDescriptorBuilder::resolve_symbolic_field_size_in_bytes(
            project_symbol_catalog,
            symbolic_field_definition,
            visited_struct_layout_ids,
            resolve_data_type_size_in_bytes,
        )
    }
}

impl SymbolLayoutFieldEditDraft {
    pub fn new(default_data_type_ref: DataTypeRef) -> Self {
        Self {
            field_name: String::new(),
            data_type_selection: DataTypeSelection::new(default_data_type_ref),
            container_edit: SymbolLayoutFieldContainerEdit::default(),
            active_when_resolver_id: String::new(),
            display_format: None,
            offset_mode: SymbolLayoutFieldOffsetMode::Sequential,
            static_offset_in_bytes: String::new(),
            offset_resolver_id: String::new(),
        }
    }

    pub fn from_symbolic_field_definition(symbolic_field_definition: &SymbolicFieldDefinition) -> Self {
        let (offset_mode, static_offset_in_bytes, offset_resolver_id) = match symbolic_field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Sequential => (SymbolLayoutFieldOffsetMode::Sequential, String::new(), String::new()),
            SymbolicFieldOffsetResolution::Static(offset_in_bytes) => (SymbolLayoutFieldOffsetMode::Static, offset_in_bytes.to_string(), String::new()),
            SymbolicFieldOffsetResolution::Resolver(resolver_id) => (SymbolLayoutFieldOffsetMode::Resolver, String::new(), resolver_id.clone()),
        };

        Self {
            field_name: symbolic_field_definition.get_field_name().to_string(),
            data_type_selection: DataTypeSelection::new(symbolic_field_definition.get_data_type_ref().clone()),
            container_edit: SymbolLayoutFieldContainerEdit::from_symbolic_field_definition(symbolic_field_definition),
            active_when_resolver_id: symbolic_field_definition
                .get_active_when_resolver()
                .map(|resolver_ref| resolver_ref.get_resolver_id().to_string())
                .unwrap_or_default(),
            display_format: symbolic_field_definition.get_display_format(),
            offset_mode,
            static_offset_in_bytes,
            offset_resolver_id,
        }
    }

    pub fn to_offset_resolution(&self) -> Result<SymbolicFieldOffsetResolution, String> {
        match self.offset_mode {
            SymbolLayoutFieldOffsetMode::Sequential => Ok(SymbolicFieldOffsetResolution::Sequential),
            SymbolLayoutFieldOffsetMode::Static => {
                let trimmed_offset = self.static_offset_in_bytes.trim();
                if trimmed_offset.is_empty() {
                    return Err(String::from("Byte offset is required."));
                }

                let offset_in_bytes = Self::parse_static_offset_text(trimmed_offset).ok_or_else(|| format!("Invalid byte offset: {}.", trimmed_offset))?;

                Ok(SymbolicFieldOffsetResolution::new_static(offset_in_bytes))
            }
            SymbolLayoutFieldOffsetMode::Resolver => {
                let trimmed_resolver_id = self.offset_resolver_id.trim();
                if trimmed_resolver_id.is_empty() {
                    return Err(String::from("Offset resolver is required."));
                }

                Ok(SymbolicFieldOffsetResolution::new_resolver(trimmed_resolver_id.to_string()))
            }
        }
    }

    pub fn to_active_when_resolver(&self) -> Option<SymbolicResolverRef> {
        SymbolicResolverRef::new(self.active_when_resolver_id.clone())
    }

    pub fn parse_static_offset_text(offset_text: &str) -> Option<u64> {
        let trimmed_offset = offset_text.trim();
        let trimmed_offset = trimmed_offset
            .strip_prefix('+')
            .map(str::trim)
            .unwrap_or(trimmed_offset);

        if let Some(binary_offset) = trimmed_offset
            .strip_prefix("0b")
            .or_else(|| trimmed_offset.strip_prefix("0B"))
        {
            u64::from_str_radix(binary_offset, 2).ok()
        } else if let Some(hex_offset) = trimmed_offset
            .strip_prefix("0x")
            .or_else(|| trimmed_offset.strip_prefix("0X"))
        {
            u64::from_str_radix(hex_offset, 16).ok()
        } else {
            trimmed_offset.parse::<u64>().ok()
        }
    }
}

impl Default for SymbolLayoutEditDraft {
    fn default() -> Self {
        Self {
            original_layout_id: None,
            layout_id: String::new(),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("4"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![SymbolLayoutFieldEditDraft::new(DataTypeRef::new(
                DataTypeI32::DATA_TYPE_ID,
            ))],
        }
    }
}
