use crate::ui::list_navigation::{ListNavigationDirection, resolve_next_index};
use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::SymbolLayoutFieldContainerEdit;
use epaint::Pos2;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::{
    data_types::{
        built_in_types::{i32::data_type_i32::DataTypeI32, u8::data_type_u8::DataTypeU8},
        data_type_ref::DataTypeRef,
    },
    data_values::anonymous_value_string_format::AnonymousValueStringFormat,
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::{
        symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
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
    pub is_hidden: bool,
    pub active_when_resolver_id: String,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolLayoutUnassignedSelection {
    layout_id: Option<String>,
    offset_in_bytes: u64,
    size_in_bytes: u64,
}

impl SymbolLayoutUnassignedSelection {
    pub fn new(
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) -> Self {
        Self {
            layout_id: None,
            offset_in_bytes,
            size_in_bytes,
        }
    }

    pub fn new_for_layout(
        layout_id: String,
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) -> Self {
        Self {
            layout_id: Some(layout_id),
            offset_in_bytes,
            size_in_bytes,
        }
    }

    pub fn get_layout_id(&self) -> Option<&str> {
        self.layout_id.as_deref()
    }

    pub fn matches(
        &self,
        layout_id: Option<&str>,
        offset_in_bytes: u64,
        size_in_bytes: u64,
    ) -> bool {
        self.get_layout_id() == layout_id && self.offset_in_bytes == offset_in_bytes && self.size_in_bytes == size_in_bytes
    }

    pub fn get_offset_in_bytes(&self) -> u64 {
        self.offset_in_bytes
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.size_in_bytes
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
    DeleteFieldConfirmation {
        layout_id: String,
        field_index: usize,
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
    selected_field_layout_id: Option<String>,
    selected_field_index: Option<usize>,
    selected_unassigned_span: Option<SymbolLayoutUnassignedSelection>,
    define_field_draft: Option<SymbolLayoutFieldEditDraft>,
    field_context_menu_target: Option<SymbolLayoutFieldContextMenuTarget>,
    unassigned_context_menu_target: Option<SymbolLayoutUnassignedContextMenuTarget>,
    unassigned_split_offsets: BTreeSet<u64>,
    unassigned_split_offsets_by_layout: BTreeMap<String, BTreeSet<u64>>,
    has_take_over_catalog_side_effects: bool,
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
            selected_field_layout_id: None,
            selected_field_index: None,
            selected_unassigned_span: None,
            define_field_draft: None,
            field_context_menu_target: None,
            unassigned_context_menu_target: None,
            unassigned_split_offsets: BTreeSet::new(),
            unassigned_split_offsets_by_layout: BTreeMap::new(),
            has_take_over_catalog_side_effects: false,
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

    pub fn has_take_over_catalog_side_effects(&self) -> bool {
        self.has_take_over_catalog_side_effects
    }

    pub fn mark_take_over_catalog_side_effect(symbol_layout_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor mark take over catalog side effect") {
            if symbol_layout_editor_view_data.take_over_state.is_some() {
                symbol_layout_editor_view_data.has_take_over_catalog_side_effects = true;
            }
        }
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

    fn collect_unassigned_split_offsets_from_descriptor(
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_descriptor: &StructLayoutDescriptor,
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
            let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(project_symbol_catalog, symbolic_field_definition, &mut HashSet::new());
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
            symbol_layout_editor_view_data.baseline_project_symbol_catalog = Some(project_symbol_catalog.clone());
            symbol_layout_editor_view_data.has_take_over_catalog_side_effects = false;
            let baseline_draft = Self::create_default_new_draft(project_symbol_catalog, default_data_type_ref);
            symbol_layout_editor_view_data.baseline_draft = Some(baseline_draft.clone());
            symbol_layout_editor_view_data.draft = Some(baseline_draft);
        }
    }

    pub fn begin_rename_symbol_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
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
            symbol_layout_editor_view_data.baseline_project_symbol_catalog = Some(project_symbol_catalog.clone());
            symbol_layout_editor_view_data.has_take_over_catalog_side_effects = false;
            let selected_struct_layout_descriptor = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id);
            symbol_layout_editor_view_data.baseline_draft = selected_struct_layout_descriptor
                .map(|struct_layout_descriptor| Self::create_draft_from_descriptor_with_catalog(project_symbol_catalog, struct_layout_descriptor));
            if let Some(struct_layout_descriptor) = selected_struct_layout_descriptor {
                symbol_layout_editor_view_data.unassigned_split_offsets =
                    Self::collect_unassigned_split_offsets_from_descriptor(project_symbol_catalog, struct_layout_descriptor);
            }
            symbol_layout_editor_view_data.draft = symbol_layout_editor_view_data.baseline_draft.clone();
        }
    }

    pub fn begin_open_symbol_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
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
            symbol_layout_editor_view_data.baseline_project_symbol_catalog = Some(project_symbol_catalog.clone());
            symbol_layout_editor_view_data.has_take_over_catalog_side_effects = false;
            let selected_struct_layout_descriptor = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id);
            symbol_layout_editor_view_data.baseline_draft = selected_struct_layout_descriptor
                .map(|struct_layout_descriptor| Self::create_draft_from_descriptor_with_catalog(project_symbol_catalog, struct_layout_descriptor));
            if let Some(struct_layout_descriptor) = selected_struct_layout_descriptor {
                symbol_layout_editor_view_data.unassigned_split_offsets =
                    Self::collect_unassigned_split_offsets_from_descriptor(project_symbol_catalog, struct_layout_descriptor);
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

    pub fn request_field_delete_confirmation(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: String,
        field_index: usize,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor request field delete confirmation") {
            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::DeleteFieldConfirmation { layout_id, field_index });
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
                | Some(SymbolLayoutEditorTakeOverState::DeleteFieldConfirmation { layout_id, .. })
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
            symbol_layout_editor_view_data.has_take_over_catalog_side_effects = false;
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
                | SymbolLayoutEditorTakeOverState::DeleteFieldConfirmation { layout_id, .. },
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
        }

        let stale_field_delete_layout_id = match symbol_layout_editor_view_data.take_over_state.as_ref() {
            Some(SymbolLayoutEditorTakeOverState::DeleteFieldConfirmation { layout_id, field_index })
                if symbol_layout_editor_view_data
                    .draft
                    .as_ref()
                    .is_none_or(|draft| *field_index >= draft.field_drafts.len()) =>
            {
                Some(layout_id.clone())
            }
            _ => None,
        };

        if let Some(layout_id) = stale_field_delete_layout_id {
            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::OpenSymbolLayout { layout_id });
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.unassigned_context_menu_target = None;
            symbol_layout_editor_view_data.has_take_over_catalog_side_effects = false;
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

    pub fn remove_field_from_draft(
        draft: &mut SymbolLayoutEditDraft,
        field_index: usize,
        default_data_type_ref: DataTypeRef,
    ) -> Option<usize> {
        if field_index >= draft.field_drafts.len() {
            return None;
        }

        draft.field_drafts.remove(field_index);
        if draft.field_drafts.is_empty() {
            draft
                .field_drafts
                .push(SymbolLayoutFieldEditDraft::new(default_data_type_ref));
        }

        Some(field_index.min(draft.field_drafts.len().saturating_sub(1)))
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

    pub fn create_draft_from_descriptor(struct_layout_descriptor: &StructLayoutDescriptor) -> SymbolLayoutEditDraft {
        Self::create_draft_from_descriptor_with_catalog(&ProjectSymbolCatalog::default(), struct_layout_descriptor)
    }

    pub fn create_draft_from_descriptor_with_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_descriptor: &StructLayoutDescriptor,
    ) -> SymbolLayoutEditDraft {
        let size_in_bytes = Self::resolve_symbolic_struct_size_in_bytes(
            project_symbol_catalog,
            struct_layout_descriptor.get_struct_layout_definition(),
            &mut HashSet::new(),
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
                    let field_size_in_bytes =
                        Self::resolve_symbolic_field_size_in_bytes(project_symbol_catalog, symbolic_field_definition, &mut HashSet::new());
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
        let default_size_in_bytes = Self::resolve_primitive_data_type_size_in_bytes(default_data_type_ref.get_data_type_id()).unwrap_or(1);

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
    ) -> Result<StructLayoutDescriptor, String> {
        Self::build_symbol_layout_descriptor_with_unassigned_split_offsets(project_symbol_catalog, draft, &BTreeSet::new())
    }

    pub fn build_symbol_layout_descriptor_with_unassigned_split_offsets(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
        unassigned_split_offsets: &BTreeSet<u64>,
    ) -> Result<StructLayoutDescriptor, String> {
        let trimmed_layout_id = draft.layout_id.trim();
        if trimmed_layout_id.is_empty() {
            return Err(String::from("Symbol layout id is required."));
        }

        let conflicts_with_existing_layout = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .any(|struct_layout_descriptor| {
                struct_layout_descriptor.get_struct_layout_id() == trimmed_layout_id && draft.original_layout_id.as_deref() != Some(trimmed_layout_id)
            });
        if conflicts_with_existing_layout {
            return Err(String::from("Symbol layout id must be unique."));
        }
        let declared_size_in_bytes = Self::parse_layout_size_text(&draft.size_text, draft.size_format)?;

        let mut symbolic_field_definitions = Vec::with_capacity(draft.field_drafts.len());
        let mut field_names = HashSet::new();
        let mut next_sequential_offset = 0_u64;
        for field_draft in &draft.field_drafts {
            let trimmed_data_type_id = field_draft
                .data_type_selection
                .visible_data_type()
                .get_data_type_id()
                .trim()
                .to_string();
            if trimmed_data_type_id.is_empty() {
                return Err(String::from("Each field needs a data type."));
            }

            let container_type = field_draft.container_edit.to_container_type()?;
            let count_resolution = field_draft.container_edit.to_count_resolution()?;
            let display_count_resolution = field_draft.container_edit.to_display_count_resolution()?;
            let offset_resolution = field_draft.to_offset_resolution()?;
            let trimmed_field_name = field_draft.field_name.trim().to_string();
            if trimmed_field_name.is_empty() {
                return Err(String::from("Each field needs a name."));
            }
            if !field_names.insert(trimmed_field_name.clone()) {
                return Err(format!("Field name `{}` is already used in this layout.", trimmed_field_name));
            }

            let data_type_ref = DataTypeRef::new(&trimmed_data_type_id);
            let symbolic_field_definition = SymbolicFieldDefinition::new_named_with_resolutions_and_display_count(
                trimmed_field_name,
                data_type_ref,
                container_type,
                count_resolution,
                display_count_resolution,
                offset_resolution,
            )
            .with_active_when_resolver(field_draft.to_active_when_resolver())
            .with_hidden(field_draft.is_hidden);

            let (field_offset, symbolic_field_definition) = match symbolic_field_definition.get_offset_resolution() {
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) if !draft.layout_kind.is_union() && *offset_in_bytes >= next_sequential_offset => {
                    if *offset_in_bytes > next_sequential_offset {
                        Self::push_unassigned_range(
                            &mut symbolic_field_definitions,
                            next_sequential_offset,
                            *offset_in_bytes,
                            unassigned_split_offsets,
                            true,
                        );
                    }

                    (
                        *offset_in_bytes,
                        symbolic_field_definition.with_offset_resolution(SymbolicFieldOffsetResolution::Sequential),
                    )
                }
                SymbolicFieldOffsetResolution::Static(offset_in_bytes) => (*offset_in_bytes, symbolic_field_definition),
                SymbolicFieldOffsetResolution::Sequential if draft.layout_kind.is_union() => (0, symbolic_field_definition),
                SymbolicFieldOffsetResolution::Sequential => (next_sequential_offset, symbolic_field_definition),
                SymbolicFieldOffsetResolution::Resolver(_) if draft.layout_kind.is_union() => (0, symbolic_field_definition),
                SymbolicFieldOffsetResolution::Resolver(_) => (next_sequential_offset, symbolic_field_definition),
            };
            let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(project_symbol_catalog, &symbolic_field_definition, &mut HashSet::new());

            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
            symbolic_field_definitions.push(symbolic_field_definition);
        }
        if !draft.layout_kind.is_union() && declared_size_in_bytes > next_sequential_offset {
            Self::push_unassigned_range(
                &mut symbolic_field_definitions,
                next_sequential_offset,
                declared_size_in_bytes,
                unassigned_split_offsets,
                false,
            );
        }

        let symbolic_struct_definition =
            SymbolicStructDefinition::new_with_layout_kind(trimmed_layout_id.to_string(), draft.layout_kind, symbolic_field_definitions)
                .with_declared_size_in_bytes(Some(declared_size_in_bytes));
        let minimum_size_in_bytes = Self::resolve_symbolic_struct_field_span_in_bytes(project_symbol_catalog, &symbolic_struct_definition, &mut HashSet::new());
        if declared_size_in_bytes < minimum_size_in_bytes {
            return Err(format!(
                "Layout size {} byte(s) would truncate fields ending at byte {}.",
                declared_size_in_bytes, minimum_size_in_bytes
            ));
        }

        let struct_layout_descriptor = StructLayoutDescriptor::new(trimmed_layout_id.to_string(), symbolic_struct_definition);

        project_symbol_catalog.validate_local_resolver_dependencies_for_struct_layout(&struct_layout_descriptor)?;

        Ok(struct_layout_descriptor)
    }

    fn push_unassigned_range(
        symbolic_field_definitions: &mut Vec<SymbolicFieldDefinition>,
        start_offset_in_bytes: u64,
        end_offset_in_bytes: u64,
        unassigned_split_offsets: &BTreeSet<u64>,
        include_unsplit_range: bool,
    ) {
        if end_offset_in_bytes <= start_offset_in_bytes {
            return;
        }

        let contained_split_offsets = unassigned_split_offsets
            .range((start_offset_in_bytes.saturating_add(1))..end_offset_in_bytes)
            .copied()
            .collect::<Vec<_>>();
        if contained_split_offsets.is_empty() && !include_unsplit_range {
            return;
        }

        let mut previous_offset_in_bytes = start_offset_in_bytes;
        for split_offset_in_bytes in contained_split_offsets {
            if split_offset_in_bytes > previous_offset_in_bytes {
                symbolic_field_definitions.push(SymbolicFieldDefinition::new_unassigned(
                    split_offset_in_bytes.saturating_sub(previous_offset_in_bytes),
                ));
            }
            previous_offset_in_bytes = split_offset_in_bytes;
        }

        if end_offset_in_bytes > previous_offset_in_bytes {
            symbolic_field_definitions.push(SymbolicFieldDefinition::new_unassigned(
                end_offset_in_bytes.saturating_sub(previous_offset_in_bytes),
            ));
        }
    }

    pub fn parse_layout_size_text(
        size_text: &str,
        size_format: AnonymousValueStringFormat,
    ) -> Result<u64, String> {
        let trimmed_size_text = size_text.trim();
        if trimmed_size_text.is_empty() {
            return Err(String::from("Layout size is required."));
        }

        let normalized_size_text = trimmed_size_text
            .strip_prefix('+')
            .map(str::trim)
            .unwrap_or(trimmed_size_text);
        let parsed_size = if let Some(binary_size_text) = normalized_size_text
            .strip_prefix("0b")
            .or_else(|| normalized_size_text.strip_prefix("0B"))
        {
            u64::from_str_radix(binary_size_text, 2)
        } else if let Some(hex_size_text) = normalized_size_text
            .strip_prefix("0x")
            .or_else(|| normalized_size_text.strip_prefix("0X"))
        {
            u64::from_str_radix(hex_size_text, 16)
        } else {
            match size_format {
                AnonymousValueStringFormat::Binary => u64::from_str_radix(normalized_size_text, 2),
                AnonymousValueStringFormat::Decimal => normalized_size_text.parse::<u64>(),
                AnonymousValueStringFormat::Hexadecimal => u64::from_str_radix(normalized_size_text, 16),
                _ => {
                    return Err(format!("Invalid layout size: {}.", trimmed_size_text));
                }
            }
        };

        parsed_size.map_err(|_| format!("Invalid layout size: {}.", trimmed_size_text))
    }

    pub fn resolve_symbolic_struct_size_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_struct_definition: &SymbolicStructDefinition,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> u64 {
        Self::resolve_symbolic_struct_field_span_in_bytes(project_symbol_catalog, symbolic_struct_definition, visited_struct_layout_ids).max(
            symbolic_struct_definition
                .get_declared_size_in_bytes()
                .unwrap_or(0),
        )
    }

    pub fn resolve_symbolic_struct_field_span_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_struct_definition: &SymbolicStructDefinition,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> u64 {
        let mut next_sequential_offset = 0_u64;

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
            let field_size_in_bytes = Self::resolve_symbolic_field_size_in_bytes(project_symbol_catalog, symbolic_field_definition, visited_struct_layout_ids);

            next_sequential_offset = next_sequential_offset.max(field_offset.saturating_add(field_size_in_bytes));
        }

        next_sequential_offset
    }

    pub fn resolve_symbolic_field_size_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_field_definition: &SymbolicFieldDefinition,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> u64 {
        if let Some(pointer_size) = symbolic_field_definition
            .get_container_type()
            .get_pointer_size()
        {
            return pointer_size.get_size_in_bytes();
        }

        let data_type_id = symbolic_field_definition.get_data_type_ref().get_data_type_id();
        let unit_size_in_bytes = Self::resolve_data_type_size_in_bytes(project_symbol_catalog, data_type_id, visited_struct_layout_ids);

        symbolic_field_definition
            .get_container_type()
            .get_total_size_in_bytes(unit_size_in_bytes)
    }

    fn resolve_data_type_size_in_bytes(
        project_symbol_catalog: &ProjectSymbolCatalog,
        data_type_id: &str,
        visited_struct_layout_ids: &mut HashSet<String>,
    ) -> u64 {
        if !visited_struct_layout_ids.insert(data_type_id.to_string()) {
            return 0;
        }

        let size_in_bytes = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == data_type_id)
            .map(|struct_layout_descriptor| {
                Self::resolve_symbolic_struct_size_in_bytes(
                    project_symbol_catalog,
                    struct_layout_descriptor.get_struct_layout_definition(),
                    visited_struct_layout_ids,
                )
            })
            .or_else(|| Self::resolve_primitive_data_type_size_in_bytes(data_type_id))
            .unwrap_or(1);

        visited_struct_layout_ids.remove(data_type_id);

        size_in_bytes
    }

    fn resolve_primitive_data_type_size_in_bytes(data_type_id: &str) -> Option<u64> {
        match data_type_id {
            "bool" | "i8" | "u8" => Some(1),
            "i16" | "u16" | "i16be" | "u16be" => Some(2),
            "i24" | "u24" | "i24be" | "u24be" => Some(3),
            "f32" | "i32" | "u32" | "f32be" | "i32be" | "u32be" => Some(4),
            "f64" | "i64" | "u64" | "f64be" | "i64be" | "u64be" => Some(8),
            "i128" | "u128" | "i128be" | "u128be" => Some(16),
            _ => None,
        }
    }

    fn retarget_catalog_struct_layout_references(
        project_symbol_catalog: &mut ProjectSymbolCatalog,
        source_struct_layout_id: &str,
        replacement_data_type_ref: &DataTypeRef,
    ) {
        for symbol_claim in project_symbol_catalog.get_symbol_claims_mut() {
            if symbol_claim.get_struct_layout_id() == source_struct_layout_id {
                symbol_claim.set_struct_layout_id(replacement_data_type_ref.get_data_type_id().to_string());
            }
        }

        for symbol_module in project_symbol_catalog.get_symbol_modules_mut() {
            for module_field in symbol_module.get_fields_mut() {
                if module_field.get_struct_layout_id() == source_struct_layout_id {
                    module_field.set_struct_layout_id(replacement_data_type_ref.get_data_type_id().to_string());
                }
            }
        }

        let retargeted_struct_layout_descriptors = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .map(|struct_layout_descriptor| {
                let retargeted_fields = struct_layout_descriptor
                    .get_struct_layout_definition()
                    .get_fields()
                    .iter()
                    .map(|symbolic_field_definition| {
                        Self::retarget_symbolic_field_definition_type(symbolic_field_definition, source_struct_layout_id, replacement_data_type_ref)
                    })
                    .collect();

                StructLayoutDescriptor::new(
                    struct_layout_descriptor.get_struct_layout_id().to_string(),
                    SymbolicStructDefinition::new_with_layout_kind(
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_symbol_namespace()
                            .to_string(),
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_layout_kind(),
                        retargeted_fields,
                    )
                    .with_declared_size_in_bytes(
                        struct_layout_descriptor
                            .get_struct_layout_definition()
                            .get_declared_size_in_bytes(),
                    ),
                )
            })
            .collect();

        project_symbol_catalog.set_struct_layout_descriptors(retargeted_struct_layout_descriptors);
    }

    fn retarget_symbolic_field_definition_type(
        symbolic_field_definition: &SymbolicFieldDefinition,
        source_struct_layout_id: &str,
        replacement_data_type_ref: &DataTypeRef,
    ) -> SymbolicFieldDefinition {
        if symbolic_field_definition.get_data_type_ref().get_data_type_id() != source_struct_layout_id {
            return symbolic_field_definition.clone();
        }

        SymbolicFieldDefinition::new_named_with_resolutions_and_display_count(
            symbolic_field_definition.get_field_name().to_string(),
            replacement_data_type_ref.clone(),
            symbolic_field_definition.get_container_type(),
            symbolic_field_definition.get_count_resolution().clone(),
            symbolic_field_definition.get_display_count_resolution().clone(),
            symbolic_field_definition.get_offset_resolution().clone(),
        )
        .with_active_when_resolver(symbolic_field_definition.get_active_when_resolver().cloned())
        .with_hidden(symbolic_field_definition.is_hidden())
    }

    pub fn apply_draft_to_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
    ) -> Result<ProjectSymbolCatalog, String> {
        Self::apply_draft_to_catalog_with_unassigned_split_offsets(project_symbol_catalog, draft, &BTreeSet::new())
    }

    pub fn apply_draft_to_catalog_with_unassigned_split_offsets(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
        unassigned_split_offsets: &BTreeSet<u64>,
    ) -> Result<ProjectSymbolCatalog, String> {
        let resolved_struct_layout_descriptor =
            Self::build_symbol_layout_descriptor_with_unassigned_split_offsets(project_symbol_catalog, draft, unassigned_split_offsets)?;
        let mut updated_project_symbol_catalog = project_symbol_catalog.clone();
        let mut updated_struct_layout_descriptors = updated_project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .filter(|struct_layout_descriptor| draft.original_layout_id.as_deref() != Some(struct_layout_descriptor.get_struct_layout_id()))
            .cloned()
            .collect::<Vec<_>>();

        updated_struct_layout_descriptors.push(resolved_struct_layout_descriptor.clone());
        updated_struct_layout_descriptors.sort_by(|left_layout, right_layout| {
            left_layout
                .get_struct_layout_id()
                .to_ascii_lowercase()
                .cmp(&right_layout.get_struct_layout_id().to_ascii_lowercase())
        });
        updated_project_symbol_catalog.set_struct_layout_descriptors(updated_struct_layout_descriptors);

        if let Some(original_layout_id) = draft.original_layout_id.as_deref() {
            if original_layout_id != resolved_struct_layout_descriptor.get_struct_layout_id() {
                Self::retarget_catalog_struct_layout_references(
                    &mut updated_project_symbol_catalog,
                    original_layout_id,
                    &DataTypeRef::new(resolved_struct_layout_descriptor.get_struct_layout_id()),
                );
            }
        }

        Ok(updated_project_symbol_catalog)
    }

    pub fn remove_symbol_layout_from_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_id: &str,
    ) -> Result<ProjectSymbolCatalog, String> {
        let mut updated_project_symbol_catalog = project_symbol_catalog.clone();
        updated_project_symbol_catalog.set_struct_layout_descriptors(
            updated_project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .filter(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() != struct_layout_id)
                .cloned()
                .collect(),
        );
        Self::retarget_catalog_struct_layout_references(
            &mut updated_project_symbol_catalog,
            struct_layout_id,
            &DataTypeRef::new(DataTypeU8::DATA_TYPE_ID),
        );

        Ok(updated_project_symbol_catalog)
    }
}

