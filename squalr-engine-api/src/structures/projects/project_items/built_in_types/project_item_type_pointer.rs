use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::registries::registry_context::RegistryContext;
use crate::structures::memory::pointer::Pointer;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use crate::structures::projects::project_items::project_item_type::ProjectItemType;
use crate::structures::{
    data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    projects::project_items::{project_item::ProjectItem, project_item_type_ref::ProjectItemTypeRef},
    structs::symbolic_struct_ref::SymbolicStructRef,
    structs::valued_struct_field::ValuedStructFieldData,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypePointer {}

impl ProjectItemTypePointer {
    pub const PROJECT_ITEM_TYPE_ID: &str = "pointer";
    pub const DEFAULT_PROJECT_ITEM_NAME: &str = "New Pointer";
    pub const PROPERTY_ADDRESS: &str = "address";
    pub const PROPERTY_MODULE: &str = "module";
    pub const PROPERTY_POINTER_OFFSETS: &str = "pointer_offsets";
    pub const PROPERTY_POINTER_SIZE: &str = "pointer_size";
    pub const PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE: &str = "symbolic_struct_definition_reference";
    pub const PROPERTY_FREEZE_DISPLAY_VALUE: &str = "freeze_data_value_interpreter";

    pub fn new_project_item(
        name: &str,
        pointer: &Pointer,
        description: &str,
        data_type_id: &str,
    ) -> ProjectItem {
        let project_item_type_ref = ProjectItemTypeRef::new(Self::PROJECT_ITEM_TYPE_ID.to_string());
        let project_item_name = if name.trim().is_empty() { Self::DEFAULT_PROJECT_ITEM_NAME } else { name };
        let mut project_item = ProjectItem::new(project_item_type_ref, project_item_name);

        project_item.set_field_description(description);
        Self::set_field_module(&mut project_item, pointer.get_module_name());
        Self::set_field_address(&mut project_item, pointer.get_address());
        Self::set_field_pointer_offsets(&mut project_item, pointer.get_offsets());
        Self::set_field_pointer_size(&mut project_item, pointer.get_pointer_size());
        Self::set_field_freeze_data_value_interpreter(&mut project_item, "");
        Self::set_field_symbolic_struct_definition_reference(&mut project_item, data_type_id);

        project_item
    }

    pub fn get_field_address(project_item: &ProjectItem) -> u64 {
        let data_value = match project_item
            .get_properties()
            .get_field(Self::PROPERTY_ADDRESS)
            .and_then(|field| field.get_data_value())
        {
            Some(data_value) => data_value,
            None => return 0,
        };
        let value_bytes = data_value.get_value_bytes();

        match value_bytes.len() {
            8 => {
                let Ok(address_bytes) = <[u8; 8]>::try_from(value_bytes.as_slice()) else {
                    return 0;
                };

                u64::from_le_bytes(address_bytes)
            }
            4 => {
                let Ok(address_bytes) = <[u8; 4]>::try_from(value_bytes.as_slice()) else {
                    return 0;
                };

                u32::from_le_bytes(address_bytes) as u64
            }
            _ => 0,
        }
    }

    pub fn set_field_address(
        project_item: &mut ProjectItem,
        address: u64,
    ) {
        let address_data_value = DataTypeU64::get_value_from_primitive(address);
        let field_data = ValuedStructFieldData::Value(address_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_ADDRESS, field_data, false);
    }

    pub fn get_field_module(project_item: &ProjectItem) -> String {
        Self::read_string_field(project_item, Self::PROPERTY_MODULE)
    }

    pub fn set_field_module(
        project_item: &mut ProjectItem,
        module: &str,
    ) {
        let module_data_value = DataTypeStringUtf8::get_value_from_primitive_string(module);
        let field_data = ValuedStructFieldData::Value(module_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_MODULE, field_data, false);
    }

    pub fn get_field_pointer_offsets(project_item: &ProjectItem) -> Vec<i64> {
        let serialized_offsets = Self::read_string_field(project_item, Self::PROPERTY_POINTER_OFFSETS);

        if serialized_offsets.trim().is_empty() {
            return Vec::new();
        }

        match serde_json::from_str::<Vec<i64>>(&serialized_offsets) {
            Ok(pointer_offsets) => pointer_offsets,
            Err(error) => {
                log::warn!("Failed to deserialize pointer offsets for project item: {}", error);
                Vec::new()
            }
        }
    }

    pub fn set_field_pointer_offsets(
        project_item: &mut ProjectItem,
        pointer_offsets: &[i64],
    ) {
        let serialized_pointer_offsets = match serde_json::to_string(pointer_offsets) {
            Ok(serialized_pointer_offsets) => serialized_pointer_offsets,
            Err(error) => {
                log::warn!("Failed to serialize pointer offsets for project item: {}", error);
                String::from("[]")
            }
        };
        let pointer_offsets_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&serialized_pointer_offsets);
        let field_data = ValuedStructFieldData::Value(pointer_offsets_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_POINTER_OFFSETS, field_data, false);
    }

    pub fn get_field_pointer_size(project_item: &ProjectItem) -> PointerScanPointerSize {
        let pointer_size = Self::read_string_field(project_item, Self::PROPERTY_POINTER_SIZE);

        PointerScanPointerSize::from_str(&pointer_size).unwrap_or_default()
    }

    pub fn set_field_pointer_size(
        project_item: &mut ProjectItem,
        pointer_size: PointerScanPointerSize,
    ) {
        let pointer_size_data_value = DataTypeStringUtf8::get_value_from_primitive_string(&pointer_size.to_string());
        let field_data = ValuedStructFieldData::Value(pointer_size_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_POINTER_SIZE, field_data, false);
    }

    pub fn get_field_pointer(project_item: &ProjectItem) -> Pointer {
        Pointer::new_with_size(
            Self::get_field_address(project_item),
            Self::get_field_pointer_offsets(project_item),
            Self::get_field_module(project_item),
            Self::get_field_pointer_size(project_item),
        )
    }

    pub fn get_field_freeze_data_value_interpreter(project_item: &ProjectItem) -> String {
        Self::read_string_field(project_item, Self::PROPERTY_FREEZE_DISPLAY_VALUE)
    }

    pub fn set_field_freeze_data_value_interpreter(
        project_item: &mut ProjectItem,
        freeze_data_value_interpreter: &str,
    ) {
        let freeze_data_value_interpreter_data_value = DataTypeStringUtf8::get_value_from_primitive_string(freeze_data_value_interpreter);
        let field_data = ValuedStructFieldData::Value(freeze_data_value_interpreter_data_value);

        project_item
            .get_properties_mut()
            .set_field_data(Self::PROPERTY_FREEZE_DISPLAY_VALUE, field_data, true);
    }

    pub fn get_field_symbolic_struct_definition_reference(project_item: &ProjectItem) -> Option<SymbolicStructRef> {
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

impl ProjectItemType for ProjectItemTypePointer {
    fn get_project_item_type_id(&self) -> &str {
        &Self::PROJECT_ITEM_TYPE_ID
    }

    fn on_activated_changed(
        &self,
        _engine_bindings: &Arc<RwLock<dyn EngineApiPrivilegedBindings>>,
        _registry_context: &dyn RegistryContext,
        _project_item: &ProjectItemRef,
    ) {
        // JIRA: Implement.
    }

    fn tick(
        &self,
        _engine_bindings: &dyn EngineApiPrivilegedBindings,
        _opened_process: &Option<OpenedProcessInfo>,
        _registry_context: &dyn RegistryContext,
        _project_item: &ProjectItemRef,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemTypePointer;
    use crate::structures::memory::pointer::Pointer;
    use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

    #[test]
    fn new_project_item_uses_new_pointer_for_empty_name() {
        let pointer = Pointer::new_with_size(0x10, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer64);
        let project_item = ProjectItemTypePointer::new_project_item("", &pointer, "", "u8");

        assert_eq!(project_item.get_field_name(), ProjectItemTypePointer::DEFAULT_PROJECT_ITEM_NAME);
    }

    #[test]
    fn new_project_item_uses_supplied_name_when_non_empty() {
        let pointer = Pointer::new_with_size(0x10, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer64);
        let project_item = ProjectItemTypePointer::new_project_item("Pointer Name", &pointer, "", "u8");

        assert_eq!(project_item.get_field_name(), "Pointer Name");
    }

    #[test]
    fn new_project_item_persists_pointer_chain_and_data_type() {
        let pointer = Pointer::new_with_size(0x10, vec![0x20, -0x10], "game.exe".to_string(), PointerScanPointerSize::Pointer32);
        let project_item = ProjectItemTypePointer::new_project_item("Pointer Name", &pointer, "desc", "u16");
        let persisted_pointer = ProjectItemTypePointer::get_field_pointer(&project_item);
        let symbolic_struct_reference =
            ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(&project_item).expect("Expected symbolic struct reference to be persisted.");

        assert_eq!(persisted_pointer, pointer);
        assert_eq!(symbolic_struct_reference.get_symbolic_struct_namespace(), "u16");
        assert_eq!(ProjectItemTypePointer::get_field_freeze_data_value_interpreter(&project_item), "");
    }
}
