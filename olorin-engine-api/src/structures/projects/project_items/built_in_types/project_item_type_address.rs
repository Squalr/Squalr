use crate::engine::engine_execution_context::EngineExecutionContext;
use crate::registries::project_item_types::project_item_type_registry::ProjectItemTypeRegistry;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::{
    data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
    data_values::data_value::DataValue,
    projects::project_items::{project_item::ProjectItem, project_item_type::ProjectItemType, project_item_type_ref::ProjectItemTypeRef},
    structs::valued_struct_field::ValuedStructFieldNode,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypeAddress {}

impl ProjectItemType for ProjectItemTypeAddress {
    fn get_project_item_type_id(&self) -> &str {
        &Self::PROJECT_ITEM_TYPE_ID
    }

    fn on_activated_changed(
        &self,
        project_item: &ProjectItem,
    ) {
        // JIRA: implement me
    }

    fn tick(
        &self,
        engine_execution_context: &Arc<EngineExecutionContext>,
        opened_process: &Option<OpenedProcessInfo>,
        project_item_type_registry: &ProjectItemTypeRegistry,
        project_item: &mut ProjectItem,
    ) {
        if let Some(opened_process) = opened_process {
            let address = ProjectItemTypeAddress::get_field_address(project_item);
            let value = ProjectItemTypeAddress::get_field_freeze_value(project_item);
            // let memory_write_request = MemoryWriteRequest { address, value };

            // memory_write_request.send(engine_execution_context);
        }
    }
}

impl ProjectItemTypeAddress {
    pub const PROJECT_ITEM_TYPE_ID: &str = "address";
    pub const PROPERTY_ADDRESS: &str = "address";
    pub const PROPERTY_MODULE: &str = "module";
    pub const PROPERTY_FREEZE_VALUE: &str = "freeze_value";

    pub fn new_project_item(
        path: &Path,
        address: u64,
        module: &str,
        description: &str,
        freeze_value: DataValue,
    ) -> ProjectItem {
        let directory_type = ProjectItemTypeRef::new(Self::PROJECT_ITEM_TYPE_ID.to_string());
        let mut project_item = ProjectItem::new(path.to_path_buf(), directory_type, false);

        project_item.set_field_description(description);
        Self::set_field_module(&mut project_item, module);
        Self::set_field_address(&mut project_item, address);
        Self::set_field_freeze_value(&mut project_item, freeze_value);

        project_item
    }

    pub fn get_field_address(project_item: &mut ProjectItem) -> u64 {
        if let Some(name_field) = project_item
            .get_properties()
            .get_fields()
            .iter()
            .find(|field| field.get_name() == Self::PROPERTY_ADDRESS)
        {
            let bytes = name_field.get_bytes();
            match bytes.len() {
                8 => return u64::from_le_bytes(bytes.try_into().unwrap_or([0u8; 8])),
                4 => {
                    let arr = [0u8; 4];

                    return u32::from_le_bytes(arr) as u64;
                }
                _ => {}
            }
        }

        0
    }

    pub fn set_field_address(
        project_item: &mut ProjectItem,
        address: u64,
    ) {
        let description_address = DataTypeU64::get_value_from_primitive(address);
        let field_node = ValuedStructFieldNode::Value(description_address);

        project_item
            .get_properties_mut()
            .set_field_node(Self::PROPERTY_ADDRESS, field_node, false);
    }

    pub fn get_field_module(project_item: &mut ProjectItem) -> String {
        if let Some(name_field) = project_item
            .get_properties()
            .get_fields()
            .iter()
            .find(|field| field.get_name() == Self::PROPERTY_MODULE)
        {
            name_field.get_display_string(true, 0)
        } else {
            String::new()
        }
    }

    pub fn set_field_module(
        project_item: &mut ProjectItem,
        module: &str,
    ) {
        let module_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&module);
        let field_node = ValuedStructFieldNode::Value(module_data_value);

        project_item
            .get_properties_mut()
            .set_field_node(Self::PROPERTY_MODULE, field_node, false);
    }

    pub fn get_field_freeze_value(project_item: &mut ProjectItem) -> String {
        if let Some(name_field) = project_item
            .get_properties()
            .get_fields()
            .iter()
            .find(|field| field.get_name() == Self::PROPERTY_FREEZE_VALUE)
        {
            name_field.get_display_string(true, 0)
        } else {
            String::new()
        }
    }

    pub fn set_field_freeze_value(
        project_item: &mut ProjectItem,
        freeze_value: DataValue,
    ) {
        let field_node = ValuedStructFieldNode::Value(freeze_value);

        project_item
            .get_properties_mut()
            .set_field_node(Self::PROPERTY_FREEZE_VALUE, field_node, false);
    }
}
