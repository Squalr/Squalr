use crate::structures::{
    data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
    data_values::data_value::DataValue,
    projects::project_items::{project_item::ProjectItem, project_item_type::ProjectItemType, project_item_type_ref::ProjectItemTypeRef},
    structs::valued_struct_field::ValuedStructFieldNode,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypeAddress {}

impl ProjectItemType for ProjectItemTypeAddress {
    fn get_project_item_type_id(&self) -> &str {
        &Self::PROJECT_ITEM_TYPE_ID
    }
}

impl ProjectItemTypeAddress {
    pub const PROJECT_ITEM_TYPE_ID: &str = "address";
    pub const PROPERTY_DESCRIPTION: &str = "description";
    pub const PROPERTY_FREEZE_VALUE: &str = "freeze_value";

    pub fn new_project_item(
        path: &Path,
        address: u64,
        description: &str,
        freeze_value: DataValue,
    ) -> ProjectItem {
        let directory_type = ProjectItemTypeRef::new(Self::PROJECT_ITEM_TYPE_ID.to_string());
        let mut project_item = ProjectItem::new(path.to_path_buf(), directory_type, false);

        Self::set_field_address(&mut project_item, address);
        Self::set_field_description(&mut project_item, description);
        Self::set_field_freeze_value(&mut project_item, freeze_value);

        project_item
    }

    pub fn get_field_description(project_item: &ProjectItem) -> String {
        if let Some(name_field) = project_item
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

    pub fn set_field_address(
        project_item: &mut ProjectItem,
        address: u64,
    ) {
        let description_address = DataTypeU64::get_value_from_primitive(address);
        let field_node = ValuedStructFieldNode::Value(description_address);

        project_item
            .get_properties_mut()
            .set_field_node(Self::PROPERTY_DESCRIPTION, field_node, false);
    }

    pub fn set_field_description(
        project_item: &mut ProjectItem,
        description: &str,
    ) {
        let description_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&description);
        let field_node = ValuedStructFieldNode::Value(description_data_value);

        project_item
            .get_properties_mut()
            .set_field_node(Self::PROPERTY_DESCRIPTION, field_node, false);
    }

    pub fn set_field_freeze_value(
        project_item: &mut ProjectItem,
        freeze_value: DataValue,
    ) {
        let field_node = ValuedStructFieldNode::Value(freeze_value);

        project_item
            .get_properties_mut()
            .set_field_node(Self::PROPERTY_DESCRIPTION, field_node, false);
    }
}
