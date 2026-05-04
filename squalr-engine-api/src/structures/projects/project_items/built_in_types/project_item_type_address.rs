use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::registries::registry_context::RegistryContext;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::built_in_types::project_item_type_address_target::ProjectItemAddressTarget;
use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use crate::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use crate::structures::{
    data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8,
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
    pub const PROPERTY_TARGET: &str = "target_data";
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
        Self::set_address_target(&mut project_item, ProjectItemAddressTarget::new_address(address, module.to_string()));
        // Default to unknown until project-item refresh logic reads live memory.
        Self::set_field_freeze_data_value_interpreter(&mut project_item, "");
        Self::set_field_symbolic_struct_definition_reference(&mut project_item, freeze_value.get_data_type_id());

        project_item
    }

    pub fn get_field_address(project_item: &mut ProjectItem) -> u64 {
        Self::get_address_target(project_item)
            .get_root_offset()
            .and_then(|root_offset| u64::try_from(root_offset).ok())
            .unwrap_or(0)
    }

    pub fn set_field_address(
        project_item: &mut ProjectItem,
        address: u64,
    ) {
        let mut address_target = Self::get_address_target(project_item);
        let mut pointer_offsets = address_target.get_pointer_offsets().to_vec();

        if let Some(first_pointer_offset) = pointer_offsets.first_mut() {
            *first_pointer_offset = crate::structures::memory::pointer_chain_segment::PointerChainSegment::new_offset(address as i64);
        }

        address_target.set_pointer_offsets(pointer_offsets);
        Self::set_address_target(project_item, address_target);
    }

    pub fn get_field_module(project_item: &mut ProjectItem) -> String {
        Self::get_address_target(project_item)
            .get_module_name()
            .to_string()
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

        let mut address_target = Self::get_address_target(project_item);
        address_target.set_module_name(module.to_string());
        Self::set_address_target(project_item, address_target);
    }

    pub fn get_address_target(project_item: &mut ProjectItem) -> ProjectItemAddressTarget {
        let serialized_address_target = Self::read_string_field(project_item, Self::PROPERTY_TARGET);

        if let Ok(address_target) = serde_json::from_str::<ProjectItemAddressTarget>(&serialized_address_target) {
            return address_target;
        }

        ProjectItemAddressTarget::new_address(0, String::new())
    }

    pub fn set_address_target(
        project_item: &mut ProjectItem,
        address_target: ProjectItemAddressTarget,
    ) {
        Self::set_module_property(project_item, address_target.get_module_name());
        project_item
            .get_properties_mut()
            .remove_field(Self::PROPERTY_ADDRESS);

        let serialized_address_target = match serde_json::to_string(&address_target) {
            Ok(serialized_address_target) => serialized_address_target,
            Err(error) => {
                log::error!("Failed to serialize project address target: {}", error);
                return;
            }
        };
        let target_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&serialized_address_target);
        let field_data = ValuedStructFieldData::Value(target_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_TARGET, field_data, false);
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

    fn set_module_property(
        project_item: &mut ProjectItem,
        module: &str,
    ) {
        let module_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&module);
        let field_data = ValuedStructFieldData::Value(module_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_MODULE, field_data, false);
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemTypeAddress;
    use crate::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use crate::structures::memory::pointer_chain_segment::PointerChainSegment;

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
    fn new_project_item_stores_address_as_first_chain_segment() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0x579C, "winmine.exe", "", DataTypeU8::get_value_from_primitive(7));
        let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

        assert_eq!(ProjectItemTypeAddress::get_field_address(&mut project_item), 0x579C);
        assert_eq!(address_target.get_module_name(), "winmine.exe");
        assert_eq!(address_target.get_pointer_offsets(), &[PointerChainSegment::Offset(0x579C)]);
    }
}
