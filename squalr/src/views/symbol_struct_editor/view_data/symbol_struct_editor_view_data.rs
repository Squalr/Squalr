use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::views::symbol_struct_editor::view_data::symbol_struct_field_container_edit::SymbolStructFieldContainerEdit;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use squalr_engine_api::structures::{
    data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
    projects::project_symbol_catalog::ProjectSymbolCatalog,
    structs::{
        symbolic_expression::SymbolicExpression,
        symbolic_field_definition::{SymbolicFieldDefinition, SymbolicFieldOffsetResolution},
        symbolic_struct_definition::SymbolicStructDefinition,
    },
};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SymbolStructFieldOffsetMode {
    #[default]
    Sequential,
    Expression,
}

impl SymbolStructFieldOffsetMode {
    pub const ALL: [Self; 2] = [Self::Sequential, Self::Expression];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Sequential => "Sequential",
            Self::Expression => "Expression",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolStructFieldEditDraft {
    pub field_name: String,
    pub data_type_selection: DataTypeSelection,
    pub container_edit: SymbolStructFieldContainerEdit,
    pub offset_mode: SymbolStructFieldOffsetMode,
    pub offset_expression: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolStructLayoutEditDraft {
    pub original_layout_id: Option<String>,
    pub layout_id: String,
    pub field_drafts: Vec<SymbolStructFieldEditDraft>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolStructEditorTakeOverState {
    CreateStructLayout,
    EditStructLayout { layout_id: String },
    DeleteConfirmation { layout_id: String },
}

#[derive(Clone, Default)]
pub struct SymbolStructEditorViewData {
    selected_layout_id: Option<String>,
    filter_text: String,
    take_over_state: Option<SymbolStructEditorTakeOverState>,
    baseline_draft: Option<SymbolStructLayoutEditDraft>,
    draft: Option<SymbolStructLayoutEditDraft>,
    field_layout_editor_index: Option<usize>,
}

impl SymbolStructEditorViewData {
    pub fn new() -> Self {
        Self {
            selected_layout_id: None,
            filter_text: String::new(),
            take_over_state: None,
            baseline_draft: None,
            draft: None,
            field_layout_editor_index: None,
        }
    }

    pub fn get_selected_layout_id(&self) -> Option<&str> {
        self.selected_layout_id.as_deref()
    }

    pub fn get_filter_text(&self) -> &str {
        &self.filter_text
    }

    pub fn get_take_over_state(&self) -> Option<&SymbolStructEditorTakeOverState> {
        self.take_over_state.as_ref()
    }

    pub fn get_draft(&self) -> Option<&SymbolStructLayoutEditDraft> {
        self.draft.as_ref()
    }

    pub fn get_baseline_draft(&self) -> Option<&SymbolStructLayoutEditDraft> {
        self.baseline_draft.as_ref()
    }

    pub fn get_field_layout_editor_index(&self) -> Option<usize> {
        self.field_layout_editor_index
    }

    pub fn set_filter_text(
        symbol_struct_editor_view_data: Dependency<Self>,
        filter_text: String,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor set filter text") {
            symbol_struct_editor_view_data.filter_text = filter_text;
        }
    }

    pub fn update_draft(
        symbol_struct_editor_view_data: Dependency<Self>,
        draft: SymbolStructLayoutEditDraft,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor update draft") {
            if symbol_struct_editor_view_data
                .field_layout_editor_index
                .is_some_and(|field_index| field_index >= draft.field_drafts.len())
            {
                symbol_struct_editor_view_data.field_layout_editor_index = None;
            }
            symbol_struct_editor_view_data.draft = Some(draft);
        }
    }

    pub fn select_struct_layout(
        symbol_struct_editor_view_data: Dependency<Self>,
        selected_layout_id: Option<String>,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor select struct layout") {
            symbol_struct_editor_view_data.selected_layout_id = selected_layout_id;
        }
    }

