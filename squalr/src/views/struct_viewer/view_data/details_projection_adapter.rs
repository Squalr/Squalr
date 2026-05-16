use crate::views::struct_viewer::view_data::struct_viewer_field_presentation::{StructViewerFieldEditorKind, StructViewerFieldPresentation};
use squalr_engine_api::structures::{
    data_types::{built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8, data_type_ref::DataTypeRef},
    data_values::{container_type::ContainerType, data_value::DataValue},
    details::{DetailsEdit, DetailsEditorHint, DetailsField, DetailsFieldId, DetailsFieldSource, DetailsProjection, DetailsTarget, DetailsValue},
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
        let mut rendered_field_names = HashSet::new();

        for (field_index, details_field) in details_projection.get_fields().iter().enumerate() {
            let rendered_field_name = Self::rendered_field_name(field_index, details_field, &mut rendered_field_names);
            let valued_struct_field = Self::valued_struct_field_from_details_field(engine_unprivileged_state, details_field, &rendered_field_name);

            rendered_field_ids.insert(rendered_field_name.clone(), details_field.get_id().clone());
            rendered_field_sources.insert(rendered_field_name.clone(), details_field.get_source().clone());
            field_presentations.insert(
                rendered_field_name.clone(),
                StructViewerFieldPresentation::new(details_field.get_label().to_string(), Self::editor_kind_from_details_field(details_field)),
            );

            if let Some(validation_data_type_ref) = Self::validation_data_type_ref_from_details_field(details_field) {
                field_validation_data_type_refs.insert(rendered_field_name, validation_data_type_ref);
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
                DetailsValue::Text(_) | DetailsValue::Boolean(_) | DetailsValue::UnsignedInteger(_) | DetailsValue::SignedInteger(_) | DetailsValue::Empty => {
                    Some(DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID))
                }
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
        match details_field.get_editor_hint() {
            DetailsEditorHint::Value | DetailsEditorHint::Address | DetailsEditorHint::Text | DetailsEditorHint::Boolean => {
                StructViewerFieldEditorKind::ValueBox
            }
            DetailsEditorHint::Code => StructViewerFieldEditorKind::CodeViewerButton,
            DetailsEditorHint::DataType => StructViewerFieldEditorKind::DataTypeSelector,
            DetailsEditorHint::ContainerType => StructViewerFieldEditorKind::ContainerTypeSelector,
            DetailsEditorHint::PointerOffsets => StructViewerFieldEditorKind::ProjectItemPointerOffsetsEditor,
            DetailsEditorHint::PointerSize => StructViewerFieldEditorKind::ProjectItemPointerSizeSelector,
            DetailsEditorHint::SymbolResolver | DetailsEditorHint::SymbolLayout => StructViewerFieldEditorKind::ValueBox,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DetailsProjectionAdapter;
    use crate::views::struct_viewer::view_data::struct_viewer_field_presentation::StructViewerFieldEditorKind;
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::structures::{
        data_types::{built_in_types::u32::data_type_u32::DataTypeU32, data_type_ref::DataTypeRef},
        data_values::container_type::ContainerType,
        details::{DetailsEditorHint, DetailsField, DetailsFieldId, DetailsFieldSource, DetailsProjection, DetailsTarget, DetailsValue},
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
    fn details_projection_adapter_uses_labels_without_routing_edits_by_label() {
        let target = DetailsTarget::new("project_item", "/Health");
        let field_id = DetailsFieldId::new("runtime.value");
        let details_projection = DetailsProjection::new(
            target.clone(),
            "Health",
            vec![DetailsField::new(
                field_id.clone(),
                "Current HP",
                DetailsValue::DataValue(DataTypeU32::get_value_from_primitive(100)),
                false,
                DetailsEditorHint::Value,
                Some(DataTypeRef::new(DataTypeU32::DATA_TYPE_ID)),
                ContainerType::None,
                DetailsFieldSource::ProjectItemRuntimeValue {
                    field_path: vec!["value".to_string()],
                },
            )],
        );
        let engine_unprivileged_state = create_test_engine_unprivileged_state();
        let adapter = DetailsProjectionAdapter::adapt_projection(&engine_unprivileged_state, &details_projection);
        let (valued_struct, adapter_state) = adapter.into_parts();
        let rendered_field = valued_struct
            .get_fields()
            .first()
            .expect("Expected details projection adapter to render a field.");
        let field_presentation = adapter_state
            .field_presentations
            .get(rendered_field.get_name())
            .expect("Expected details field presentation.");
        let details_edit = adapter_state
            .build_details_edit(rendered_field)
            .expect("Expected rendered field edit to route back to a details edit.");

        assert_eq!(field_presentation.display_name(), "Current HP");
        assert_eq!(field_presentation.editor_kind(), &StructViewerFieldEditorKind::ValueBox);
        assert_eq!(details_edit.get_target(), &target);
        assert_eq!(details_edit.get_field_id(), &field_id);
    }
}
