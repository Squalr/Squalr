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
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::{
        symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
    },
};
use std::collections::HashSet;

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
    pub offset_mode: SymbolLayoutFieldOffsetMode,
    pub static_offset_in_bytes: String,
    pub offset_resolver_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolLayoutEditDraft {
    pub original_layout_id: Option<String>,
    pub layout_id: String,
    pub layout_kind: SymbolicLayoutKind,
    pub field_drafts: Vec<SymbolLayoutFieldEditDraft>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolLayoutEditorTakeOverState {
    CreateSymbolLayout,
    RenameSymbolLayout { layout_id: String },
    OpenSymbolLayout { layout_id: String },
    DeleteConfirmation { layout_id: String },
    DeleteFieldConfirmation { layout_id: String, field_index: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolLayoutFieldContextMenuTarget {
    field_index: usize,
    position: Pos2,
}

impl SymbolLayoutFieldContextMenuTarget {
    pub fn new(
        field_index: usize,
        position: Pos2,
    ) -> Self {
        Self { field_index, position }
    }

    pub fn get_field_index(&self) -> usize {
        self.field_index
    }

    pub fn get_position(&self) -> Pos2 {
        self.position
    }
}

#[derive(Clone, Default)]
pub struct SymbolLayoutEditorViewData {
    selected_layout_id: Option<String>,
    filter_text: String,
    take_over_state: Option<SymbolLayoutEditorTakeOverState>,
    baseline_draft: Option<SymbolLayoutEditDraft>,
    draft: Option<SymbolLayoutEditDraft>,
    selected_field_index: Option<usize>,
    field_context_menu_target: Option<SymbolLayoutFieldContextMenuTarget>,
}

impl SymbolLayoutEditorViewData {
    pub fn new() -> Self {
        Self {
            selected_layout_id: None,
            filter_text: String::new(),
            take_over_state: None,
            baseline_draft: None,
            draft: None,
            selected_field_index: None,
            field_context_menu_target: None,
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

    pub fn get_selected_field_index(&self) -> Option<usize> {
        self.selected_field_index
    }

    pub fn get_field_context_menu_target(&self) -> Option<&SymbolLayoutFieldContextMenuTarget> {
        self.field_context_menu_target.as_ref()
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
            .is_some_and(|field_index| field_index >= draft.field_drafts.len())
        {
            self.selected_field_index = None;
        }
        if self
            .field_context_menu_target
            .as_ref()
            .is_some_and(|context_menu_target| context_menu_target.get_field_index() >= draft.field_drafts.len())
        {
            self.field_context_menu_target = None;
        }
        self.draft = Some(draft);
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
            symbol_layout_editor_view_data.field_context_menu_target = None;
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
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.baseline_draft = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
                .map(Self::create_draft_from_descriptor);
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
            symbol_layout_editor_view_data.field_context_menu_target = None;
            symbol_layout_editor_view_data.baseline_draft = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
                .map(Self::create_draft_from_descriptor);
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
            symbol_layout_editor_view_data.field_context_menu_target = None;
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
            symbol_layout_editor_view_data.field_context_menu_target = None;
        }
    }

    pub fn return_to_open_symbol_layout(
        symbol_layout_editor_view_data: Dependency<Self>,
        layout_id: String,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor return to open symbol layout") {
            symbol_layout_editor_view_data.take_over_state = Some(SymbolLayoutEditorTakeOverState::OpenSymbolLayout { layout_id });
            symbol_layout_editor_view_data.field_context_menu_target = None;
        }
    }

    pub fn show_field_context_menu(
        symbol_layout_editor_view_data: Dependency<Self>,
        field_index: usize,
        position: Pos2,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor show field context menu") {
            symbol_layout_editor_view_data.field_context_menu_target = Some(SymbolLayoutFieldContextMenuTarget::new(field_index, position));
            symbol_layout_editor_view_data.selected_field_index = Some(field_index);
        }
    }

    pub fn hide_field_context_menu(symbol_layout_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor hide field context menu") {
            symbol_layout_editor_view_data.field_context_menu_target = None;
        }
    }

    pub fn select_field(
        symbol_layout_editor_view_data: Dependency<Self>,
        field_index: usize,
    ) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor select field") {
            symbol_layout_editor_view_data.selected_field_index = Some(field_index);
            symbol_layout_editor_view_data.field_context_menu_target = None;
        }
    }

    pub fn clear_field_selection(symbol_layout_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor clear field selection") {
            symbol_layout_editor_view_data.selected_field_index = None;
        }
    }

    pub fn cancel_take_over_state(symbol_layout_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_layout_editor_view_data) = symbol_layout_editor_view_data.write("SymbolLayoutEditor cancel take over state") {
            symbol_layout_editor_view_data.take_over_state = None;
            symbol_layout_editor_view_data.baseline_draft = None;
            symbol_layout_editor_view_data.draft = None;
            symbol_layout_editor_view_data.selected_field_index = None;
            symbol_layout_editor_view_data.field_context_menu_target = None;
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
            Some(
                SymbolLayoutEditorTakeOverState::RenameSymbolLayout { layout_id }
                | SymbolLayoutEditorTakeOverState::OpenSymbolLayout { layout_id }
                | SymbolLayoutEditorTakeOverState::DeleteConfirmation { layout_id }
                | SymbolLayoutEditorTakeOverState::DeleteFieldConfirmation { layout_id, .. },
            ) => !project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id),
            None => false,
        };

        if should_clear_take_over_state {
            symbol_layout_editor_view_data.take_over_state = None;
            symbol_layout_editor_view_data.baseline_draft = None;
            symbol_layout_editor_view_data.draft = None;
            symbol_layout_editor_view_data.selected_field_index = None;
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
        }

        if symbol_layout_editor_view_data
            .selected_field_index
            .is_some_and(|field_index| {
                symbol_layout_editor_view_data
                    .draft
                    .as_ref()
                    .is_none_or(|draft| field_index >= draft.field_drafts.len())
            })
        {
            symbol_layout_editor_view_data.selected_field_index = None;
        }

        if symbol_layout_editor_view_data
            .field_context_menu_target
            .as_ref()
            .is_some_and(|context_menu_target| {
                symbol_layout_editor_view_data
                    .draft
                    .as_ref()
                    .is_none_or(|draft| context_menu_target.get_field_index() >= draft.field_drafts.len())
            })
        {
            symbol_layout_editor_view_data.field_context_menu_target = None;
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
        SymbolLayoutEditDraft {
            original_layout_id: Some(struct_layout_descriptor.get_struct_layout_id().to_string()),
            layout_id: struct_layout_descriptor.get_struct_layout_id().to_string(),
            layout_kind: struct_layout_descriptor
                .get_struct_layout_definition()
                .get_layout_kind(),
            field_drafts: struct_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .iter()
                .map(SymbolLayoutFieldEditDraft::from_symbolic_field_definition)
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

        SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: proposed_layout_id,
            layout_kind: SymbolicLayoutKind::Struct,
            field_drafts: vec![SymbolLayoutFieldEditDraft::new(default_data_type_ref)],
        }
    }

    pub fn build_symbol_layout_descriptor(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
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

        let mut symbolic_field_definitions = Vec::with_capacity(draft.field_drafts.len());
        let mut field_names = HashSet::new();
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
            if !trimmed_field_name.is_empty() && !field_names.insert(trimmed_field_name.clone()) {
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
            .with_hidden(field_draft.is_hidden);

            symbolic_field_definitions.push(symbolic_field_definition);
        }

        let struct_layout_descriptor = StructLayoutDescriptor::new(
            trimmed_layout_id.to_string(),
            SymbolicStructDefinition::new_with_layout_kind(trimmed_layout_id.to_string(), draft.layout_kind, symbolic_field_definitions),
        );

        project_symbol_catalog.validate_local_resolver_dependencies_for_struct_layout(&struct_layout_descriptor)?;

        Ok(struct_layout_descriptor)
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
    }

    pub fn apply_draft_to_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolLayoutEditDraft,
    ) -> Result<ProjectSymbolCatalog, String> {
        let resolved_struct_layout_descriptor = Self::build_symbol_layout_descriptor(project_symbol_catalog, draft)?;
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
                    return Err(String::from("Static offset is required."));
                }

                let offset_in_bytes = Self::parse_static_offset_text(trimmed_offset).ok_or_else(|| format!("Invalid static offset: {}.", trimmed_offset))?;

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
            field_drafts: vec![SymbolLayoutFieldEditDraft::new(DataTypeRef::new(
                DataTypeI32::DATA_TYPE_ID,
            ))],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SymbolLayoutEditDraft, SymbolLayoutEditorViewData, SymbolLayoutFieldContextMenuTarget, SymbolLayoutFieldEditDraft, SymbolLayoutFieldOffsetMode,
    };
    use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::{SymbolLayoutFieldContainerEdit, SymbolLayoutFieldContainerKind};
    use epaint::pos2;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::{
            built_in_types::{i32::data_type_i32::DataTypeI32, u8::data_type_u8::DataTypeU8},
            data_type_ref::DataTypeRef,
        },
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
    use std::str::FromStr;

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
    }

    #[test]
    fn remove_field_from_draft_focuses_next_available_field() {
        let mut draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
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
            field_drafts: vec![create_field_draft(
                "item_id",
                "u32",
                SymbolLayoutFieldContainerEdit::default(),
            )],
        });

        assert_eq!(view_data.get_field_context_menu_target(), None);
    }

    #[test]
    fn build_symbol_layout_descriptor_parses_container_suffixes() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            layout_kind: SymbolicLayoutKind::Struct,
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
    fn draft_round_trips_static_offsets() {
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

        assert_eq!(
            round_tripped_descriptor
                .get_struct_layout_definition()
                .get_fields()[1]
                .to_string(),
            "sections:win.Section[resolver(pe.section_count)] @ +4"
        );
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
    fn build_symbol_layout_descriptor_rejects_duplicate_field_names() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("timer.state"),
            layout_kind: SymbolicLayoutKind::Struct,
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