impl SymbolLayoutFieldEditDraft {
    pub fn new(default_data_type_ref: DataTypeRef) -> Self {
        Self {
            field_name: String::new(),
            data_type_selection: DataTypeSelection::new(default_data_type_ref),
            container_edit: SymbolLayoutFieldContainerEdit::default(),
            is_hidden: false,
            active_when_resolver_id: String::new(),
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
            is_hidden: symbolic_field_definition.is_hidden(),
            active_when_resolver_id: symbolic_field_definition
                .get_active_when_resolver()
                .map(|resolver_ref| resolver_ref.get_resolver_id().to_string())
                .unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::{
        SymbolLayoutDefineFieldReturnState, SymbolLayoutEditDraft, SymbolLayoutEditorTakeOverState, SymbolLayoutEditorViewData,
        SymbolLayoutFieldContextMenuTarget, SymbolLayoutFieldEditDraft, SymbolLayoutFieldOffsetMode, SymbolLayoutUnassignedContextMenuTarget,
        SymbolLayoutUnassignedSelection,
    };
    use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::{SymbolLayoutFieldContainerEdit, SymbolLayoutFieldContainerKind};
    use epaint::pos2;
    use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::{
            built_in_types::{i32::data_type_i32::DataTypeI32, u8::data_type_u8::DataTypeU8},
            data_type_ref::DataTypeRef,
        },
        data_values::anonymous_value_string_format::AnonymousValueStringFormat,
        data_values::container_type::ContainerType,
        projects::{
            project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_module::ProjectSymbolModule,
            project_symbol_module_field::ProjectSymbolModuleField,
        },
        structs::{
            symbolic_field_definition::SymbolicFieldDefinition,
            symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
        },
    };
    use std::{collections::BTreeSet, str::FromStr};

    fn create_project_symbol_catalog() -> ProjectSymbolCatalog {
        ProjectSymbolCatalog::new_with_symbol_claims(
            vec![StructLayoutDescriptor::new(
                String::from("player.stats"),
                SymbolicStructDefinition::new(
                    String::from("player.stats"),
                    vec![SymbolicFieldDefinition::new_named(
                        String::from("health"),
                        DataTypeRef::new("u32"),
                        ContainerType::None,
                    )],
                ),
            )],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        )
    }

    fn create_field_draft(
        field_name: &str,
        data_type_id: &str,
        container_edit: SymbolLayoutFieldContainerEdit,
    ) -> SymbolLayoutFieldEditDraft {
        let mut field_draft = SymbolLayoutFieldEditDraft::new(DataTypeRef::new(data_type_id));
        field_draft.field_name = field_name.to_string();
        field_draft.container_edit = container_edit;

        field_draft
    }

    #[test]
    fn begin_define_field_from_new_layout_returns_to_create_takeover() {
        let dependency_container = DependencyContainer::new();
        let view_data = dependency_container.register(SymbolLayoutEditorViewData::new());

        {
            let mut view_data_write = view_data
                .write("Symbol layout define return test setup")
                .expect("Expected symbol layout view data write access in test.");
            view_data_write.take_over_state = Some(SymbolLayoutEditorTakeOverState::CreateSymbolLayout);
            view_data_write.draft = Some(SymbolLayoutEditDraft {
                original_layout_id: None,
                layout_id: String::from("new.struct"),
                layout_kind: SymbolicLayoutKind::Struct,
                size_text: String::from("256"),
                size_format: AnonymousValueStringFormat::Decimal,
                field_drafts: Vec::new(),
            });
        }

        SymbolLayoutEditorViewData::begin_define_field_from_unassigned_span(
            view_data.clone(),
            String::from("new.struct"),
            0,
            256,
            DataTypeRef::new(DataTypeI32::DATA_TYPE_ID),
        );

        let return_state = view_data
            .read("Symbol layout define return test")
            .and_then(|view_data_read| match view_data_read.get_take_over_state() {
                Some(SymbolLayoutEditorTakeOverState::DefineFieldFromUnassignedSpan { return_state, .. }) => Some(return_state.clone()),
                _ => None,
            });

        assert_eq!(return_state, Some(SymbolLayoutDefineFieldReturnState::CreateSymbolLayout));

        SymbolLayoutEditorViewData::return_to_define_field_source(view_data.clone(), SymbolLayoutDefineFieldReturnState::CreateSymbolLayout);

        let take_over_state = view_data
            .read("Symbol layout define return final state test")
            .and_then(|view_data_read| view_data_read.get_take_over_state().cloned());

        assert_eq!(take_over_state, Some(SymbolLayoutEditorTakeOverState::CreateSymbolLayout));
    }

    #[test]
    fn create_default_new_draft_picks_unique_layout_id() {
        let project_symbol_catalog = create_project_symbol_catalog();

        let draft = SymbolLayoutEditorViewData::create_default_new_draft(&project_symbol_catalog, DataTypeRef::new(DataTypeI32::DATA_TYPE_ID));

        assert_eq!(draft.layout_id, "new.struct");
        assert_eq!(
            draft.field_drafts.first().map(|field_draft| field_draft
                .data_type_selection
                .visible_data_type()
                .get_data_type_id()),
            Some(DataTypeI32::DATA_TYPE_ID)
        );
        assert_eq!(
            draft
                .field_drafts
                .first()
                .map(|field_draft| field_draft.field_name.as_str()),
            Some("field_1")
        );
    }

    #[test]
    fn remove_field_from_draft_focuses_next_available_field() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("7"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_field_draft("item_id", "u32", SymbolLayoutFieldContainerEdit::default()),
                create_field_draft("quantity", "u16", SymbolLayoutFieldContainerEdit::default()),
                create_field_draft("flags", "u8", SymbolLayoutFieldContainerEdit::default()),
            ],
        };

