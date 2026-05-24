use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::views::struct_viewer::view_data::struct_viewer_field_presentation::{StructViewerFieldEditorKind, StructViewerFieldPresentation};
use squalr_engine_api::structures::{
    data_types::{built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8, data_type_ref::DataTypeRef},
    data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType, data_value::DataValue},
    details::{DetailsEdit, DetailsEditorHint, DetailsField, DetailsFieldId, DetailsFieldSource, DetailsProjection, DetailsTarget, DetailsValue},
    projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress,
    structs::{
        valued_struct::ValuedStruct,
        valued_struct_field::{ValuedStructField, ValuedStructFieldData},
    },
};
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Debug, Default)]
pub struct DetailsProjectionAdapterState {
    target: DetailsTarget,
    rendered_field_ids: HashMap<String, DetailsFieldId>,
    rendered_field_sources: HashMap<String, DetailsFieldSource>,
    field_presentations: HashMap<String, StructViewerFieldPresentation>,
    field_validation_data_type_refs: HashMap<String, DataTypeRef>,
    field_data_type_selections: HashMap<String, DataTypeSelection>,
    field_display_format_overrides: HashMap<String, AnonymousValueStringFormat>,
}

impl DetailsProjectionAdapterState {
    pub fn get_rendered_field_ids(&self) -> &HashMap<String, DetailsFieldId> {
        &self.rendered_field_ids
    }

    pub fn contains_rendered_field_name(
        &self,
        rendered_field_name: &str,
    ) -> bool {
        self.rendered_field_ids.contains_key(rendered_field_name)
    }

    pub fn apply_field_presentations(
        &self,
        field_presentations: &mut HashMap<String, StructViewerFieldPresentation>,
    ) {
        for (rendered_field_name, field_presentation) in &self.field_presentations {
            if field_presentations
                .get(rendered_field_name)
                .is_some_and(|existing_field_presentation| {
                    field_presentation.editor_kind() == &StructViewerFieldEditorKind::ValueBox
                        && existing_field_presentation.editor_kind() != &StructViewerFieldEditorKind::ValueBox
                })
            {
                continue;
            }

            field_presentations.insert(rendered_field_name.clone(), field_presentation.clone());
        }
    }

    pub fn apply_field_validation_data_type_refs(
        &self,
        field_validation_data_type_refs: &mut HashMap<String, DataTypeRef>,
    ) {
        for (rendered_field_name, data_type_ref) in &self.field_validation_data_type_refs {
            field_validation_data_type_refs.insert(rendered_field_name.clone(), data_type_ref.clone());
        }
    }

    pub fn apply_field_data_type_selections(
        &self,
        field_data_type_selections: &mut HashMap<String, DataTypeSelection>,
    ) {
        for (rendered_field_name, data_type_selection) in &self.field_data_type_selections {
            field_data_type_selections.insert(rendered_field_name.clone(), data_type_selection.clone());
        }
    }

    pub fn apply_field_display_format_overrides(
        &self,
        field_edit_values: &mut HashMap<String, squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString>,
    ) {
        for (rendered_field_name, display_format) in &self.field_display_format_overrides {
            if let Some(field_edit_value) = field_edit_values.get_mut(rendered_field_name) {
                field_edit_value.set_anonymous_value_string_format(*display_format);
            }
        }
    }

    pub fn build_details_edit(
        &self,
        edited_field: &ValuedStructField,
    ) -> Option<DetailsEdit> {
        let field_id = self.rendered_field_ids.get(edited_field.get_name())?.clone();
        let source = self
            .rendered_field_sources
            .get(edited_field.get_name())
            .cloned()
            .unwrap_or_default();
        let value = Self::details_value_from_valued_struct_field(edited_field);

        Some(DetailsEdit::new_with_source(self.target.clone(), field_id, source, value))
    }

    pub fn build_display_format_edit(
        &self,
        field_name: &str,
        display_format: AnonymousValueStringFormat,
    ) -> Option<DetailsEdit> {
        let field_id = self.rendered_field_ids.get(field_name)?.clone();
        let source = self
            .rendered_field_sources
            .get(field_name)
            .cloned()
            .unwrap_or_default();

        Some(DetailsEdit::new_with_source(
            self.target.clone(),
            field_id,
            source,
            DetailsValue::DisplayFormat(display_format),
        ))
    }

