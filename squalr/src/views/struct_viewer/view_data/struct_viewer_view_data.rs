use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::views::struct_viewer::view_data::struct_viewer_container_mode::StructViewerContainerMode;
use crate::views::struct_viewer::view_data::struct_viewer_field_presentation::{StructViewerFieldEditorKind, StructViewerFieldPresentation};
use crate::views::struct_viewer::view_data::struct_viewer_focus_target::StructViewerFocusTarget;
use squalr_engine_api::plugins::instruction_set::normalize_instruction_data_type_id;
use squalr_engine_api::{
    dependency_injection::dependency::Dependency,
    structures::{
        data_types::{
            built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u8::data_type_u8::DataTypeU8, u64::data_type_u64::DataTypeU64},
            data_type_ref::DataTypeRef,
        },
        data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType},
        projects::project_items::built_in_types::{project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer},
        structs::{
            symbolic_field_definition::SymbolicFieldDefinition,
            valued_struct::ValuedStruct,
            valued_struct_field::{ValuedStructField, ValuedStructFieldData},
        },
    },
};
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub struct StructViewerViewData {
    pub source_struct_under_view: Arc<Option<ValuedStruct>>,
    pub struct_under_view: Arc<Option<ValuedStruct>>,
    pub struct_field_modified_callback: Option<Arc<dyn Fn(ValuedStructField) + Send + Sync>>,
    pub focus_target: Arc<Option<StructViewerFocusTarget>>,
    pub selected_field_name: Arc<Option<String>>,
    pub field_edit_values: HashMap<String, AnonymousValueString>,
    pub field_display_values: HashMap<String, Vec<AnonymousValueString>>,
    pub field_presentations: HashMap<String, StructViewerFieldPresentation>,
    pub field_validation_data_type_refs: HashMap<String, DataTypeRef>,
    pub field_data_type_selections: HashMap<String, DataTypeSelection>,
    pub value_splitter_ratio: f32,
}

impl StructViewerViewData {
    pub const DEFAULT_NAME_SPLITTER_RATIO: f32 = 0.5;
    pub const VIRTUAL_FIELD_CONTAINER_TYPE: &'static str = "__virtual_container_type";
    pub const VIRTUAL_FIELD_ARRAY_SIZE: &'static str = "__virtual_array_size";

    pub fn new() -> Self {
        Self {
            source_struct_under_view: Arc::new(None),
            struct_under_view: Arc::new(None),
            struct_field_modified_callback: None,
            focus_target: Arc::new(None),
            selected_field_name: Arc::new(None),
            field_edit_values: HashMap::new(),
            field_display_values: HashMap::new(),
            field_presentations: HashMap::new(),
            field_validation_data_type_refs: HashMap::new(),
            field_data_type_selections: HashMap::new(),
            value_splitter_ratio: Self::DEFAULT_NAME_SPLITTER_RATIO,
        }
    }

    pub fn set_selected_field(
        struct_viewer_view_data: Dependency<Self>,
        valued_struct_field_name: String,
    ) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Set selected field") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };

