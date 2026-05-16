use crate::structures::{
    data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8,
    details::{DetailsFieldSource, DetailsValue},
    memory::pointer_chain_segment::PointerChainSegment,
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    projects::project_items::{
        built_in_types::{
            project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
            project_item_type_pointer::ProjectItemTypePointer,
        },
        project_item::ProjectItem,
    },
    structs::valued_struct_field::ValuedStructFieldData,
};
use std::str::FromStr;

pub struct ProjectItemDetailsEditApplier;

impl ProjectItemDetailsEditApplier {
    pub fn apply_update(
        project_item: &mut ProjectItem,
        details_field_source: &DetailsFieldSource,
        details_value: &DetailsValue,
    ) -> Result<bool, String> {
        match details_field_source {
            DetailsFieldSource::ProjectItemProperty { property_name } => Self::apply_project_item_property(project_item, property_name, details_value),
            DetailsFieldSource::ProjectItemAddressTarget { property_name } => Self::apply_address_target_property(project_item, property_name, details_value),
            _ => Ok(false),
        }
    }

    fn apply_project_item_property(
        project_item: &mut ProjectItem,
        property_name: &str,
        details_value: &DetailsValue,
    ) -> Result<bool, String> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if property_name == ProjectItem::PROPERTY_NAME && project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            return Ok(false);
        }

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID && property_name == ProjectItemTypeAddress::PROPERTY_MODULE {
            let module_name = Self::details_value_to_text(details_value)?;

            ProjectItemTypeAddress::set_field_module(project_item, &module_name);

            return Ok(true);
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID && property_name == ProjectItemTypePointer::PROPERTY_POINTER_SIZE {
            let pointer_size_text = Self::details_value_to_text(details_value)?;
            let pointer_size =
                PointerScanPointerSize::from_str(&pointer_size_text).map_err(|error| format!("Invalid pointer size `{}`: {}.", pointer_size_text, error))?;

            ProjectItemTypePointer::set_field_pointer_size(project_item, pointer_size);

            return Ok(true);
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID && property_name == ProjectItemTypePointer::PROPERTY_POINTER_OFFSETS {
            let pointer_offsets = Self::details_value_to_pointer_offsets(details_value)?;

            ProjectItemTypePointer::set_field_pointer_chain_segments(project_item, pointer_offsets);

            return Ok(true);
        }

        project_item
            .get_properties_mut()
            .set_field_data(property_name, Self::details_value_to_field_data(details_value), false);

        Ok(true)
    }

    fn apply_address_target_property(
        project_item: &mut ProjectItem,
        property_name: &str,
        details_value: &DetailsValue,
    ) -> Result<bool, String> {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            return Ok(false);
        }

        let mut address_target = ProjectItemTypeAddress::get_address_target(project_item);

        match property_name {
            "pointer_size" => {
                let pointer_size_text = Self::details_value_to_text(details_value)?;
                let pointer_size = PointerScanPointerSize::from_str(&pointer_size_text)
                    .map_err(|error| format!("Invalid address target pointer size `{}`: {}.", pointer_size_text, error))?;

                address_target.set_pointer_size(pointer_size);
            }
            "pointer_offsets" => {
                address_target.set_pointer_offsets(Self::ensure_minimum_pointer_offsets(Self::details_value_to_pointer_offsets(details_value)?));
            }
            _ => return Ok(false),
        }

        ProjectItemTypeAddress::set_address_target(project_item, address_target);

        Ok(true)
    }

    fn details_value_to_field_data(details_value: &DetailsValue) -> ValuedStructFieldData {
        match details_value {
            DetailsValue::DataValue(data_value) => ValuedStructFieldData::Value(data_value.clone()),
            DetailsValue::Text(text) => ValuedStructFieldData::Value(DataTypeStringUtf8::get_value_from_primitive_string(text)),
            DetailsValue::Boolean(value) => ValuedStructFieldData::Value(DataTypeStringUtf8::get_value_from_primitive_string(&value.to_string())),
            DetailsValue::UnsignedInteger(value) => ValuedStructFieldData::Value(DataTypeStringUtf8::get_value_from_primitive_string(&value.to_string())),
            DetailsValue::SignedInteger(value) => ValuedStructFieldData::Value(DataTypeStringUtf8::get_value_from_primitive_string(&value.to_string())),
            DetailsValue::AnonymousValue(anonymous_value_string) => ValuedStructFieldData::Value(DataTypeStringUtf8::get_value_from_primitive_string(
                anonymous_value_string.get_anonymous_value_string(),
            )),
            DetailsValue::Empty => ValuedStructFieldData::Value(DataTypeStringUtf8::get_value_from_primitive_string("")),
        }
    }

    fn details_value_to_text(details_value: &DetailsValue) -> Result<String, String> {
        match details_value {
            DetailsValue::Text(text) => Ok(text.clone()),
            DetailsValue::DataValue(data_value) => {
                String::from_utf8(data_value.get_value_bytes().clone()).map_err(|error| format!("Details value is not valid UTF-8 text: {}.", error))
            }
            DetailsValue::AnonymousValue(anonymous_value_string) => Ok(anonymous_value_string.get_anonymous_value_string().to_string()),
            DetailsValue::Boolean(value) => Ok(value.to_string()),
            DetailsValue::UnsignedInteger(value) => Ok(value.to_string()),
            DetailsValue::SignedInteger(value) => Ok(value.to_string()),
            DetailsValue::Empty => Ok(String::new()),
        }
    }

    fn details_value_to_pointer_offsets(details_value: &DetailsValue) -> Result<Vec<PointerChainSegment>, String> {
        let pointer_offsets_text = Self::details_value_to_text(details_value)?;
        let pointer_offsets = PointerChainSegment::parse_text_list(&pointer_offsets_text);

        if pointer_offsets.is_empty() {
            Err(String::from("Pointer offsets cannot be empty."))
        } else {
            Ok(pointer_offsets)
        }
    }

    fn ensure_minimum_pointer_offsets(mut pointer_offsets: Vec<PointerChainSegment>) -> Vec<PointerChainSegment> {
        if pointer_offsets.is_empty() {
            pointer_offsets.push(PointerChainSegment::new_offset(0));
        }

        pointer_offsets
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemDetailsEditApplier;
    use crate::structures::{
        data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
        details::{DetailsFieldSource, DetailsValue},
        memory::pointer_chain_segment::PointerChainSegment,
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::project_items::built_in_types::{
            project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
            project_item_type_pointer::ProjectItemTypePointer,
        },
        projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef},
    };
    use std::path::PathBuf;

    #[test]
    fn apply_update_updates_regular_property() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));

        let did_apply = ProjectItemDetailsEditApplier::apply_update(
            &mut project_item,
            &DetailsFieldSource::ProjectItemProperty {
                property_name: ProjectItem::PROPERTY_ICON_ID.to_string(),
            },
            &DetailsValue::DataValue(DataTypeStringUtf8::get_value_from_primitive_string("u64")),
        )
        .expect("Expected property update to apply.");

        assert!(did_apply);
        assert_eq!(project_item.get_field_icon_id(), "u64");
    }

    #[test]
    fn apply_update_ignores_directory_name_property() {
        let project_item_ref = ProjectItemRef::new(PathBuf::from("Root"));
        let mut project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);

        let did_apply = ProjectItemDetailsEditApplier::apply_update(
            &mut project_item,
            &DetailsFieldSource::ProjectItemProperty {
                property_name: ProjectItem::PROPERTY_NAME.to_string(),
            },
            &DetailsValue::DataValue(DataTypeStringUtf8::get_value_from_primitive_string("New Root")),
        )
        .expect("Expected directory name update to be ignored cleanly.");

        assert!(!did_apply);
        assert_eq!(project_item.get_field_name(), "Root");
    }

    #[test]
    fn apply_update_updates_address_target_pointer_size() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));

        let did_apply = ProjectItemDetailsEditApplier::apply_update(
            &mut project_item,
            &DetailsFieldSource::ProjectItemAddressTarget {
                property_name: "pointer_size".to_string(),
            },
            &DetailsValue::Text("u32".to_string()),
        )
        .expect("Expected pointer size update to apply.");
        let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

        assert!(did_apply);
        assert_eq!(address_target.get_pointer_size(), PointerScanPointerSize::Pointer32);
    }

    #[test]
    fn apply_update_serializes_pointer_item_offsets() {
        let pointer = crate::structures::memory::pointer::Pointer::new(0x1000, vec![0x20], "game.exe".to_string());
        let mut project_item = ProjectItemTypePointer::new_project_item("Pointer", &pointer, "", "u64");

        let did_apply = ProjectItemDetailsEditApplier::apply_update(
            &mut project_item,
            &DetailsFieldSource::ProjectItemProperty {
                property_name: ProjectItemTypePointer::PROPERTY_POINTER_OFFSETS.to_string(),
            },
            &DetailsValue::Text("0x40, health".to_string()),
        )
        .expect("Expected pointer offsets update to apply.");

        assert!(did_apply);
        assert_eq!(
            ProjectItemTypePointer::get_field_pointer_chain_segments(&project_item),
            vec![
                PointerChainSegment::Offset(0x40),
                PointerChainSegment::Symbol(String::from("health"))
            ]
        );
    }
}