    fn details_value_from_valued_struct_field(valued_struct_field: &ValuedStructField) -> DetailsValue {
        match valued_struct_field.get_field_data() {
            ValuedStructFieldData::Value(data_value) => DetailsValue::DataValue(data_value.clone()),
            ValuedStructFieldData::NestedStruct(nested_struct) => DetailsValue::Text(nested_struct.get_display_string(false)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DetailsProjectionAdapter {
    valued_struct: ValuedStruct,
    state: DetailsProjectionAdapterState,
}

impl DetailsProjectionAdapter {
    pub fn adapt_projection(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        details_projection: &DetailsProjection,
    ) -> Self {
        let mut fields = Vec::new();
        let mut rendered_field_ids = HashMap::new();
        let mut rendered_field_sources = HashMap::new();
        let mut field_presentations = HashMap::new();
        let mut field_validation_data_type_refs = HashMap::new();
        let mut field_data_type_selections = HashMap::new();
        let mut field_display_format_overrides = HashMap::new();
        let mut rendered_field_names = HashSet::new();

        for (field_index, details_field) in details_projection.get_fields().iter().enumerate() {
            let rendered_field_name = Self::rendered_field_name(field_index, details_field, &mut rendered_field_names);
            let valued_struct_field = Self::valued_struct_field_from_details_field(engine_unprivileged_state, details_field, &rendered_field_name);
            let editor_kind = Self::editor_kind_from_details_field(details_field);

            rendered_field_ids.insert(rendered_field_name.clone(), details_field.get_id().clone());
            rendered_field_sources.insert(rendered_field_name.clone(), details_field.get_source().clone());
            field_presentations.insert(
                rendered_field_name.clone(),
                StructViewerFieldPresentation::new(details_field.get_label().to_string(), editor_kind.clone()),
            );

            if let Some(validation_data_type_ref) = Self::validation_data_type_ref_from_details_field(details_field) {
                field_validation_data_type_refs.insert(rendered_field_name.clone(), validation_data_type_ref);
            }

            if let Some(preferred_display_format) = details_field.get_preferred_display_format() {
                field_display_format_overrides.insert(rendered_field_name.clone(), preferred_display_format);
            }

            if matches!(
                editor_kind,
                StructViewerFieldEditorKind::DataTypeSelector
                    | StructViewerFieldEditorKind::SymbolResolverDataTypeSelector
                    | StructViewerFieldEditorKind::SymbolLayoutFieldDataTypeSelector
            ) && let Some(data_type_ref) = Self::selected_data_type_ref_from_details_field(details_field)
            {
                field_data_type_selections.insert(rendered_field_name.clone(), DataTypeSelection::new(data_type_ref));
            }

            fields.push(valued_struct_field);
        }

        Self {
            valued_struct: ValuedStruct::new_anonymous(fields),
            state: DetailsProjectionAdapterState {
                target: details_projection.get_target().clone(),
                rendered_field_ids,
                rendered_field_sources,
                field_presentations,
                field_validation_data_type_refs,
                field_data_type_selections,
                field_display_format_overrides,
            },
        }
    }

    pub fn into_parts(self) -> (ValuedStruct, DetailsProjectionAdapterState) {
        (self.valued_struct, self.state)
    }

    fn rendered_field_name(
        field_index: usize,
        details_field: &DetailsField,
        rendered_field_names: &mut HashSet<String>,
    ) -> String {
        let preferred_field_name = match details_field.get_source() {
            DetailsFieldSource::ProjectItemProperty { property_name } => Some(property_name.clone()),
            DetailsFieldSource::ProjectSymbolRuntimeValue { .. }
                if matches!(details_field.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_)) =>
            {
                Some(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE.to_string())
            }
            DetailsFieldSource::SymbolLayoutMetadata { metadata_name } if metadata_name == "type" => {
                Some(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string())
            }
            DetailsFieldSource::SymbolResolverMetadata { metadata_name } => Some(metadata_name.clone()),
            _ => details_field
                .get_id()
                .get_field_id()
                .strip_prefix("property.")
                .map(str::to_string),
        }
        .filter(|field_name| !field_name.trim().is_empty());

        if let Some(preferred_field_name) = preferred_field_name {
            if rendered_field_names.insert(preferred_field_name.clone()) {
                return preferred_field_name;
            }
        }

        let mut candidate_field_name = format!("__details_field_{}", field_index);
        let mut collision_index = 0_usize;
        while !rendered_field_names.insert(candidate_field_name.clone()) {
            collision_index = collision_index.saturating_add(1);
            candidate_field_name = format!("__details_field_{}_{}", field_index, collision_index);
        }

        candidate_field_name
    }

    fn valued_struct_field_from_details_field(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        details_field: &DetailsField,
        rendered_field_name: &str,
    ) -> ValuedStructField {
        let data_value = Self::data_value_from_details_value(engine_unprivileged_state, details_field)
            .unwrap_or_else(|| DataTypeStringUtf8::get_value_from_primitive_string(""));

        data_value.to_named_valued_struct_field(rendered_field_name.to_string(), details_field.get_is_read_only())
    }

    fn data_value_from_details_value(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        details_field: &DetailsField,
    ) -> Option<DataValue> {
        match details_field.get_value() {
            DetailsValue::Empty => Some(DataTypeStringUtf8::get_value_from_primitive_string("")),
            DetailsValue::AnonymousValue(anonymous_value_string) => details_field
                .get_validation_data_type_ref()
                .and_then(|validation_data_type_ref| {
                    engine_unprivileged_state
                        .deanonymize_value_string(validation_data_type_ref, anonymous_value_string)
                        .ok()
                })
                .or_else(|| {
                    Some(DataTypeStringUtf8::get_value_from_primitive_string(
                        anonymous_value_string.get_anonymous_value_string(),
                    ))
                }),
            DetailsValue::DataValue(data_value) => Some(data_value.clone()),
            DetailsValue::DisplayFormat(display_format) => Some(DataTypeStringUtf8::get_value_from_primitive_string(&display_format.to_string())),
            DetailsValue::Text(text) => Some(DataTypeStringUtf8::get_value_from_primitive_string(text)),
            DetailsValue::Boolean(value) => Some(DataTypeStringUtf8::get_value_from_primitive_string(&value.to_string())),
            DetailsValue::UnsignedInteger(value) => Some(DataTypeStringUtf8::get_value_from_primitive_string(&value.to_string())),
            DetailsValue::SignedInteger(value) => Some(DataTypeStringUtf8::get_value_from_primitive_string(&value.to_string())),
        }
    }

    fn validation_data_type_ref_from_details_field(details_field: &DetailsField) -> Option<DataTypeRef> {
        details_field
            .get_validation_data_type_ref()
            .cloned()
            .or_else(|| match details_field.get_value() {
                DetailsValue::DataValue(data_value) => Some(data_value.get_data_type_ref().clone()),
                DetailsValue::Text(_)
                | DetailsValue::DisplayFormat(_)
                | DetailsValue::Boolean(_)
                | DetailsValue::UnsignedInteger(_)
                | DetailsValue::SignedInteger(_)
                | DetailsValue::Empty => Some(DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)),
                DetailsValue::AnonymousValue(anonymous_value_string) => {
                    if anonymous_value_string.get_container_type() == ContainerType::None {
                        Some(DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID))
                    } else {
                        None
                    }
                }
            })
    }

    fn editor_kind_from_details_field(details_field: &DetailsField) -> StructViewerFieldEditorKind {
        if let DetailsFieldSource::SymbolLayoutMetadata { metadata_name } = details_field.get_source() {
            match metadata_name.as_str() {
                "layout.kind" => return StructViewerFieldEditorKind::SymbolLayoutKindSelector,
                "field.element_type" => return StructViewerFieldEditorKind::SymbolLayoutFieldElementTypeSelector,
                "field.data_type" => return StructViewerFieldEditorKind::SymbolLayoutFieldDataTypeSelector,
                "field.symbol_layout" => return StructViewerFieldEditorKind::SymbolLayoutFieldSymbolLayoutSelector,
                "field.count_resolver" | "field.display_count_resolver" | "field.active_when_resolver" | "field.offset_resolver" => {
                    return StructViewerFieldEditorKind::SymbolLayoutFieldResolverSelector;
                }
                "field.container_kind" => return StructViewerFieldEditorKind::SymbolLayoutFieldContainerKindSelector,
                "field.pointer_size" => return StructViewerFieldEditorKind::SymbolLayoutFieldPointerSizeSelector,
                _ => {}
            }
        }

        if let DetailsFieldSource::SymbolResolverMetadata { metadata_name } = details_field.get_source() {
            match metadata_name.as_str() {
                "node.kind" => return StructViewerFieldEditorKind::SymbolResolverNodeKindSelector,
                "operator" => return StructViewerFieldEditorKind::SymbolResolverOperatorSelector,
                "data_type" => return StructViewerFieldEditorKind::SymbolResolverDataTypeSelector,
                _ => {}
            }
        }

        match details_field.get_editor_hint() {
            DetailsEditorHint::Value | DetailsEditorHint::Address | DetailsEditorHint::Text | DetailsEditorHint::Boolean => {
                StructViewerFieldEditorKind::ValueBox
            }
            DetailsEditorHint::DataType if details_field.get_is_read_only() => StructViewerFieldEditorKind::ValueBox,
            DetailsEditorHint::DataType => StructViewerFieldEditorKind::DataTypeSelector,
            DetailsEditorHint::PointerOffsets => StructViewerFieldEditorKind::ProjectItemPointerOffsetsEditor,
            DetailsEditorHint::PointerSize => StructViewerFieldEditorKind::ProjectItemPointerSizeSelector,
        }
    }

    fn selected_data_type_ref_from_details_field(details_field: &DetailsField) -> Option<DataTypeRef> {
        match details_field.get_value() {
            DetailsValue::Text(text) => Some(DataTypeRef::new(text.trim())),
            DetailsValue::DataValue(data_value) => String::from_utf8(data_value.get_value_bytes().clone())
                .ok()
                .map(|text| DataTypeRef::new(text.trim())),
            DetailsValue::AnonymousValue(anonymous_value_string) => Some(DataTypeRef::new(anonymous_value_string.get_anonymous_value_string().trim())),
            _ => details_field.get_validation_data_type_ref().cloned(),
        }
        .filter(|data_type_ref| !data_type_ref.get_data_type_id().trim().is_empty())
    }
}
