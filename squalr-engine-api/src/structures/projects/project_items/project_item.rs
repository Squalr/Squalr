use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::registries::registry_context::RegistryContext;
use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use crate::structures::{
    data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8,
    projects::project_items::project_item_type_ref::ProjectItemTypeRef,
    structs::{valued_struct::ValuedStruct, valued_struct_field::ValuedStructFieldData},
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Represents a unique reference to a project item in an opened project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItem {
    // The type of this project item.
    item_type: ProjectItemTypeRef,

    /// The container for all properties on this project item.
    properties: ValuedStruct,

    /// A value indicating whether this item has been activated / enabled.
    #[serde(skip)]
    is_activated: bool,

    /// A value indicating whether this project item has unsaved changes.
    #[serde(skip)]
    has_unsaved_changes: bool,

    #[serde(skip)]
    current_data_value_interpreter: String,
}

impl ProjectItem {
    pub const PROPERTY_NAME: &str = "name";
    pub const PROPERTY_ICON_ID: &str = "icon_id";
    pub const PROPERTY_DESCRIPTION: &str = "description";

    pub fn new(
        item_type: ProjectItemTypeRef,
        name: &str,
    ) -> Self {
        let mut project_item = Self {
            item_type,
            properties: ValuedStruct::new_anonymous(vec![]),
            is_activated: false,
            has_unsaved_changes: true,
            current_data_value_interpreter: String::new(),
        };

        project_item.set_field_name(name);

        project_item
    }

    pub fn get_item_type(&self) -> &ProjectItemTypeRef {
        &self.item_type
    }

    pub fn get_properties(&self) -> &ValuedStruct {
        &self.properties
    }

    pub fn get_properties_mut(&mut self) -> &mut ValuedStruct {
        &mut self.properties
    }

    pub fn get_has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    pub fn set_has_unsaved_changes(
        &mut self,
        has_unsaved_changes: bool,
    ) {
        self.has_unsaved_changes = has_unsaved_changes;
    }

    pub fn get_is_activated(&self) -> bool {
        self.is_activated
    }

    pub fn toggle_activated(&mut self) {
        self.is_activated = !self.is_activated
    }

    pub fn set_activated(
        &mut self,
        project_item_ref: &ProjectItemRef,
        engine_bindings: &Arc<RwLock<dyn EngineApiPrivilegedBindings>>,
        registry_context: &dyn RegistryContext,
        is_activated: bool,
    ) {
        self.is_activated = is_activated;

        if let Ok(project_item_type_registry) = registry_context.get_project_item_type_registry().read() {
            if let Some(project_item_type) = project_item_type_registry.get(self.item_type.get_project_item_type_id()) {
                project_item_type.on_activated_changed(engine_bindings, registry_context, project_item_ref);
            }
        }
    }

    pub fn get_display_string(&self) -> &str {
        &self.current_data_value_interpreter
    }

    pub fn get_field_name(&self) -> String {
        Self::read_string_field(self, Self::PROPERTY_NAME)
    }

    pub fn set_field_name(
        &mut self,
        name: &str,
    ) {
        let name_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&name);
        let field_data = ValuedStructFieldData::Value(name_data_value);

        self.get_properties_mut()
            .set_field_data(Self::PROPERTY_NAME, field_data, false);
    }

    pub fn get_field_icon_id(&self) -> String {
        Self::read_string_field(self, Self::PROPERTY_ICON_ID)
    }

    pub fn set_field_icon_id(
        &mut self,
        icon_id: &str,
    ) {
        let icon_id_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&icon_id);
        let field_data = ValuedStructFieldData::Value(icon_id_data_value);

        self.get_properties_mut()
            .set_field_data(Self::PROPERTY_ICON_ID, field_data, false);
    }

    pub fn get_field_description(&self) -> String {
        Self::read_string_field(self, Self::PROPERTY_DESCRIPTION)
    }

    pub fn set_field_description(
        &mut self,
        description: &str,
    ) {
        let description_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&description);
        let field_data = ValuedStructFieldData::Value(description_data_value);

        self.get_properties_mut()
            .set_field_data(Self::PROPERTY_DESCRIPTION, field_data, false);
    }

    fn read_string_field(
        project_item: &ProjectItem,
        field_name: &str,
    ) -> String {
        let data_value = match project_item
            .get_properties()
            .get_field(field_name)
            .and_then(|field| field.get_data_value())
        {
            Some(data_value) => data_value,
            None => return String::new(),
        };

        String::from_utf8(data_value.get_value_bytes().clone()).unwrap_or_default()
    }
}
