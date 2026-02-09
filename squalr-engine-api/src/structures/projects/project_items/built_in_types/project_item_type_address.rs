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
        let mut project_item = ProjectItem::new(project_item_type_ref, name);

        project_item.set_field_description(description);
        Self::set_field_module(&mut project_item, module);
        Self::set_field_address(&mut project_item, address);
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
        let field_data = ValuedStructFieldData::Value(description_address);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_ADDRESS, field_data, false);
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
        let field_data = ValuedStructFieldData::Value(module_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_MODULE, field_data, false);
    }

    pub fn get_field_freeze_data_value_interpreter(project_item: &mut ProjectItem) -> String {
        if let Some(name_field) = project_item
            .get_properties()
            .get_fields()
            .iter()
            .find(|field| field.get_name() == Self::PROPERTY_FREEZE_DISPLAY_VALUE)
        {
            name_field.get_display_string(true, 0)
        } else {
            String::new()
        }
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
        if let Some(name_field) = project_item
            .get_properties()
            .get_fields()
            .iter()
            .find(|field| field.get_name() == Self::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
        {
            Some(SymbolicStructRef::new(name_field.get_display_string(true, 0)))
        } else {
            None
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
}
