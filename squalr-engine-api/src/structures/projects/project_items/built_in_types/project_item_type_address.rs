use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::registries::registry_context::RegistryContext;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use crate::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use crate::structures::{
    data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
    data_values::data_value::DataValue,
    projects::project_items::{project_item::ProjectItem, project_item_type::ProjectItemType, project_item_type_ref::ProjectItemTypeRef},
    structs::valued_struct_field::ValuedStructFieldData,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypeAddress {}

impl ProjectItemType for ProjectItemTypeAddress {
    fn get_project_item_type_id(&self) -> &str {
        &Self::PROJECT_ITEM_TYPE_ID
    }

    fn on_activated_changed(
        &self,
        _engine_bindings: &Arc<RwLock<dyn EngineApiPrivilegedBindings>>,
        _registry_context: &dyn RegistryContext,
        _project_item: &ProjectItemRef,
    ) {
        /*
        let address = ProjectItemTypeAddress::get_field_address(project_item);
        let module_name = ProjectItemTypeAddress::get_field_module(project_item);

        if let Some(symbolic_struct_ref) = ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(project_item) {
            if let Ok(symbol_registry) = registries.get_symbol_registry().read() {
                if let Some(symbolic_struct_definition) = symbol_registry.get(symbolic_struct_ref.get_symbolic_struct_namespace()) {
                    let freeze_list_registry = registries.get_freeze_list_registry();
                    let pointer = Pointer::new(address, vec![], module_name.clone());

                    if project_item.get_is_activated() {
                        let memory_read_request = MemoryReadRequest {
                            address,
                            module_name,
                            symbolic_struct_definition: symbolic_struct_definition.deref().clone(),
                        };

                        memory_read_request.send_privileged(engine_bindings, move |memory_read_response| {
                            let read_valued_struct_bytes = memory_read_response.valued_struct.get_bytes();

                            if let Ok(mut freeze_list_registry) = freeze_list_registry.write() {
                                freeze_list_registry.set_address_frozen(pointer, read_valued_struct_bytes);
                            }
                        });
                    } else {
                        if let Ok(mut freeze_list_registry) = freeze_list_registry.write() {
                            freeze_list_registry.set_address_unfrozen(&pointer);
                        }
                    }
                }
            }
        }*/
    }

    fn tick(
        &self,
        _engine_bindings: &dyn EngineApiPrivilegedBindings,
        _opened_process: &Option<OpenedProcessInfo>,
        _registry_context: &dyn RegistryContext,
        _project_item_ref: &ProjectItemRef,
    ) {
        /*
        let memory_read_request = MemoryReadRequest {
            address,
            module_name,
            symbolic_struct_definition: symbolic_struct_definition.deref().clone(),
        };
        memory_read_request.send_privileged(engine_bindings, move |memory_read_response| {
            let read_valued_struct_bytes = memory_read_response.valued_struct.get_bytes();

            if let Ok(mut freeze_list_registry) = freeze_list_registry.write() {
                freeze_list_registry.set_address_frozen(pointer, read_valued_struct_bytes);
            }
        });*/
    }
}

impl ProjectItemTypeAddress {
    pub const PROJECT_ITEM_TYPE_ID: &str = "address";
    pub const DEFAULT_PROJECT_ITEM_NAME: &str = "New Address";
    pub const PROPERTY_ADDRESS: &str = "address";
    pub const PROPERTY_MODULE: &str = "module";
    pub const PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE: &str = "symbolic_struct_definition_reference";
    pub const PROPERTY_FREEZE_DISPLAY_VALUE: &str = "freeze_data_value_interpreter";

    pub fn new_project_item(
        name: &str,
        address: u64,
        module: &str,
        description: &str,
        freeze_value: DataValue,
    ) -> ProjectItem {
        let project_item_type_ref = ProjectItemTypeRef::new(Self::PROJECT_ITEM_TYPE_ID.to_string());
        let project_item_name = if name.trim().is_empty() { Self::DEFAULT_PROJECT_ITEM_NAME } else { name };
        let mut project_item = ProjectItem::new(project_item_type_ref, project_item_name);

        project_item.set_field_description(description);
        Self::set_field_module(&mut project_item, module);
        Self::set_field_address(&mut project_item, address);
        // Default to unknown until project-item refresh logic reads live memory.
        Self::set_field_freeze_data_value_interpreter(&mut project_item, "");
        Self::set_field_symbolic_struct_definition_reference(&mut project_item, freeze_value.get_data_type_id());

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
                    let arr: [u8; 4] = bytes.try_into().unwrap_or([0u8; 4]);

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
        let field_data = ValuedStructFieldData::Value(description_address);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_ADDRESS, field_data, false);
    }

    pub fn get_field_module(project_item: &mut ProjectItem) -> String {
        Self::read_string_field(project_item, Self::PROPERTY_MODULE)
    }

    pub fn set_field_module(
        project_item: &mut ProjectItem,
        module: &str,
    ) {
        let module_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&module);
        let field_data = ValuedStructFieldData::Value(module_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_MODULE, field_data, false);
    }

    pub fn get_field_freeze_data_value_interpreter(project_item: &mut ProjectItem) -> String {
        Self::read_string_field(project_item, Self::PROPERTY_FREEZE_DISPLAY_VALUE)
    }

    pub fn set_field_freeze_data_value_interpreter(
        project_item: &mut ProjectItem,
        data_value_interpreter: &str,
    ) {
        let data_value_interpreter_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&data_value_interpreter);
        let field_data = ValuedStructFieldData::Value(data_value_interpreter_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_FREEZE_DISPLAY_VALUE, field_data, true);
    }

    pub fn get_field_symbolic_struct_definition_reference(project_item: &mut ProjectItem) -> Option<SymbolicStructRef> {
        let symbolic_struct_definition_reference = Self::read_string_field(project_item, Self::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE);

        if symbolic_struct_definition_reference.is_empty() {
            None
        } else {
            Some(SymbolicStructRef::new(symbolic_struct_definition_reference))
        }
    }

    pub fn set_field_symbolic_struct_definition_reference(
        project_item: &mut ProjectItem,
        symbolic_struct_definition: &str,
    ) {
        let symbolic_struct_definition_data_value = DataTypeStringUtf8::get_value_from_primitive_string(symbolic_struct_definition);
        let field_data = ValuedStructFieldData::Value(symbolic_struct_definition_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE, field_data, false);
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

#[cfg(test)]
mod tests {
    use super::ProjectItemTypeAddress;
    use crate::structures::data_types::built_in_types::{u8::data_type_u8::DataTypeU8, u32::data_type_u32::DataTypeU32};
    use crate::structures::structs::valued_struct_field::ValuedStructFieldData;

    #[test]
    fn new_project_item_uses_new_address_for_empty_name() {
        let project_item = ProjectItemTypeAddress::new_project_item("", 0x1234, "module", "", DataTypeU8::get_value_from_primitive(0));

        assert_eq!(project_item.get_field_name(), ProjectItemTypeAddress::DEFAULT_PROJECT_ITEM_NAME);
    }

    #[test]
    fn new_project_item_uses_supplied_name_when_non_empty() {
        let project_item = ProjectItemTypeAddress::new_project_item("Address Name", 0x1234, "module", "", DataTypeU8::get_value_from_primitive(0));

        assert_eq!(project_item.get_field_name(), "Address Name");
    }

    #[test]
    fn new_project_item_defaults_freeze_display_value_to_unknown() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "module", "", DataTypeU8::get_value_from_primitive(7));

        assert_eq!(ProjectItemTypeAddress::get_field_freeze_data_value_interpreter(&mut project_item), "");
    }

    #[test]
    fn get_field_address_reads_u32_bytes() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0, "module", "", DataTypeU8::get_value_from_primitive(7));
        let address_field_data = ValuedStructFieldData::Value(DataTypeU32::get_value_from_primitive(0x89ABCDEF));

        project_item
            .get_properties_mut()
            .set_field_data(ProjectItemTypeAddress::PROPERTY_ADDRESS, address_field_data, false);

        assert_eq!(ProjectItemTypeAddress::get_field_address(&mut project_item), 0x89ABCDEF);
    }
}