        struct_viewer_view_data.selected_field_name = Arc::new(Some(valued_struct_field_name));
    }

    pub fn clear_selected_field(struct_viewer_view_data: Dependency<Self>) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Clear selected field") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };

        struct_viewer_view_data.selected_field_name = Arc::new(None);
    }

    pub fn focus_valued_struct(
        struct_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        valued_struct: ValuedStruct,
        valued_struct_field_edited_callback: Arc<dyn Fn(ValuedStructField) + Send + Sync>,
    ) {
        Self::focus_valued_struct_with_focus_target(
            struct_viewer_view_data,
            engine_unprivileged_state,
            valued_struct,
            valued_struct_field_edited_callback,
            None,
        );
    }

    pub fn focus_valued_struct_with_focus_target(
        struct_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        valued_struct: ValuedStruct,
        valued_struct_field_edited_callback: Arc<dyn Fn(ValuedStructField) + Send + Sync>,
        focus_target: Option<StructViewerFocusTarget>,
    ) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Focus valued struct") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };
        struct_viewer_view_data.set_valued_struct_and_callback(
            engine_unprivileged_state,
            Some(valued_struct),
            Some(valued_struct_field_edited_callback),
            focus_target,
        );
    }

    pub fn focus_valued_structs(
        struct_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        valued_structs: Vec<ValuedStruct>,
        valued_struct_field_edited_callback: Arc<dyn Fn(ValuedStructField) + Send + Sync>,
    ) {
        Self::focus_valued_structs_with_focus_target(
            struct_viewer_view_data,
            engine_unprivileged_state,
            valued_structs,
            valued_struct_field_edited_callback,
            None,
        );
    }

    pub fn focus_valued_structs_with_focus_target(
        struct_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        valued_structs: Vec<ValuedStruct>,
        valued_struct_field_edited_callback: Arc<dyn Fn(ValuedStructField) + Send + Sync>,
        focus_target: Option<StructViewerFocusTarget>,
    ) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Focus valued struct") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };
        let valued_struct = ValuedStruct::combine_exclusive(&valued_structs);

        struct_viewer_view_data.set_valued_struct_and_callback(
            engine_unprivileged_state,
            Some(valued_struct),
            Some(valued_struct_field_edited_callback),
            focus_target,
        );
    }

    pub fn clear_focus(struct_viewer_view_data: Dependency<Self>) {
        let mut struct_viewer_view_data = match struct_viewer_view_data.write("Focus valued struct") {
            Some(struct_viewer_view_data) => struct_viewer_view_data,
            None => return,
        };
        struct_viewer_view_data.field_presentations.clear();
        struct_viewer_view_data.field_edit_values.clear();
        struct_viewer_view_data.field_display_values.clear();
        struct_viewer_view_data.field_validation_data_type_refs.clear();
        struct_viewer_view_data.field_data_type_selections.clear();
        struct_viewer_view_data.selected_field_name = Arc::new(None);
        struct_viewer_view_data.source_struct_under_view = Arc::new(None);
        struct_viewer_view_data.struct_under_view = Arc::new(None);
        struct_viewer_view_data.struct_field_modified_callback = None;
        struct_viewer_view_data.focus_target = Arc::new(None);
    }

    pub fn get_focus_target(&self) -> Option<&StructViewerFocusTarget> {
        self.focus_target.as_ref().as_ref()
    }

    pub fn build_read_only_presented_state(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        valued_struct: ValuedStruct,
    ) -> Self {
        let mut struct_viewer_view_data = Self::new();

        struct_viewer_view_data.source_struct_under_view = Arc::new(Some(valued_struct));
        struct_viewer_view_data.refresh_cached_field_state(engine_unprivileged_state);

        struct_viewer_view_data
    }

    fn set_valued_struct_and_callback(
        &mut self,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        valued_struct: Option<ValuedStruct>,
        valued_struct_field_edited_callback: Option<Arc<dyn Fn(ValuedStructField) + Send + Sync>>,
        focus_target: Option<StructViewerFocusTarget>,
    ) {
        self.selected_field_name = Arc::new(None);
        self.source_struct_under_view = Arc::new(valued_struct);
        self.struct_field_modified_callback = valued_struct_field_edited_callback;
        self.focus_target = Arc::new(focus_target);
        self.refresh_cached_field_state(&engine_unprivileged_state);
    }

    pub fn refresh_cached_field_state(
        &mut self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) {
        let Some(source_struct_under_view) = self.source_struct_under_view.as_ref().as_ref() else {
            self.field_presentations.clear();
            self.field_edit_values.clear();
            self.field_display_values.clear();
            self.field_validation_data_type_refs.clear();
            self.field_data_type_selections.clear();
            self.struct_under_view = Arc::new(None);
            return;
        };
        let struct_under_view = Self::create_presented_struct(source_struct_under_view);
        let field_validation_data_type_refs = Self::create_field_validation_data_type_refs(&struct_under_view, engine_unprivileged_state);

        self.struct_under_view = Arc::new(Some(struct_under_view.clone()));
        self.field_presentations = Self::create_field_presentations(&struct_under_view);
        self.field_edit_values = Self::create_field_edit_values(&struct_under_view, &field_validation_data_type_refs, engine_unprivileged_state);
        self.field_display_values = Self::create_field_display_values(&struct_under_view, &field_validation_data_type_refs, engine_unprivileged_state);
        self.field_validation_data_type_refs = field_validation_data_type_refs;
        self.field_data_type_selections = Self::create_field_data_type_selections(&struct_under_view);
    }

    pub fn resolve_source_field_edit(
        &self,
        edited_field: &ValuedStructField,
    ) -> Option<ValuedStructField> {
        if Self::is_virtual_container_type_field(edited_field) {
            let symbolic_field_definition = self
                .source_struct_under_view
                .as_ref()
                .as_ref()
                .and_then(Self::read_symbolic_field_definition_reference)?;
            let container_mode = Self::read_container_mode_value(edited_field)?;
            let updated_container_type = match container_mode {
                StructViewerContainerMode::Element => ContainerType::None,
                StructViewerContainerMode::Array => ContainerType::ArrayFixed(Self::fixed_array_length_from_symbolic_definition(&symbolic_field_definition)),
            };

            return Some(Self::build_symbolic_field_definition_edit(
                &symbolic_field_definition,
                symbolic_field_definition.get_data_type_ref().clone(),
                updated_container_type,
            ));
        }

        if Self::is_virtual_array_size_field(edited_field) {
            let symbolic_field_definition = self
                .source_struct_under_view
                .as_ref()
                .as_ref()
                .and_then(Self::read_symbolic_field_definition_reference)?;
            let array_size = Self::read_u64_field_value(edited_field)?.max(1);

            return Some(Self::build_symbolic_field_definition_edit(
                &symbolic_field_definition,
                symbolic_field_definition.get_data_type_ref().clone(),
                ContainerType::ArrayFixed(array_size),
            ));
        }

        Some(edited_field.clone())
    }

    fn create_field_edit_values(
        valued_struct: &ValuedStruct,
        field_validation_data_type_refs: &HashMap<String, DataTypeRef>,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> HashMap<String, AnonymousValueString> {
        let mut field_edit_values = HashMap::new();
        let symbolic_field_container_type = Self::read_symbolic_field_definition_reference(valued_struct)
            .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
            .unwrap_or(ContainerType::None);

        for valued_struct_field in valued_struct.get_fields() {
            if Self::is_live_value_field(valued_struct_field) {
                let live_value_edit_value = Self::create_live_value_edit_value(
                    valued_struct_field,
                    field_validation_data_type_refs.get(valued_struct_field.get_name()),
                    symbolic_field_container_type,
                    engine_unprivileged_state,
                );

                field_edit_values.insert(valued_struct_field.get_name().to_string(), live_value_edit_value);
                continue;
            }

            let Some(data_value) = valued_struct_field.get_data_value() else {
                continue;
            };
            let data_type_ref = data_value.get_data_type_ref();
            let default_format = engine_unprivileged_state.get_default_anonymous_value_string_format(data_type_ref);
            let anonymous_value_string = engine_unprivileged_state
                .anonymize_value(data_value, default_format)
                .unwrap_or_else(|_| AnonymousValueString::new(String::new(), default_format, ContainerType::None));

            field_edit_values.insert(valued_struct_field.get_name().to_string(), anonymous_value_string);
        }

        field_edit_values
    }

    fn create_field_display_values(
        valued_struct: &ValuedStruct,
        field_validation_data_type_refs: &HashMap<String, DataTypeRef>,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> HashMap<String, Vec<AnonymousValueString>> {
        let mut field_display_values = HashMap::new();
        let symbolic_field_container_type = Self::read_symbolic_field_definition_reference(valued_struct)
            .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
            .unwrap_or(ContainerType::None);

        for valued_struct_field in valued_struct.get_fields() {
            if Self::is_live_value_field(valued_struct_field) {
                let live_value_display_values = Self::create_live_value_display_values(
                    valued_struct_field,
                    field_validation_data_type_refs.get(valued_struct_field.get_name()),
                    symbolic_field_container_type,
                    engine_unprivileged_state,
                );

                field_display_values.insert(valued_struct_field.get_name().to_string(), live_value_display_values);
                continue;
            }

            let Some(data_value) = valued_struct_field.get_data_value() else {
                continue;
            };

            let display_values = engine_unprivileged_state
                .anonymize_value_to_supported_formats(data_value)
                .unwrap_or_else(|_| {
                    let data_type_ref = data_value.get_data_type_ref();
                    let default_format = engine_unprivileged_state.get_default_anonymous_value_string_format(data_type_ref);
                    vec![
                        engine_unprivileged_state
                            .anonymize_value(data_value, default_format)
                            .unwrap_or_else(|_| AnonymousValueString::new(String::new(), default_format, ContainerType::None)),
                    ]
                });

            field_display_values.insert(valued_struct_field.get_name().to_string(), display_values);
        }

        field_display_values
    }

    fn create_live_value_edit_value(
        valued_struct_field: &ValuedStructField,
        validation_data_type_ref: Option<&DataTypeRef>,
        container_type: ContainerType,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> AnonymousValueString {
        let raw_display_value = Self::read_utf8_field_text(valued_struct_field);
        let anonymous_value_string_format = validation_data_type_ref
            .map(|validation_data_type_ref| engine_unprivileged_state.get_default_anonymous_value_string_format(validation_data_type_ref))
            .unwrap_or_default();

        AnonymousValueString::new(raw_display_value, anonymous_value_string_format, container_type)
    }

    fn create_live_value_display_values(
        valued_struct_field: &ValuedStructField,
        validation_data_type_ref: Option<&DataTypeRef>,
        container_type: ContainerType,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> Vec<AnonymousValueString> {
        let Some(validation_data_type_ref) = validation_data_type_ref else {
            return Vec::new();
        };
        let live_value_edit_value =
            Self::create_live_value_edit_value(valued_struct_field, Some(validation_data_type_ref), container_type, engine_unprivileged_state);
        let Ok(data_value) = engine_unprivileged_state.deanonymize_value_string(validation_data_type_ref, &live_value_edit_value) else {
            return Vec::new();
        };

        engine_unprivileged_state
            .anonymize_value_to_supported_formats(&data_value)
            .unwrap_or_else(|_| vec![live_value_edit_value])
    }

    fn create_field_presentations(valued_struct: &ValuedStruct) -> HashMap<String, StructViewerFieldPresentation> {
        let mut field_presentations = HashMap::new();
        let live_value_uses_external_viewer = Self::is_live_value_external_viewer_editable(valued_struct);
        let live_value_uses_code_viewer = Self::is_live_value_code_viewer_editable(valued_struct);

        for valued_struct_field in valued_struct.get_fields() {
            let field_presentation = if Self::is_data_type_reference_field(valued_struct_field) {
                StructViewerFieldPresentation::new(String::from("data_type"), StructViewerFieldEditorKind::DataTypeSelector)
            } else if Self::is_virtual_container_type_field(valued_struct_field) {
                StructViewerFieldPresentation::new(String::from("container_type"), StructViewerFieldEditorKind::ContainerTypeSelector)
            } else if Self::is_virtual_array_size_field(valued_struct_field) {
                StructViewerFieldPresentation::new(String::from("array_size"), StructViewerFieldEditorKind::ValueBox)
            } else if Self::is_live_value_field(valued_struct_field) && live_value_uses_code_viewer {
                StructViewerFieldPresentation::new(String::from("value"), StructViewerFieldEditorKind::CodeViewerButton)
            } else if Self::is_live_value_field(valued_struct_field) && live_value_uses_external_viewer {
                StructViewerFieldPresentation::new(String::from("value"), StructViewerFieldEditorKind::MemoryViewerButton)
            } else if Self::is_live_value_field(valued_struct_field) {
                StructViewerFieldPresentation::new(String::from("value"), StructViewerFieldEditorKind::ValueBox)
            } else {
                StructViewerFieldPresentation::new(valued_struct_field.get_name().to_string(), StructViewerFieldEditorKind::ValueBox)
            };

            field_presentations.insert(valued_struct_field.get_name().to_string(), field_presentation);
        }

        field_presentations
    }

    fn create_field_validation_data_type_refs(
        valued_struct: &ValuedStruct,
        _engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> HashMap<String, DataTypeRef> {
        let symbolic_struct_data_type_ref = Self::read_symbolic_struct_definition_reference(valued_struct);
        let mut field_validation_data_type_refs = HashMap::new();

        for valued_struct_field in valued_struct.get_fields() {
            let Some(data_value) = valued_struct_field.get_data_value() else {
                continue;
            };
            let validation_data_type_ref = if Self::is_virtual_array_size_field(valued_struct_field) {
                DataTypeRef::new(DataTypeU64::DATA_TYPE_ID)
            } else if Self::is_live_value_field(valued_struct_field) {
                symbolic_struct_data_type_ref
                    .clone()
                    .unwrap_or_else(|| data_value.get_data_type_ref().clone())
            } else {
                data_value.get_data_type_ref().clone()
            };

            field_validation_data_type_refs.insert(valued_struct_field.get_name().to_string(), validation_data_type_ref);
        }

        field_validation_data_type_refs
    }

    fn create_field_data_type_selections(valued_struct: &ValuedStruct) -> HashMap<String, DataTypeSelection> {
        let mut field_data_type_selections = HashMap::new();

        for valued_struct_field in valued_struct.get_fields() {
            if !Self::is_data_type_reference_field(valued_struct_field) {
                continue;
            }

            let selected_data_type_ref = Self::read_symbolic_field_definition_reference_from_field_set(valued_struct_field)
                .map(|symbolic_field_definition| symbolic_field_definition.get_data_type_ref().clone())
                .unwrap_or_else(|| DataTypeRef::new(DataTypeU8::DATA_TYPE_ID));

            field_data_type_selections.insert(valued_struct_field.get_name().to_string(), DataTypeSelection::new(selected_data_type_ref));
        }

        field_data_type_selections
    }

    fn is_data_type_reference_field(valued_struct_field: &ValuedStructField) -> bool {
        valued_struct_field.get_name() == ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE
    }

    fn is_virtual_container_type_field(valued_struct_field: &ValuedStructField) -> bool {
        valued_struct_field.get_name() == Self::VIRTUAL_FIELD_CONTAINER_TYPE
    }

    fn is_virtual_array_size_field(valued_struct_field: &ValuedStructField) -> bool {
        valued_struct_field.get_name() == Self::VIRTUAL_FIELD_ARRAY_SIZE
    }

    fn is_live_value_field(valued_struct_field: &ValuedStructField) -> bool {
        let field_name = valued_struct_field.get_name();

        field_name == ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE || field_name == ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE
    }

    fn is_live_value_external_viewer_editable(valued_struct: &ValuedStruct) -> bool {
        matches!(
            Self::read_symbolic_field_definition_reference(valued_struct).map(|symbolic_field_definition| symbolic_field_definition.get_container_type()),
            Some(ContainerType::Array | ContainerType::ArrayFixed(_))
        )
    }

    fn is_live_value_code_viewer_editable(valued_struct: &ValuedStruct) -> bool {
        Self::is_live_value_external_viewer_editable(valued_struct)
            && Self::read_symbolic_struct_definition_reference(valued_struct)
                .and_then(|data_type_ref| normalize_instruction_data_type_id(data_type_ref.get_data_type_id()))
                .map(|data_type_id| matches!(data_type_id.as_str(), "i_x86" | "i_x64"))
                .unwrap_or(false)
    }

    fn read_symbolic_struct_definition_reference(valued_struct: &ValuedStruct) -> Option<DataTypeRef> {
        Self::read_symbolic_field_definition_reference(valued_struct).map(|symbolic_field_definition| symbolic_field_definition.get_data_type_ref().clone())
    }

    fn read_symbolic_field_definition_reference(valued_struct: &ValuedStruct) -> Option<SymbolicFieldDefinition> {
        valued_struct
            .get_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
            .and_then(Self::read_symbolic_field_definition_reference_from_field_set)
    }

    pub fn read_symbolic_field_definition_reference_from_field_set(valued_struct_field: &ValuedStructField) -> Option<SymbolicFieldDefinition> {
        let data_value = valued_struct_field.get_data_value()?;
        let symbolic_field_definition_string = String::from_utf8(data_value.get_value_bytes().clone()).ok()?;
        let trimmed_symbolic_field_definition = symbolic_field_definition_string.trim();

        if trimmed_symbolic_field_definition.is_empty() {
            None
        } else {
            SymbolicFieldDefinition::from_str(trimmed_symbolic_field_definition).ok()
        }
    }

    pub fn read_utf8_field_text(valued_struct_field: &ValuedStructField) -> String {
        valued_struct_field
            .get_data_value()
            .and_then(|data_value| String::from_utf8(data_value.get_value_bytes().clone()).ok())
            .unwrap_or_default()
    }

    fn create_presented_struct(source_struct_under_view: &ValuedStruct) -> ValuedStruct {
        let mut presented_fields = Vec::new();

        for source_field in source_struct_under_view.get_fields() {
            presented_fields.push(source_field.clone());

            if !Self::is_data_type_reference_field(source_field) {
                continue;
            }

            let Some(symbolic_field_definition) = Self::read_symbolic_field_definition_reference_from_field_set(source_field) else {
                continue;
            };
            let container_mode = if matches!(
                symbolic_field_definition.get_container_type(),
                ContainerType::Array | ContainerType::ArrayFixed(_)
            ) {
                StructViewerContainerMode::Array
            } else {
                StructViewerContainerMode::Element
            };

            presented_fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(container_mode.label())
                    .to_named_valued_struct_field(Self::VIRTUAL_FIELD_CONTAINER_TYPE.to_string(), false),
            );

            if container_mode == StructViewerContainerMode::Array {
                presented_fields.push(
                    DataTypeU64::get_value_from_primitive(Self::fixed_array_length_from_symbolic_definition(&symbolic_field_definition))
                        .to_named_valued_struct_field(Self::VIRTUAL_FIELD_ARRAY_SIZE.to_string(), false),
                );
            }
        }

        ValuedStruct::new_anonymous(presented_fields)
    }

    fn build_symbolic_field_definition_edit(
        _symbolic_field_definition: &SymbolicFieldDefinition,
        data_type_ref: DataTypeRef,
        container_type: ContainerType,
    ) -> ValuedStructField {
        let symbolic_field_definition_value =
            DataTypeStringUtf8::get_value_from_primitive_string(&SymbolicFieldDefinition::new(data_type_ref, container_type).to_string());

        ValuedStructField::new(
            ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(),
            ValuedStructFieldData::Value(symbolic_field_definition_value),
            false,
        )
    }

    fn fixed_array_length_from_symbolic_definition(symbolic_field_definition: &SymbolicFieldDefinition) -> u64 {
        match symbolic_field_definition.get_container_type() {
            ContainerType::ArrayFixed(array_size) => array_size.max(1),
            ContainerType::Array => 1,
            _ => 1,
        }
    }

    fn read_container_mode_value(valued_struct_field: &ValuedStructField) -> Option<StructViewerContainerMode> {
        Self::read_utf8_field_text(valued_struct_field)
            .parse::<StructViewerContainerMode>()
            .ok()
    }

    fn read_u64_field_value(valued_struct_field: &ValuedStructField) -> Option<u64> {
        let data_value = valued_struct_field.get_data_value()?;
        let value_bytes = data_value.get_value_bytes();

        match value_bytes.len() {
            8 => <[u8; 8]>::try_from(value_bytes.as_slice())
                .ok()
                .map(u64::from_le_bytes),
            4 => <[u8; 4]>::try_from(value_bytes.as_slice())
                .ok()
                .map(|value_bytes| u32::from_le_bytes(value_bytes) as u64),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StructViewerViewData;
    use crate::views::struct_viewer::view_data::struct_viewer_container_mode::StructViewerContainerMode;
    use crate::views::struct_viewer::view_data::struct_viewer_field_presentation::StructViewerFieldEditorKind;
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::structures::{
        data_types::built_in_types::{
            string::utf8::data_type_string_utf8::DataTypeStringUtf8, u16::data_type_u16::DataTypeU16, u32::data_type_u32::DataTypeU32,
        },
        data_types::data_type_ref::DataTypeRef,
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat},
        projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress,
    };
    use squalr_engine_api::{
        commands::{
            privileged_command::PrivilegedCommand,
            privileged_command_response::PrivilegedCommandResponse,
            project::{list::project_list_response::ProjectListResponse, project_response::ProjectResponse},
            unprivileged_command::UnprivilegedCommand,
            unprivileged_command_response::{TypedUnprivilegedCommandResponse, UnprivilegedCommandResponse},
        },
        engine::{
            engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings, engine_binding_error::EngineBindingError,
            engine_event_envelope::EngineEventEnvelope, engine_execution_context::EngineExecutionContext,
        },
        structures::structs::valued_struct::ValuedStruct,
    };
    use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
    use std::sync::{Arc, RwLock};

    struct TestEngineBindings;

    impl EngineApiUnprivilegedBindings for TestEngineBindings {
        fn dispatch_privileged_command(
            &self,
            _engine_command: PrivilegedCommand,
            callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            callback(PrivilegedCommandResponse::Project(ProjectResponse::List {
                project_list_response: ProjectListResponse::default(),
            }));

            Ok(())
        }

        fn dispatch_unprivileged_command(
            &self,
            _engine_command: UnprivilegedCommand,
            _engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
            callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            callback(ProjectListResponse::default().to_engine_response());

            Ok(())
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEventEnvelope>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();

            Ok(event_receiver)
        }
    }

    fn create_test_engine_unprivileged_state() -> Arc<EngineUnprivilegedState> {
        EngineUnprivilegedState::new(Arc::new(RwLock::new(TestEngineBindings)))
    }

    #[test]
    fn create_field_edit_values_populates_utf8_fields() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("module.exe").to_named_valued_struct_field("module".to_string(), false),
        ]);

        let engine_unprivileged_state = create_test_engine_unprivileged_state();
        let field_validation_data_type_refs = StructViewerViewData::create_field_validation_data_type_refs(&valued_struct, &engine_unprivileged_state);
        let engine_unprivileged_state = create_test_engine_unprivileged_state();
        let field_edit_values = StructViewerViewData::create_field_edit_values(&valued_struct, &field_validation_data_type_refs, &engine_unprivileged_state);
        let module_edit_value = field_edit_values.get("module");

        assert!(module_edit_value.is_some());
        assert_eq!(
            module_edit_value
                .map(|anonymous_value_string| anonymous_value_string.get_anonymous_value_string())
                .unwrap_or_default(),
            "module.exe"
        );
    }

    #[test]
    fn create_field_presentations_maps_symbolic_struct_reference_to_data_type_editor() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string(DataTypeU32::DATA_TYPE_ID)
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
        ]);

        let field_presentations = StructViewerViewData::create_field_presentations(&valued_struct);
        let field_presentation = field_presentations
            .get(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
            .expect("Expected data-type field presentation.");

        assert_eq!(field_presentation.display_name(), "data_type");
        assert_eq!(field_presentation.editor_kind(), &StructViewerFieldEditorKind::DataTypeSelector);
    }

    #[test]
    fn create_field_presentations_maps_live_value_field_to_value_editor() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("1234")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE.to_string(), true),
        ]);

        let field_presentations = StructViewerViewData::create_field_presentations(&valued_struct);
        let field_presentation = field_presentations
            .get(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
            .expect("Expected live value field presentation.");

        assert_eq!(field_presentation.display_name(), "value");
        assert_eq!(field_presentation.editor_kind(), &StructViewerFieldEditorKind::ValueBox);
    }

    #[test]
    fn create_field_presentations_maps_array_live_value_field_to_memory_viewer_button() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("u8[16]")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string("1, 2, 3, 4")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE.to_string(), true),
        ]);

        let field_presentations = StructViewerViewData::create_field_presentations(&valued_struct);
        let field_presentation = field_presentations
            .get(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
            .expect("Expected live value field presentation.");

        assert_eq!(field_presentation.display_name(), "value");
        assert_eq!(field_presentation.editor_kind(), &StructViewerFieldEditorKind::MemoryViewerButton);
    }

    #[test]
    fn create_field_presentations_maps_x86_instruction_array_live_value_field_to_code_viewer_button() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("i_x86[16]")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string("90 90 C3")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE.to_string(), true),
        ]);

        let field_presentations = StructViewerViewData::create_field_presentations(&valued_struct);
        let field_presentation = field_presentations
            .get(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
            .expect("Expected live value field presentation.");

        assert_eq!(field_presentation.display_name(), "value");
        assert_eq!(field_presentation.editor_kind(), &StructViewerFieldEditorKind::CodeViewerButton);
    }

    #[test]
    fn create_field_edit_values_uses_numeric_format_for_live_value_field() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string(DataTypeU16::DATA_TYPE_ID)
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string("4660")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE.to_string(), true),
        ]);
        let engine_unprivileged_state = create_test_engine_unprivileged_state();
        let field_validation_data_type_refs = StructViewerViewData::create_field_validation_data_type_refs(&valued_struct, &engine_unprivileged_state);
        let engine_unprivileged_state = create_test_engine_unprivileged_state();
        let field_edit_values = StructViewerViewData::create_field_edit_values(&valued_struct, &field_validation_data_type_refs, &engine_unprivileged_state);

        assert_eq!(
            field_edit_values
                .get(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
                .map(AnonymousValueString::get_anonymous_value_string_format),
            Some(AnonymousValueStringFormat::Decimal)
        );
    }

    #[test]
    fn create_field_edit_values_preserves_array_container_for_live_value_field() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("u16[2]")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string("1, 2")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE.to_string(), true),
        ]);

        let engine_unprivileged_state = create_test_engine_unprivileged_state();
        let field_validation_data_type_refs = StructViewerViewData::create_field_validation_data_type_refs(&valued_struct, &engine_unprivileged_state);
        let field_edit_values = StructViewerViewData::create_field_edit_values(&valued_struct, &field_validation_data_type_refs, &engine_unprivileged_state);

        assert_eq!(
            field_edit_values
                .get(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
                .map(AnonymousValueString::get_container_type),
            Some(squalr_engine_api::structures::data_values::container_type::ContainerType::ArrayFixed(2))
        );
    }

    #[test]
    fn create_field_data_type_selections_reads_current_symbolic_struct_reference() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string(DataTypeU32::DATA_TYPE_ID)
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
        ]);

        let field_data_type_selections = StructViewerViewData::create_field_data_type_selections(&valued_struct);
        let field_data_type_selection = field_data_type_selections
            .get(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
            .expect("Expected data-type selection for the symbolic struct field.");

        assert_eq!(field_data_type_selection.visible_data_type(), &DataTypeRef::new(DataTypeU32::DATA_TYPE_ID));
    }

    #[test]
    fn create_presented_struct_adds_virtual_container_rows() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("u16[4]")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
        ]);

        let presented_struct = StructViewerViewData::create_presented_struct(&valued_struct);

        assert!(
            presented_struct
                .get_field(StructViewerViewData::VIRTUAL_FIELD_CONTAINER_TYPE)
                .is_some()
        );
        assert!(
            presented_struct
                .get_field(StructViewerViewData::VIRTUAL_FIELD_ARRAY_SIZE)
                .is_some()
        );
    }

    #[test]
    fn resolve_source_field_edit_maps_virtual_container_row_back_to_symbolic_definition() {
        let mut struct_viewer_view_data = StructViewerViewData::new();
        struct_viewer_view_data.source_struct_under_view = Arc::new(Some(ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("u16")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
        ])));

        let edited_virtual_field = DataTypeStringUtf8::get_value_from_primitive_string(StructViewerContainerMode::Array.label())
            .to_named_valued_struct_field(StructViewerViewData::VIRTUAL_FIELD_CONTAINER_TYPE.to_string(), false);
        let resolved_field = struct_viewer_view_data
            .resolve_source_field_edit(&edited_virtual_field)
            .expect("Expected virtual container edit to resolve.");

        assert_eq!(resolved_field.get_name(), ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE);
        assert_eq!(StructViewerViewData::read_utf8_field_text(&resolved_field), "u16[1]");
    }

    #[test]
    fn create_field_validation_data_type_refs_uses_symbolic_type_for_live_value_field() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string(DataTypeU16::DATA_TYPE_ID)
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string("4660")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE.to_string(), true),
        ]);

        let engine_unprivileged_state = create_test_engine_unprivileged_state();
        let field_validation_data_type_refs = StructViewerViewData::create_field_validation_data_type_refs(&valued_struct, &engine_unprivileged_state);

        assert_eq!(
            field_validation_data_type_refs.get(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE),
            Some(&DataTypeRef::new(DataTypeU16::DATA_TYPE_ID))
        );
    }
}
