use crate::registries::project_item_types::project_item_type_registry::ProjectItemTypeRegistry;
use crate::structures::{
    data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8,
    projects::project_items::project_item_type_ref::ProjectItemTypeRef,
    structs::{valued_struct::ValuedStruct, valued_struct_field::ValuedStructFieldNode},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a unique reference to a project item in an opened project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItem {
    /// The unique path to this project item.
    path: PathBuf,

    // The type of this project item.
    item_type: ProjectItemTypeRef,

    /// The container for all properties on this project item.
    properties: ValuedStruct,

    /// A value indicating whether this item has been activated / enabled.
    is_activated: bool,

    /// The child project items underneath this project item.
    children: Vec<ProjectItem>,

    /// A value indicating whether this project item accepts children.
    is_container: bool,

    /// A value indicating whether this project item has unsaved changes.
    has_unsaved_changes: bool,

    #[serde(skip)]
    current_display_value: String,
}

impl ProjectItem {
    pub const PROPERTY_NAME: &str = "name";
    pub const PROPERTY_DESCRIPTION: &str = "description";

    pub fn new(
        path: PathBuf,
        item_type: ProjectItemTypeRef,
        is_container: bool,
    ) -> Self {
        let mut project_item = Self {
            path,
            item_type,
            properties: ValuedStruct::new_anonymous(vec![]),
            is_activated: false,
            children: vec![],
            is_container,
            has_unsaved_changes: true,
            current_display_value: String::new(),
        };

        let name = project_item
            .path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("")
            .to_owned();
        project_item.set_field_name(&name);

        project_item
    }

    pub fn get_file_or_directory_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("")
            .to_string()
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
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
        project_item_type_registry: &ProjectItemTypeRegistry,
        is_activated: bool,
    ) {
        self.is_activated = is_activated;

        if let Some(project_item_type) = project_item_type_registry.get(self.item_type.get_project_item_type_id()) {
            project_item_type.on_activated_changed(project_item_type_registry, self);
        }
    }

    pub fn get_is_container(&self) -> bool {
        self.is_container
    }

    pub fn get_children(&self) -> &Vec<ProjectItem> {
        debug_assert!(self.is_container);

        &self.children
    }

    pub fn append_child(
        &mut self,
        new_child: ProjectItem,
    ) {
        debug_assert!(self.is_container);

        self.children.push(new_child);
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<ProjectItem> {
        debug_assert!(self.is_container);

        &mut self.children
    }

    pub fn get_display_string(&self) -> &str {
        &self.current_display_value
    }

    pub fn get_field_name(&self) -> String {
        if let Some(name_field) = self
            .get_properties()
            .get_fields()
            .iter()
            .find(|field| field.get_name() == Self::PROPERTY_NAME)
        {
            name_field.get_display_string(true, 0)
        } else {
            String::new()
        }
    }

    pub fn set_field_name(
        &mut self,
        name: &str,
    ) {
        let name_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&name);
        let field_node = ValuedStructFieldNode::Value(name_data_value);

        self.get_properties_mut()
            .set_field_node(Self::PROPERTY_NAME, field_node, false);
    }

    pub fn get_field_description(&self) -> String {
        if let Some(name_field) = self
            .get_properties()
            .get_fields()
            .iter()
            .find(|field| field.get_name() == Self::PROPERTY_DESCRIPTION)
        {
            name_field.get_display_string(true, 0)
        } else {
            String::new()
        }
    }

    pub fn set_field_description(
        &mut self,
        description: &str,
    ) {
        let description_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&description);
        let field_node = ValuedStructFieldNode::Value(description_data_value);

        self.get_properties_mut()
            .set_field_node(Self::PROPERTY_DESCRIPTION, field_node, false);
    }
}