    pub fn begin_create_struct_layout(
        symbol_struct_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        default_data_type_ref: DataTypeRef,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor begin create struct layout") {
            symbol_struct_editor_view_data.selected_layout_id = None;
            symbol_struct_editor_view_data.take_over_state = Some(SymbolStructEditorTakeOverState::CreateStructLayout);
            symbol_struct_editor_view_data.field_layout_editor_index = None;
            let baseline_draft = Self::create_default_new_draft(project_symbol_catalog, default_data_type_ref);
            symbol_struct_editor_view_data.baseline_draft = Some(baseline_draft.clone());
            symbol_struct_editor_view_data.draft = Some(baseline_draft);
        }
    }

    pub fn begin_edit_struct_layout(
        symbol_struct_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        layout_id: &str,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor begin edit struct layout") {
            symbol_struct_editor_view_data.selected_layout_id = Some(layout_id.to_string());
            symbol_struct_editor_view_data.take_over_state = Some(SymbolStructEditorTakeOverState::EditStructLayout {
                layout_id: layout_id.to_string(),
            });
            symbol_struct_editor_view_data.field_layout_editor_index = None;
            symbol_struct_editor_view_data.baseline_draft = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
                .map(Self::create_draft_from_descriptor);
            symbol_struct_editor_view_data.draft = symbol_struct_editor_view_data.baseline_draft.clone();
        }
    }

    pub fn request_delete_confirmation(
        symbol_struct_editor_view_data: Dependency<Self>,
        layout_id: String,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor request delete confirmation") {
            symbol_struct_editor_view_data.take_over_state = Some(SymbolStructEditorTakeOverState::DeleteConfirmation { layout_id });
            symbol_struct_editor_view_data.field_layout_editor_index = None;
        }
    }

    pub fn begin_field_layout_editor(
        symbol_struct_editor_view_data: Dependency<Self>,
        field_index: usize,
    ) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor begin field layout editor") {
            let Some(draft) = symbol_struct_editor_view_data.draft.as_ref() else {
                return;
            };

            if field_index < draft.field_drafts.len() {
                symbol_struct_editor_view_data.field_layout_editor_index = Some(field_index);
            }
        }
    }

    pub fn cancel_field_layout_editor(symbol_struct_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor cancel field layout editor") {
            symbol_struct_editor_view_data.field_layout_editor_index = None;
        }
    }

    pub fn cancel_take_over_state(symbol_struct_editor_view_data: Dependency<Self>) {
        if let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor cancel take over state") {
            symbol_struct_editor_view_data.take_over_state = None;
            symbol_struct_editor_view_data.baseline_draft = None;
            symbol_struct_editor_view_data.draft = None;
            symbol_struct_editor_view_data.field_layout_editor_index = None;
        }
    }

    pub fn synchronize(
        symbol_struct_editor_view_data: Dependency<Self>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) {
        let Some(mut symbol_struct_editor_view_data) = symbol_struct_editor_view_data.write("SymbolStructEditor synchronize") else {
            return;
        };

        let next_selected_layout_id = symbol_struct_editor_view_data
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

        symbol_struct_editor_view_data.selected_layout_id = next_selected_layout_id.clone();

        let should_clear_take_over_state = match symbol_struct_editor_view_data.take_over_state.as_ref() {
            Some(SymbolStructEditorTakeOverState::CreateStructLayout) => false,
            Some(SymbolStructEditorTakeOverState::EditStructLayout { layout_id }) | Some(SymbolStructEditorTakeOverState::DeleteConfirmation { layout_id }) => {
                !project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == layout_id)
            }
            None => false,
        };

        if should_clear_take_over_state {
            symbol_struct_editor_view_data.take_over_state = None;
            symbol_struct_editor_view_data.baseline_draft = None;
            symbol_struct_editor_view_data.draft = None;
            symbol_struct_editor_view_data.field_layout_editor_index = None;
        }

        if symbol_struct_editor_view_data
            .field_layout_editor_index
            .is_some_and(|field_index| {
                symbol_struct_editor_view_data
                    .draft
                    .as_ref()
                    .is_none_or(|draft| field_index >= draft.field_drafts.len())
            })
        {
            symbol_struct_editor_view_data.field_layout_editor_index = None;
        }
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

    pub fn create_draft_from_descriptor(struct_layout_descriptor: &StructLayoutDescriptor) -> SymbolStructLayoutEditDraft {
        SymbolStructLayoutEditDraft {
            original_layout_id: Some(struct_layout_descriptor.get_struct_layout_id().to_string()),
            layout_id: struct_layout_descriptor.get_struct_layout_id().to_string(),
            field_drafts: struct_layout_descriptor
                .get_struct_layout_definition()
                .get_fields()
                .iter()
                .map(SymbolStructFieldEditDraft::from_symbolic_field_definition)
                .collect(),
        }
    }

    pub fn create_default_new_draft(
        project_symbol_catalog: &ProjectSymbolCatalog,
        default_data_type_ref: DataTypeRef,
    ) -> SymbolStructLayoutEditDraft {
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

        SymbolStructLayoutEditDraft {
            original_layout_id: None,
            layout_id: proposed_layout_id,
            field_drafts: vec![SymbolStructFieldEditDraft::new(default_data_type_ref)],
        }
    }

    pub fn build_struct_layout_descriptor(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolStructLayoutEditDraft,
    ) -> Result<StructLayoutDescriptor, String> {
        let trimmed_layout_id = draft.layout_id.trim();
        if trimmed_layout_id.is_empty() {
            return Err(String::from("Struct layout id is required."));
        }

        let conflicts_with_existing_layout = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .any(|struct_layout_descriptor| {
                struct_layout_descriptor.get_struct_layout_id() == trimmed_layout_id && draft.original_layout_id.as_deref() != Some(trimmed_layout_id)
            });
        if conflicts_with_existing_layout {
            return Err(String::from("Struct layout id must be unique."));
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
            let offset_resolution = field_draft.to_offset_resolution()?;
            let trimmed_field_name = field_draft.field_name.trim().to_string();
            if !trimmed_field_name.is_empty() && !field_names.insert(trimmed_field_name.clone()) {
                return Err(format!("Field name `{}` is already used in this struct.", trimmed_field_name));
            }

            let data_type_ref = DataTypeRef::new(&trimmed_data_type_id);
            let symbolic_field_definition =
                SymbolicFieldDefinition::new_named_with_resolutions(trimmed_field_name, data_type_ref, container_type, count_resolution, offset_resolution);

            symbolic_field_definitions.push(symbolic_field_definition);
        }

        let struct_layout_descriptor = StructLayoutDescriptor::new(
            trimmed_layout_id.to_string(),
            SymbolicStructDefinition::new(trimmed_layout_id.to_string(), symbolic_field_definitions),
        );

        Self::validate_local_expression_dependency_cycles(&struct_layout_descriptor)?;

        Ok(struct_layout_descriptor)
    }

    fn validate_local_expression_dependency_cycles(struct_layout_descriptor: &StructLayoutDescriptor) -> Result<(), String> {
        let fields = struct_layout_descriptor
            .get_struct_layout_definition()
            .get_fields();
        let field_names = fields
            .iter()
            .filter_map(|field_definition| {
                let field_name = field_definition.get_field_name();

                (!field_name.is_empty()).then_some(field_name.to_string())
            })
            .collect::<HashSet<_>>();
        let mut dependencies_by_field_name = HashMap::new();

        for field_definition in fields {
            let field_name = field_definition.get_field_name();
            if field_name.is_empty() {
                continue;
            }

            let dependencies = Self::collect_local_field_expression_dependencies(field_definition, &field_names);
            dependencies_by_field_name.insert(field_name.to_string(), dependencies);
        }

        let mut visiting_field_names = HashSet::new();
        let mut visited_field_names = HashSet::new();
        let mut dependency_stack = Vec::new();

        for field_name in dependencies_by_field_name.keys() {
            if let Some(cycle_path) = Self::find_local_dependency_cycle(
                field_name,
                &dependencies_by_field_name,
                &mut visiting_field_names,
                &mut visited_field_names,
                &mut dependency_stack,
            ) {
                return Err(format!("Field layout expressions contain a dependency cycle: {}.", cycle_path.join(" -> ")));
            }
        }

        Ok(())
    }

    fn collect_local_field_expression_dependencies(
        field_definition: &SymbolicFieldDefinition,
        field_names: &HashSet<String>,
    ) -> Vec<String> {
        let mut dependencies = Vec::new();

        if let Some(expression) = field_definition.get_count_resolution().as_expression() {
            dependencies.extend(expression.referenced_identifiers());
        }

        if let SymbolicFieldOffsetResolution::Expression(expression) = field_definition.get_offset_resolution() {
            dependencies.extend(expression.referenced_identifiers());
        }

        dependencies.retain(|dependency| field_names.contains(dependency));
        dependencies.sort();
        dependencies.dedup();

        dependencies
    }

    fn find_local_dependency_cycle(
        field_name: &str,
        dependencies_by_field_name: &HashMap<String, Vec<String>>,
        visiting_field_names: &mut HashSet<String>,
        visited_field_names: &mut HashSet<String>,
        dependency_stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if visited_field_names.contains(field_name) {
            return None;
        }

        if visiting_field_names.contains(field_name) {
            let cycle_start_index = dependency_stack
                .iter()
                .position(|dependency_field_name| dependency_field_name == field_name)
                .unwrap_or(0);
            let mut cycle_path = dependency_stack[cycle_start_index..].to_vec();
            cycle_path.push(field_name.to_string());

            return Some(cycle_path);
        }

        visiting_field_names.insert(field_name.to_string());
        dependency_stack.push(field_name.to_string());

        if let Some(dependencies) = dependencies_by_field_name.get(field_name) {
            for dependency in dependencies {
                if let Some(cycle_path) = Self::find_local_dependency_cycle(
                    dependency,
                    dependencies_by_field_name,
                    visiting_field_names,
                    visited_field_names,
                    dependency_stack,
                ) {
                    return Some(cycle_path);
                }
            }
        }

        dependency_stack.pop();
        visiting_field_names.remove(field_name);
        visited_field_names.insert(field_name.to_string());

        None
    }

    pub fn apply_draft_to_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        draft: &SymbolStructLayoutEditDraft,
    ) -> Result<ProjectSymbolCatalog, String> {
        let resolved_struct_layout_descriptor = Self::build_struct_layout_descriptor(project_symbol_catalog, draft)?;
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
                for symbol_claim in updated_project_symbol_catalog.get_symbol_claims_mut() {
                    if symbol_claim.get_struct_layout_id() == original_layout_id {
                        symbol_claim.set_struct_layout_id(
                            resolved_struct_layout_descriptor
                                .get_struct_layout_id()
                                .to_string(),
                        );
                    }
                }

                for symbol_module in updated_project_symbol_catalog.get_symbol_modules_mut() {
                    for module_field in symbol_module.get_fields_mut() {
                        if module_field.get_struct_layout_id() == original_layout_id {
                            module_field.set_struct_layout_id(
                                resolved_struct_layout_descriptor
                                    .get_struct_layout_id()
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }

        Ok(updated_project_symbol_catalog)
    }

    pub fn remove_struct_layout_from_catalog(
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_id: &str,
    ) -> Result<ProjectSymbolCatalog, String> {
        if Self::count_symbol_claim_usages(project_symbol_catalog, struct_layout_id) > 0 {
            return Err(String::from("Struct layouts that are still used by symbol claims cannot be deleted."));
        }

        let mut updated_project_symbol_catalog = project_symbol_catalog.clone();
        updated_project_symbol_catalog.set_struct_layout_descriptors(
            updated_project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .filter(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() != struct_layout_id)
                .cloned()
                .collect(),
        );

        Ok(updated_project_symbol_catalog)
    }
}

impl SymbolStructFieldEditDraft {
    pub fn new(default_data_type_ref: DataTypeRef) -> Self {
        Self {
            field_name: String::new(),
            data_type_selection: DataTypeSelection::new(default_data_type_ref),
            container_edit: SymbolStructFieldContainerEdit::default(),
            offset_mode: SymbolStructFieldOffsetMode::Sequential,
            offset_expression: String::new(),
        }
    }

    pub fn from_symbolic_field_definition(symbolic_field_definition: &SymbolicFieldDefinition) -> Self {
        let (offset_mode, offset_expression) = match symbolic_field_definition.get_offset_resolution() {
            SymbolicFieldOffsetResolution::Sequential => (SymbolStructFieldOffsetMode::Sequential, String::new()),
            SymbolicFieldOffsetResolution::Expression(offset_expression) => (SymbolStructFieldOffsetMode::Expression, offset_expression.to_string()),
            SymbolicFieldOffsetResolution::Resolver(resolver_id) => (SymbolStructFieldOffsetMode::Expression, format!("resolver({})", resolver_id)),
        };

        Self {
            field_name: symbolic_field_definition.get_field_name().to_string(),
            data_type_selection: DataTypeSelection::new(symbolic_field_definition.get_data_type_ref().clone()),
            container_edit: SymbolStructFieldContainerEdit::from_symbolic_field_definition(symbolic_field_definition),
            offset_mode,
            offset_expression,
        }
    }

    pub fn to_offset_resolution(&self) -> Result<SymbolicFieldOffsetResolution, String> {
        match self.offset_mode {
            SymbolStructFieldOffsetMode::Sequential => Ok(SymbolicFieldOffsetResolution::Sequential),
            SymbolStructFieldOffsetMode::Expression => {
                let trimmed_expression = self.offset_expression.trim();
                if trimmed_expression.is_empty() {
                    return Err(String::from("Offset expression is required."));
                }

                if let Some(resolver_id) = parse_resolver_reference(trimmed_expression) {
                    return Ok(SymbolicFieldOffsetResolution::new_resolver(resolver_id));
                }

                Ok(SymbolicFieldOffsetResolution::new_expression(SymbolicExpression::from_str(trimmed_expression)?))
            }
        }
    }
}

fn parse_resolver_reference(resolver_reference: &str) -> Option<String> {
    let resolver_id = resolver_reference
        .strip_prefix("resolver(")?
        .strip_suffix(')')?
        .trim();

    (!resolver_id.is_empty()).then_some(resolver_id.to_string())
}

impl Default for SymbolStructLayoutEditDraft {
    fn default() -> Self {
        Self {
            original_layout_id: None,
            layout_id: String::new(),
            field_drafts: vec![SymbolStructFieldEditDraft::new(DataTypeRef::new(
                DataTypeI32::DATA_TYPE_ID,
            ))],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SymbolStructEditorViewData, SymbolStructFieldEditDraft, SymbolStructFieldOffsetMode, SymbolStructLayoutEditDraft};
    use crate::views::symbol_struct_editor::view_data::symbol_struct_field_container_edit::{SymbolStructFieldContainerEdit, SymbolStructFieldContainerKind};
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef},
        data_values::container_type::ContainerType,
        projects::{
            project_symbol_catalog::ProjectSymbolCatalog, project_symbol_claim::ProjectSymbolClaim, project_symbol_module::ProjectSymbolModule,
            project_symbol_module_field::ProjectSymbolModuleField,
        },
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
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
        container_edit: SymbolStructFieldContainerEdit,
    ) -> SymbolStructFieldEditDraft {
        let mut field_draft = SymbolStructFieldEditDraft::new(DataTypeRef::new(data_type_id));
        field_draft.field_name = field_name.to_string();
        field_draft.container_edit = container_edit;

        field_draft
    }

    #[test]
    fn create_default_new_draft_picks_unique_layout_id() {
        let project_symbol_catalog = create_project_symbol_catalog();

        let draft = SymbolStructEditorViewData::create_default_new_draft(&project_symbol_catalog, DataTypeRef::new(DataTypeI32::DATA_TYPE_ID));

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
    fn build_struct_layout_descriptor_parses_container_suffixes() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolStructLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            field_drafts: vec![create_field_draft(
                "items",
                "u16",
                SymbolStructFieldContainerEdit {
                    kind: SymbolStructFieldContainerKind::FixedArray,
                    fixed_array_length: String::from("4"),
                    ..SymbolStructFieldContainerEdit::default()
                },
            )],
        };

        let struct_layout_descriptor =
            SymbolStructEditorViewData::build_struct_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected draft to build.");

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
    fn draft_round_trips_dynamic_count_and_offset_expressions() {
        let struct_layout_descriptor = StructLayoutDescriptor::new(
            String::from("image.headers"),
            SymbolicStructDefinition::new(
                String::from("image.headers"),
                vec![
                    SymbolicFieldDefinition::from_str("count:u24 @ +0").expect("Expected count field to parse."),
                    SymbolicFieldDefinition::from_str("sections:win.Section[count] @ +4 + sizeof(win.Header)").expect("Expected dynamic field to parse."),
                ],
            ),
        );

        let draft = SymbolStructEditorViewData::create_draft_from_descriptor(&struct_layout_descriptor);
        let sections_draft = draft.field_drafts.get(1).expect("Expected sections draft.");

        assert_eq!(sections_draft.container_edit.kind, SymbolStructFieldContainerKind::DynamicArray);
        assert_eq!(sections_draft.container_edit.dynamic_array_count_expression, "count");
        assert_eq!(sections_draft.offset_mode, SymbolStructFieldOffsetMode::Expression);
        assert_eq!(sections_draft.offset_expression, "+4 + sizeof(win.Header)");

        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let round_tripped_descriptor =
            SymbolStructEditorViewData::build_struct_layout_descriptor(&project_symbol_catalog, &draft).expect("Expected dynamic draft to build.");
        let round_tripped_field_text = round_tripped_descriptor
            .get_struct_layout_definition()
            .get_fields()[1]
            .to_string();

        assert_eq!(round_tripped_field_text, "sections:win.Section[count] @ +4 + sizeof(win.Header)");
    }

    #[test]
    fn build_struct_layout_descriptor_rejects_empty_dynamic_count_expression() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolStructLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("inventory.slot"),
            field_drafts: vec![create_field_draft(
                "items",
                "u16",
                SymbolStructFieldContainerEdit {
                    kind: SymbolStructFieldContainerKind::DynamicArray,
                    ..SymbolStructFieldContainerEdit::default()
                },
            )],
        };

        let result = SymbolStructEditorViewData::build_struct_layout_descriptor(&project_symbol_catalog, &draft);

        assert!(result.is_err_and(|error| error.contains("Dynamic array count expression")));
    }

    #[test]
    fn build_struct_layout_descriptor_rejects_local_expression_dependency_cycles() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let mut first_field_draft = create_field_draft(
            "first",
            "u32",
            SymbolStructFieldContainerEdit {
                kind: SymbolStructFieldContainerKind::DynamicArray,
                dynamic_array_count_expression: String::from("second"),
                ..SymbolStructFieldContainerEdit::default()
            },
        );
        first_field_draft.offset_mode = SymbolStructFieldOffsetMode::Expression;
        first_field_draft.offset_expression = String::from("+0");
        let second_field_draft = create_field_draft(
            "second",
            "u32",
            SymbolStructFieldContainerEdit {
                kind: SymbolStructFieldContainerKind::DynamicArray,
                dynamic_array_count_expression: String::from("first"),
                ..SymbolStructFieldContainerEdit::default()
            },
        );
        let draft = SymbolStructLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("bad.cycle"),
            field_drafts: vec![first_field_draft, second_field_draft],
        };

        let result = SymbolStructEditorViewData::build_struct_layout_descriptor(&project_symbol_catalog, &draft);

        assert!(result.is_err_and(|error| error.contains("dependency cycle")));
    }

    #[test]
    fn build_struct_layout_descriptor_rejects_duplicate_field_names() {
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let draft = SymbolStructLayoutEditDraft {
            original_layout_id: None,
            layout_id: String::from("timer.state"),
            field_drafts: vec![
                create_field_draft("Timer", "u32", SymbolStructFieldContainerEdit::default()),
                create_field_draft("Timer", "u32", SymbolStructFieldContainerEdit::default()),
            ],
        };

        let result = SymbolStructEditorViewData::build_struct_layout_descriptor(&project_symbol_catalog, &draft);

        assert!(result.is_err());
    }

    #[test]
    fn apply_draft_to_catalog_renames_symbol_claim_type_usage() {
        let project_symbol_catalog = create_project_symbol_catalog();
        let draft = SymbolStructLayoutEditDraft {
            original_layout_id: Some(String::from("player.stats")),
            layout_id: String::from("player.profile"),
            field_drafts: vec![create_field_draft(
                "health",
                "u32",
                SymbolStructFieldContainerEdit::default(),
            )],
        };

        let updated_project_symbol_catalog =
            SymbolStructEditorViewData::apply_draft_to_catalog(&project_symbol_catalog, &draft).expect("Expected draft to apply.");

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
    fn remove_struct_layout_from_catalog_rejects_in_use_layouts() {
        let project_symbol_catalog = create_project_symbol_catalog();

        let result = SymbolStructEditorViewData::remove_struct_layout_from_catalog(&project_symbol_catalog, "player.stats");

        assert!(result.is_err());
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
            SymbolStructEditorViewData::count_symbol_claim_usages(&project_symbol_catalog, "player.stats"),
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
        let draft = SymbolStructLayoutEditDraft {
            original_layout_id: Some(String::from("player.stats")),
            layout_id: String::from("player.profile"),
            field_drafts: vec![create_field_draft(
                "health",
                "u32",
                SymbolStructFieldContainerEdit::default(),
            )],
        };

        let updated_project_symbol_catalog =
            SymbolStructEditorViewData::apply_draft_to_catalog(&project_symbol_catalog, &draft).expect("Expected draft to apply.");

        let module_field_type_id = updated_project_symbol_catalog
            .get_symbol_modules()
            .first()
            .and_then(|symbol_module| symbol_module.get_fields().first())
            .map(ProjectSymbolModuleField::get_struct_layout_id);

        assert_eq!(module_field_type_id, Some("player.profile"));
    }
}