        let field_index_to_focus = SymbolLayoutEditorViewData::remove_field_from_draft(&mut draft, 1, DataTypeRef::new(DataTypeI32::DATA_TYPE_ID));

        assert_eq!(field_index_to_focus, Some(1));
        assert_eq!(
            draft
                .field_drafts
                .iter()
                .map(|field_draft| field_draft.field_name.as_str())
                .collect::<Vec<_>>(),
            vec!["item_id", "flags"]
        );
    }

    #[test]
    fn remove_field_from_draft_keeps_one_default_field() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("4"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "item_id",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        };

        let field_index_to_focus = SymbolLayoutEditorViewData::remove_field_from_draft(&mut draft, 0, DataTypeRef::new(DataTypeI32::DATA_TYPE_ID));

        assert_eq!(field_index_to_focus, Some(0));
        assert_eq!(draft.field_drafts.len(), 1);
        assert_eq!(
            draft.field_drafts.first().map(|field_draft| field_draft
                .data_type_selection
                .visible_data_type()
                .get_data_type_id()),
            Some(DataTypeI32::DATA_TYPE_ID)
        );
    }

    #[test]
    fn replace_draft_clears_stale_field_context_menu_target() {
        let mut view_data = SymbolLayoutEditorViewData::new();
        view_data.field_context_menu_target = Some(SymbolLayoutFieldContextMenuTarget::new(2, pos2(12.0, 34.0)));

        view_data.replace_draft(SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("4"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "item_id",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        });

        assert_eq!(view_data.get_field_context_menu_target(), None);
    }

    #[test]
    fn replace_draft_preserves_scoped_variant_field_selection() {
        let mut view_data = SymbolLayoutEditorViewData::new();
        view_data.selected_field_layout_id = Some(String::from("inventory.slot.variant_1"));
        view_data.selected_field_index = Some(2);

        view_data.replace_draft(SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Union,
            size_text: String::from("4"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "variant_1",
                "inventory.slot.variant_1",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        });

        assert_eq!(view_data.get_selected_field_layout_id(), Some("inventory.slot.variant_1"));
        assert_eq!(view_data.get_selected_field_index(), Some(2));
    }

    #[test]
    fn replace_draft_preserves_active_unassigned_context_menu_target() {
        let mut view_data = SymbolLayoutEditorViewData::new();
        let context_menu_target = SymbolLayoutUnassignedContextMenuTarget::new(4, 12, pos2(12.0, 34.0));
        view_data.unassigned_context_menu_target = Some(context_menu_target.clone());

        view_data.replace_draft(SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("16"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "item_id",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        });

        assert_eq!(view_data.get_unassigned_context_menu_target(), Some(&context_menu_target));
    }

    #[test]
    fn replace_draft_clears_stale_unassigned_context_menu_target() {
        let mut view_data = SymbolLayoutEditorViewData::new();
        view_data.unassigned_context_menu_target = Some(SymbolLayoutUnassignedContextMenuTarget::new(16, 12, pos2(12.0, 34.0)));

        view_data.replace_draft(SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("16"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "item_id",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        });

        assert_eq!(view_data.get_unassigned_context_menu_target(), None);
    }

    #[test]
    fn replace_draft_prunes_stale_unassigned_split_offsets() {
        let mut view_data = SymbolLayoutEditorViewData::new();
        view_data.unassigned_split_offsets = BTreeSet::from([0, 4, 12, 16]);

        view_data.replace_draft(SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("16"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "item_id",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        });

        assert_eq!(view_data.get_unassigned_split_offsets(), &BTreeSet::from([4, 12]));
    }

    #[test]
    fn split_unassigned_span_for_layout_keeps_variant_split_offsets_separate() {
        let dependency_container = DependencyContainer::new();
        let view_data = dependency_container.register(SymbolLayoutEditorViewData::new());

        let selected_span = SymbolLayoutEditorViewData::split_unassigned_span_for_layout(view_data.clone(), Some(String::from("variant.a")), 0, 16);

        let view_data_read = view_data
            .read("Symbol layout variant split offset test")
            .expect("Expected symbol layout view data read access in test.");

        assert_eq!(
            selected_span,
            Some(SymbolLayoutUnassignedSelection::new_for_layout(String::from("variant.a"), 0, 8))
        );
        assert_eq!(view_data_read.get_unassigned_split_offsets_for_layout(None), BTreeSet::new());
        assert_eq!(view_data_read.get_unassigned_split_offsets_for_layout(Some("variant.a")), BTreeSet::from([8]));
        assert_eq!(view_data_read.get_unassigned_split_offsets_for_layout(Some("variant.b")), BTreeSet::new());
    }

    #[test]
    fn select_field_for_layout_tracks_variant_field_selection() {
        let dependency_container = DependencyContainer::new();
        let view_data = dependency_container.register(SymbolLayoutEditorViewData::new());

        SymbolLayoutEditorViewData::select_field_for_layout(view_data.clone(), Some(String::from("variant.a")), 3);

        let view_data_read = view_data
            .read("Symbol layout variant field selection test")
            .expect("Expected symbol layout view data read access in test.");

        assert_eq!(view_data_read.get_selected_field_layout_id(), Some("variant.a"));
        assert_eq!(view_data_read.get_selected_field_index(), Some(3));
        assert_eq!(view_data_read.get_selected_unassigned_span(), None);
    }

    #[test]
    fn show_unassigned_context_menu_for_layout_tracks_variant_selection_target() {
        let dependency_container = DependencyContainer::new();
        let view_data = dependency_container.register(SymbolLayoutEditorViewData::new());

        SymbolLayoutEditorViewData::show_unassigned_context_menu_for_layout(
            view_data.clone(),
            Some(String::from("variant.a")),
            4,
            12,
            pos2(10.0, 20.0),
            None,
            None,
        );

        let view_data_read = view_data
            .read("Symbol layout variant selection target test")
            .expect("Expected symbol layout view data read access in test.");

        assert!(
            view_data_read
                .get_selected_unassigned_span()
                .is_some_and(|selected_span| selected_span.matches(Some("variant.a"), 4, 12))
        );
        assert_eq!(
            view_data_read
                .get_unassigned_context_menu_target()
                .and_then(SymbolLayoutUnassignedContextMenuTarget::get_layout_id),
            Some("variant.a")
        );
    }

    #[test]
    fn build_symbol_layout_descriptor_parses_container_suffixes() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("8"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "items",
                "u16",
                SymbolLayoutFieldContainerEdit {
                    kind: SymbolLayoutFieldContainerKind::FixedArray,
                    fixed_array_length: String::from("4"),
                    ..SymbolLayoutFieldContainerEdit::default()
                },
            )],
        };

        let struct_layout_descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected draft to build.");

        assert_eq!(struct_layout_descriptor.get_struct_layout_id(), "inventory.slot");
        assert_eq!(
            struct_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .first()
                .map(SymbolicFieldDefinition::to_string),
            Some(String::from("items:u16[4]"))
        );
    }

    #[test]
    fn build_symbol_layout_descriptor_preserves_union_kind() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("variant.payload"),
            layout_kind: SymbolicLayoutKind::Union,
            size_text: String::from("16"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_field_draft("as_u32", "u32", SymbolLayoutFieldContainerEdit::default()),
                create_field_draft(
                    "raw",
                    "u8",
                    SymbolLayoutFieldContainerEdit {
                        kind: SymbolLayoutFieldContainerKind::FixedArray,
                        fixed_array_length: String::from("16"),
                        ..SymbolLayoutFieldContainerEdit::default()
                    },
                ),
            ],
        };

        let struct_layout_descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected draft to build.");

        assert_eq!(
            struct_layout_descriptor
                .get_struct_layout_definition()
                .get_layout_kind(),
            SymbolicLayoutKind::Union
        );

        let round_trip_draft = SymbolLayoutEditorViewData::create_draft_from_descriptor(&struct_layout_descriptor);

        assert_eq!(round_trip_draft.layout_kind, SymbolicLayoutKind::Union);
    }

    #[test]
    fn build_symbol_layout_descriptor_persists_declared_size() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("0x20"),
            size_format: AnonymousValueStringFormat::Hexadecimal,
            field_drafts: vec![create_field_draft(
                "item_id",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        };

        let struct_layout_descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected draft to build.");

        assert_eq!(
            struct_layout_descriptor
                .get_struct_layout_definition()
                .get_declared_size_in_bytes(),
            Some(32)
        );
        assert_eq!(
            SymbolLayoutEditorViewData::create_draft_from_descriptor(&struct_layout_descriptor).size_text,
            "32"
        );
    }

    #[test]
    fn build_symbol_layout_descriptor_allows_empty_layout() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: Some(String::from("inventory.slot.variant_1")),
            layout_id: String::from("inventory.slot.variant_1"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("16"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: Vec::new(),
        };

        let struct_layout_descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected empty draft to build.");

        assert!(
            struct_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .is_empty()
        );
        assert_eq!(
            struct_layout_descriptor
                .get_struct_layout_definition()
                .get_declared_size_in_bytes(),
            Some(16)
        );
    }

    #[test]
    fn build_symbol_layout_descriptor_persists_split_unassigned_empty_layout() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: Some(String::from("module.root")),
            layout_id: String::from("module.root"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("16"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: Vec::new(),
        };
        let split_offsets = BTreeSet::from([8]);

        let struct_layout_descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(&project_symbol_catalog, &draft, &split_offsets)
                .expect("Expected split empty layout to build.");
        let fields = struct_layout_descriptor
            .get_struct_layout_definition()
            .get_fields();

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].to_string(), "unassigned[8]");
        assert_eq!(fields[1].to_string(), "unassigned[8]");
    }

    #[test]
    fn build_symbol_layout_descriptor_persists_split_unassigned_before_field() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let mut field_draft = create_field_draft("value", DataTypeU8::DATA_TYPE_ID, SymbolLayoutFieldContainerEdit::default());
        field_draft.offset_mode = SymbolLayoutFieldOffsetMode::Static;
        field_draft.static_offset_in_bytes = String::from("0x10");
        let draft = SymbolLayoutEditDraft {
            original_layout_id: Some(String::from("module.root")),
            layout_id: String::from("module.root"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("32"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![field_draft],
        };
        let split_offsets = BTreeSet::from([8]);

        let struct_layout_descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor_with_unassigned_split_offsets(&project_symbol_catalog, &draft, &split_offsets)
                .expect("Expected split field layout to build.");
        let fields = struct_layout_descriptor
            .get_struct_layout_definition()
            .get_fields();

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].to_string(), "unassigned[8]");
        assert_eq!(fields[1].to_string(), "unassigned[8]");
        assert_eq!(fields[2].to_string(), "value:u8");
    }

    #[test]
    fn begin_open_symbol_layout_restores_persisted_unassigned_split_offsets() {
        let dependency_container = DependencyContainer::new();
        let view_data = dependency_container.register(SymbolLayoutEditorViewData::new());
        let struct_layout_descriptor = StructLayoutDescriptor::new(
            String::from("module.root"),
            SymbolicStructDefinition::new(
                String::from("module.root"),
                vec![
                    SymbolicFieldDefinition::new_unassigned(8),
                    SymbolicFieldDefinition::new_unassigned(8),
                    SymbolicFieldDefinition::new_named(String::from("value"), DataTypeRef::new(DataTypeU8::DATA_TYPE_ID), ContainerType::None),
                ],
            )
            .with_declared_size_in_bytes(Some(32)),
        );
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![struct_layout_descriptor]);

        SymbolLayoutEditorViewData::begin_open_symbol_layout(view_data.clone(), &project_symbol_catalog, "module.root");

        let restored_split_offsets = view_data
            .read("Symbol layout restored split offsets")
            .map(|view_data_read| view_data_read.get_unassigned_split_offsets().clone())
            .unwrap_or_default();

        assert_eq!(restored_split_offsets, BTreeSet::from([8]));
    }

    #[test]
    fn build_symbol_layout_descriptor_rejects_size_that_truncates_fields() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("3"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "item_id",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        };

        let result = SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft);

        assert!(result.is_err_and(|error| error.contains("would truncate fields")));
    }

    #[test]
    fn draft_materializes_static_offsets_as_unassigned_entries() {
        let struct_layout_descriptor = StructLayoutDescriptor::new(
            String::from("image.headers"),
            SymbolicStructDefinition::new(
                String::from("image.headers"),
                vec![
                    SymbolicFieldDefinition::from_str("count:u24").expect("Expected count field to parse."),
                    SymbolicFieldDefinition::from_str("sections:win.Section[resolver(pe.section_count)] @ +4").expect("Expected static offset field to parse."),
                ],
            ),
        );

        let draft = SymbolLayoutEditorViewData::create_draft_from_descriptor(&struct_layout_descriptor);
        let sections_draft = draft.field_drafts.get(1).expect("Expected sections draft.");

        assert_eq!(sections_draft.container_edit.kind, SymbolLayoutFieldContainerKind::DynamicArray);
        assert_eq!(sections_draft.container_edit.dynamic_array_count_resolver_id, "pe.section_count");
        assert_eq!(sections_draft.offset_mode, SymbolLayoutFieldOffsetMode::Static);
        assert_eq!(sections_draft.static_offset_in_bytes, "4");
        assert_eq!(SymbolLayoutFieldEditDraft::parse_static_offset_text("+0x10"), Some(16));
        assert_eq!(SymbolLayoutFieldEditDraft::parse_static_offset_text("+0b10000"), Some(16));

        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let round_tripped_descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected static offset draft to build.");
        let round_tripped_fields = round_tripped_descriptor
            .get_struct_layout_definition()
            .get_fields();

        assert_eq!(round_tripped_fields[1].to_string(), "unassigned[1]");
        assert_eq!(round_tripped_fields[2].to_string(), "sections:win.Section[resolver(pe.section_count)]");
    }

    #[test]
    fn draft_round_trips_dynamic_count_and_offset_resolvers() {
        let struct_layout_descriptor = StructLayoutDescriptor::new(
            String::from("image.headers"),
            SymbolicStructDefinition::new(
                String::from("image.headers"),
                vec![
                    SymbolicFieldDefinition::from_str("count:u24").expect("Expected count field to parse."),
                    SymbolicFieldDefinition::from_str("sections:win.Section[resolver(pe.section_count)] @ resolver(pe.section_table)")
                        .expect("Expected dynamic field to parse."),
                ],
            ),
        );

        let draft = SymbolLayoutEditorViewData::create_draft_from_descriptor(&struct_layout_descriptor);
        let sections_draft = draft.field_drafts.get(1).expect("Expected sections draft.");

        assert_eq!(sections_draft.container_edit.kind, SymbolLayoutFieldContainerKind::DynamicArray);
        assert_eq!(sections_draft.container_edit.dynamic_array_count_resolver_id, "pe.section_count");
        assert_eq!(sections_draft.offset_mode, SymbolLayoutFieldOffsetMode::Resolver);
        assert_eq!(sections_draft.offset_resolver_id, "pe.section_table");

        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let round_tripped_descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected dynamic draft to build.");
        let round_tripped_field_text = round_tripped_descriptor
            .get_struct_layout_definition()
            .get_fields()[1]
            .to_string();

        assert_eq!(
            round_tripped_field_text,
            "sections:win.Section[resolver(pe.section_count)] @ resolver(pe.section_table)"
        );
    }

    #[test]
    fn draft_round_trips_hidden_fields() {
        let struct_layout_descriptor = StructLayoutDescriptor::new(
            String::from("header"),
            SymbolicStructDefinition::new(
                String::from("header"),
                vec![SymbolicFieldDefinition::from_str("reserved:u8[12] hidden").expect("Expected hidden field to parse.")],
            ),
        );

        let draft = SymbolLayoutEditorViewData::create_draft_from_descriptor(&struct_layout_descriptor);
        let field_draft = draft.field_drafts.first().expect("Expected field draft.");

        assert!(field_draft.is_hidden);

        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let round_tripped_descriptor =
            SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected hidden draft to build.");

        assert!(
            round_tripped_descriptor
                .get_struct_layout_definition()
                .get_fields()[0]
                .is_hidden()
        );
    }

    #[test]
    fn build_symbol_layout_descriptor_rejects_empty_dynamic_count_resolver() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("2"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "items",
                "u16",
                SymbolLayoutFieldContainerEdit {
                    kind: SymbolLayoutFieldContainerKind::DynamicArray,
                    ..SymbolLayoutFieldContainerEdit::default()
                },
            )],
        };

        let result = SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft);

        assert!(result.is_err_and(|error| error.contains("Dynamic array count resolver")));
    }

    #[test]
    fn build_symbol_layout_descriptor_rejects_empty_field_names() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("timer.state"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("4"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        };

        let result = SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft);

        assert!(result.is_err_and(|error| error.contains("Each field needs a name")));
    }

    #[test]
    fn build_symbol_layout_descriptor_rejects_duplicate_field_names() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("timer.state"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("8"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![
                create_field_draft("Timer", "u32", SymbolLayoutFieldContainerEdit::default()),
                create_field_draft("Timer", "u32", SymbolLayoutFieldContainerEdit::default()),
            ],
        };

        let result = SymbolLayoutEditorViewData::build_symbol_layout_descriptor(&project_symbol_catalog, &draft);

        assert!(result.is_err());
    }

    #[test]
    fn apply_draft_to_catalog_renames_symbol_claim_type_usage() {
        let project_symbol_catalog = create_project_symbol_catalog();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: Some(String::from("player.stats")),
            layout_id: String::from("player.profile"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("4"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "health",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        };

        let updated_project_symbol_catalog =
            SymbolLayoutEditorViewData::apply_draft_to_catalog(&project_symbol_catalog, &draft).expect("Expected draft to apply.");

        assert!(
            updated_project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == "player.profile")
        );
        assert_eq!(
            updated_project_symbol_catalog
                .get_symbol_claims()
                .first()
                .map(|symbol_claim| symbol_claim.get_struct_layout_id()),
            Some("player.profile")
        );
    }

    #[test]
    fn remove_symbol_layout_from_catalog_retargets_in_use_layouts_to_u8() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x1000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Player"), 0, String::from("player.stats")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![symbol_module],
            vec![StructLayoutDescriptor::new(
                String::from("player.container"),
                SymbolicStructDefinition::new(
                    String::from("player.container"),
                    vec![SymbolicFieldDefinition::new_named(
                        String::from("Stats"),
                        DataTypeRef::new("player.stats"),
                        ContainerType::None,
                    )],
                ),
            )],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        );

        let updated_project_symbol_catalog =
            SymbolLayoutEditorViewData::remove_symbol_layout_from_catalog(&project_symbol_catalog, "player.stats").expect("Expected layout to delete.");

        assert!(
            updated_project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .all(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() != "player.stats")
        );
        assert_eq!(
            updated_project_symbol_catalog
                .get_symbol_claims()
                .first()
                .map(ProjectSymbolClaim::get_struct_layout_id),
            Some(DataTypeU8::DATA_TYPE_ID)
        );
        assert_eq!(
            updated_project_symbol_catalog
                .get_symbol_modules()
                .first()
                .and_then(|symbol_module| symbol_module.get_fields().first())
                .map(ProjectSymbolModuleField::get_struct_layout_id),
            Some(DataTypeU8::DATA_TYPE_ID)
        );
        assert_eq!(
            updated_project_symbol_catalog
                .get_struct_layout_descriptors()
                .first()
                .and_then(|struct_layout_descriptor| {
                    struct_layout_descriptor
                        .get_struct_layout_definition()
                        .get_fields()
                        .first()
                })
                .map(|symbolic_field_definition| symbolic_field_definition.get_data_type_ref().get_data_type_id()),
            Some(DataTypeU8::DATA_TYPE_ID)
        );
    }

    #[test]
    fn count_symbol_claim_usages_includes_module_fields_and_nested_struct_fields() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x1000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Player"), 0, String::from("player.stats")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![symbol_module],
            vec![StructLayoutDescriptor::new(
                String::from("player.container"),
                SymbolicStructDefinition::new(
                    String::from("player.container"),
                    vec![SymbolicFieldDefinition::new_named(
                        String::from("Stats"),
                        DataTypeRef::new("player.stats"),
                        ContainerType::None,
                    )],
                ),
            )],
            vec![ProjectSymbolClaim::new_absolute_address(
                String::from("Player"),
                0x1234,
                String::from("player.stats"),
            )],
        );

        assert_eq!(
            SymbolLayoutEditorViewData::count_symbol_claim_usages(&project_symbol_catalog, "player.stats"),
            3
        );
    }

    #[test]
    fn apply_draft_to_catalog_renames_module_field_type_usage() {
        let mut symbol_module = ProjectSymbolModule::new(String::from("game.exe"), 0x1000);
        symbol_module
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Player"), 0, String::from("player.stats")));
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![symbol_module],
            vec![StructLayoutDescriptor::new(
                String::from("player.stats"),
                SymbolicStructDefinition::new(
                    String::from("player.stats"),
                    vec![SymbolicFieldDefinition::new_named(
                        String::from("health"),
                        DataTypeRef::new("u32"),
                        ContainerType::None,
                    )],
                ),
            )],
            Vec::new(),
        );
        let draft = SymbolLayoutEditDraft {
            original_layout_id: Some(String::from("player.stats")),
            layout_id: String::from("player.profile"),
            layout_kind: SymbolicLayoutKind::Struct,
            size_text: String::from("4"),
            size_format: AnonymousValueStringFormat::Decimal,
            field_drafts: vec![create_field_draft(
                "health",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        };

        let updated_project_symbol_catalog =
            SymbolLayoutEditorViewData::apply_draft_to_catalog(&project_symbol_catalog, &draft).expect("Expected draft to apply.");

        let module_field_type_id = updated_project_symbol_catalog
            .get_symbol_modules()
            .first()
            .and_then(|symbol_module| symbol_module.get_fields().first())
            .map(ProjectSymbolModuleField::get_struct_layout_id);

        assert_eq!(module_field_type_id, Some("player.profile"));
    }
}
