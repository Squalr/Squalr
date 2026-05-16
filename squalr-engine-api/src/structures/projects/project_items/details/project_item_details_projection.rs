use crate::structures::{
    data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8,
    data_values::container_type::ContainerType,
    details::{DetailsEditorHint, DetailsField, DetailsFieldId, DetailsFieldSource, DetailsProjection, DetailsTarget, DetailsValue},
    projects::project_items::{
        built_in_types::{project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer},
        project_item::ProjectItem,
    },
    structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData},
};

pub struct ProjectItemDetailsProjection;

impl ProjectItemDetailsProjection {
    pub const TARGET_KIND_PROJECT_ITEM: &'static str = "project_item";
    pub const FIELD_ID_ADDRESS_TARGET_POINTER_SIZE: &'static str = "address_target.pointer_size";
    pub const FIELD_ID_ADDRESS_TARGET_POINTER_OFFSETS: &'static str = "address_target.pointer_offsets";

    pub fn build(
        project_item: &ProjectItem,
        project_item_target_id: impl Into<String>,
    ) -> DetailsProjection {
        let target = DetailsTarget::new(Self::TARGET_KIND_PROJECT_ITEM, project_item_target_id);
        let fields = Self::build_fields(project_item);

        DetailsProjection::new(target, project_item.get_field_name(), fields)
    }

    pub fn resolve_project_item_icon_data_type_id(project_item: &ProjectItem) -> Option<String> {
        Self::read_project_item_symbolic_struct_namespace(project_item)
    }

    fn build_fields(project_item: &ProjectItem) -> Vec<DetailsField> {
        let mut fields = project_item
            .get_properties()
            .get_fields()
            .iter()
            .filter(|valued_struct_field| Self::should_show_project_item_detail_field(project_item, valued_struct_field.get_name()))
            .map(|valued_struct_field| Self::build_stored_field(project_item, valued_struct_field))
            .collect::<Vec<_>>();

        if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();
            let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

            fields.push(DetailsField::new(
                DetailsFieldId::new(Self::FIELD_ID_ADDRESS_TARGET_POINTER_SIZE),
                "Pointer Size",
                DetailsValue::Text(address_target.get_pointer_size().to_string()),
                false,
                DetailsEditorHint::PointerSize,
                Some(address_target.get_pointer_size().to_data_type_ref()),
                ContainerType::None,
                DetailsFieldSource::ProjectItemAddressTarget {
                    property_name: "pointer_size".to_string(),
                },
            ));
            fields.push(DetailsField::new(
                DetailsFieldId::new(Self::FIELD_ID_ADDRESS_TARGET_POINTER_OFFSETS),
                "Offsets",
                DetailsValue::Text(Self::format_pointer_offsets(address_target.get_pointer_offsets())),
                true,
                DetailsEditorHint::PointerOffsets,
                Some(
                    DataTypeStringUtf8::get_value_from_primitive_string("")
                        .get_data_type_ref()
                        .clone(),
                ),
                ContainerType::None,
                DetailsFieldSource::ProjectItemAddressTarget {
                    property_name: "pointer_offsets".to_string(),
                },
            ));
        }

        fields
    }

    fn build_stored_field(
        project_item: &ProjectItem,
        valued_struct_field: &ValuedStructField,
    ) -> DetailsField {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();
        let field_name = valued_struct_field.get_name();
        let field_data = Self::project_address_item_target_detail_field_data(project_item, valued_struct_field)
            .unwrap_or_else(|| valued_struct_field.get_field_data().clone());
        let details_value = Self::details_value_from_field_data(&field_data);
        let is_runtime_value_field = Self::is_runtime_value_field(project_item_type_id, field_name);

        DetailsField::new(
            DetailsFieldId::new(format!("property.{}", field_name)),
            Self::field_label(field_name),
            details_value,
            if is_runtime_value_field {
                false
            } else {
                valued_struct_field.get_is_read_only()
            },
            Self::editor_hint_for_field(project_item_type_id, field_name),
            Self::validation_data_type_ref_for_field(project_item, field_name, &field_data),
            ContainerType::None,
            Self::field_source_for_field(project_item_type_id, field_name),
        )
    }

    fn details_value_from_field_data(field_data: &ValuedStructFieldData) -> DetailsValue {
        match field_data {
            ValuedStructFieldData::Value(data_value) => DetailsValue::DataValue(data_value.clone()),
            ValuedStructFieldData::NestedStruct(nested_struct) => DetailsValue::Text(nested_struct.get_display_string(false)),
        }
    }

    fn validation_data_type_ref_for_field(
        project_item: &ProjectItem,
        field_name: &str,
        field_data: &ValuedStructFieldData,
    ) -> Option<crate::structures::data_types::data_type_ref::DataTypeRef> {
        if Self::is_runtime_value_field(project_item.get_item_type().get_project_item_type_id(), field_name) {
            return Self::read_project_item_symbolic_struct_namespace(project_item)
                .map(|symbolic_struct_namespace| crate::structures::data_types::data_type_ref::DataTypeRef::new(&symbolic_struct_namespace));
        }

        match field_data {
            ValuedStructFieldData::Value(data_value) => Some(data_value.get_data_type_ref().clone()),
            ValuedStructFieldData::NestedStruct(_) => None,
        }
    }

    fn field_source_for_field(
        project_item_type_id: &str,
        field_name: &str,
    ) -> DetailsFieldSource {
        if Self::is_runtime_value_field(project_item_type_id, field_name) {
            DetailsFieldSource::ProjectItemRuntimeValue {
                field_path: vec!["value".to_string()],
            }
        } else {
            DetailsFieldSource::ProjectItemProperty {
                property_name: field_name.to_string(),
            }
        }
    }

    fn editor_hint_for_field(
        project_item_type_id: &str,
        field_name: &str,
    ) -> DetailsEditorHint {
        match (project_item_type_id, field_name) {
            (_, ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE) => DetailsEditorHint::DataType,
            (ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID, ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
            | (ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID, ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE) => DetailsEditorHint::Value,
            (ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID, ProjectItemTypePointer::PROPERTY_POINTER_OFFSETS) => DetailsEditorHint::PointerOffsets,
            (ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID, ProjectItemTypePointer::PROPERTY_POINTER_SIZE) => DetailsEditorHint::PointerSize,
            (ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID, ProjectItemTypeAddress::PROPERTY_MODULE)
            | (ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID, ProjectItemTypePointer::PROPERTY_MODULE) => DetailsEditorHint::Text,
            _ => DetailsEditorHint::Value,
        }
    }

    fn should_show_project_item_detail_field(
        project_item: &ProjectItem,
        field_name: &str,
    ) -> bool {
        if field_name == ProjectItemTypeAddress::PROPERTY_TARGET {
            return false;
        }

        if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID
            && field_name == ProjectItemTypeAddress::PROPERTY_ADDRESS
        {
            return false;
        }

        if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return field_name != ProjectItemTypePointer::PROPERTY_EVALUATED_POINTER_PATH;
        }

        true
    }

    fn project_address_item_target_detail_field_data(
        project_item: &ProjectItem,
        valued_struct_field: &ValuedStructField,
    ) -> Option<ValuedStructFieldData> {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            return None;
        }

        let mut project_item = project_item.clone();
        let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

        match valued_struct_field.get_name() {
            ProjectItemTypeAddress::PROPERTY_MODULE => Some(ValuedStructFieldData::Value(DataTypeStringUtf8::get_value_from_primitive_string(
                address_target.get_module_name(),
            ))),
            _ => None,
        }
    }

    fn is_runtime_value_field(
        project_item_type_id: &str,
        field_name: &str,
    ) -> bool {
        matches!(
            (project_item_type_id, field_name),
            (
                ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID,
                ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE
            ) | (
                ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID,
                ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE
            )
        )
    }

    fn read_project_item_symbolic_struct_namespace(project_item: &ProjectItem) -> Option<String> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut project_item)
                .map(|symbolic_struct_ref| symbolic_struct_ref.get_symbolic_struct_namespace().to_string())
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item)
                .map(|symbolic_struct_ref| symbolic_struct_ref.get_symbolic_struct_namespace().to_string())
        } else {
            None
        }
    }

    fn format_pointer_offsets(pointer_offsets: &[crate::structures::memory::pointer_chain_segment::PointerChainSegment]) -> String {
        crate::structures::memory::pointer_chain_segment::PointerChainSegment::display_text_list(pointer_offsets)
    }

    fn field_label(field_name: &str) -> String {
        match field_name {
            ProjectItem::PROPERTY_NAME => String::from("Name"),
            ProjectItem::PROPERTY_ICON_ID => String::from("Icon ID"),
            ProjectItem::PROPERTY_DESCRIPTION => String::from("Description"),
            ProjectItemTypeAddress::PROPERTY_MODULE => String::from("Module"),
            ProjectItemTypePointer::PROPERTY_OFFSET => String::from("Offset"),
            ProjectItemTypePointer::PROPERTY_POINTER_OFFSETS => String::from("Pointer Offsets"),
            ProjectItemTypePointer::PROPERTY_POINTER_SIZE => String::from("Pointer Size"),
            ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE => String::from("Data Type"),
            ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE => String::from("Value"),
            _ => Self::humanize_field_key(field_name),
        }
    }

    fn humanize_field_key(field_name: &str) -> String {
        let mut display_name = String::new();

        for word in field_name
            .trim_matches('_')
            .split(|character| matches!(character, '_' | '.'))
            .filter(|word| !word.is_empty())
        {
            if !display_name.is_empty() {
                display_name.push(' ');
            }

            let mut characters = word.chars();
            if let Some(first_character) = characters.next() {
                display_name.extend(first_character.to_uppercase());
                display_name.push_str(characters.as_str());
            }
        }

        if display_name.is_empty() { String::from("Field") } else { display_name }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemDetailsProjection;
    use crate::structures::{
        data_types::built_in_types::{
            string::utf8::data_type_string_utf8::DataTypeStringUtf8, u16::data_type_u16::DataTypeU16, u64::data_type_u64::DataTypeU64,
        },
        details::{DetailsEditorHint, DetailsFieldSource, DetailsValue},
        memory::pointer::Pointer,
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::project_items::built_in_types::{
            project_item_type_address::ProjectItemTypeAddress, project_item_type_address_target::ProjectItemAddressTarget,
            project_item_type_pointer::ProjectItemTypePointer,
        },
    };

    #[test]
    fn project_item_details_projection_exposes_address_target_fields() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let details_projection = ProjectItemDetailsProjection::build(&project_item, "/Health");

        let pointer_size_field = details_projection
            .get_field(&crate::structures::details::DetailsFieldId::new(
                ProjectItemDetailsProjection::FIELD_ID_ADDRESS_TARGET_POINTER_SIZE,
            ))
            .expect("Expected pointer size field.");
        let pointer_offsets_field = details_projection
            .get_field(&crate::structures::details::DetailsFieldId::new(
                ProjectItemDetailsProjection::FIELD_ID_ADDRESS_TARGET_POINTER_OFFSETS,
            ))
            .expect("Expected pointer offsets field.");

        assert_eq!(details_projection.get_target().get_target_id(), "/Health");
        assert_eq!(pointer_size_field.get_label(), "Pointer Size");
        assert_eq!(pointer_size_field.get_editor_hint(), &DetailsEditorHint::PointerSize);
        assert_eq!(pointer_offsets_field.get_label(), "Offsets");
        assert_eq!(pointer_offsets_field.get_editor_hint(), &DetailsEditorHint::PointerOffsets);
        assert_eq!(pointer_offsets_field.get_value(), &DetailsValue::Text("0x1234".to_string()));
    }

    #[test]
    fn project_item_details_projection_uses_address_target_module() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        ProjectItemTypeAddress::set_address_target(
            &mut project_item,
            ProjectItemAddressTarget::new_pointer_path(Pointer::new_with_size(
                0x4567,
                vec![0x10, 0x20],
                String::from("pointer_root.exe"),
                PointerScanPointerSize::Pointer64,
            )),
        );
        let details_projection = ProjectItemDetailsProjection::build(&project_item, "/Health");
        let module_field = details_projection
            .get_field(&crate::structures::details::DetailsFieldId::new(format!(
                "property.{}",
                ProjectItemTypeAddress::PROPERTY_MODULE
            )))
            .expect("Expected module field.");
        let pointer_offsets_field = details_projection
            .get_field(&crate::structures::details::DetailsFieldId::new(
                ProjectItemDetailsProjection::FIELD_ID_ADDRESS_TARGET_POINTER_OFFSETS,
            ))
            .expect("Expected pointer offsets field.");

        assert_eq!(
            module_field.get_value(),
            &DetailsValue::DataValue(DataTypeStringUtf8::get_value_from_primitive_string("pointer_root.exe"))
        );
        assert_eq!(pointer_offsets_field.get_value(), &DetailsValue::Text("0x4567, 0x10, 0x20".to_string()));
    }

    #[test]
    fn project_item_details_projection_hides_pointer_preview_path() {
        let pointer = Pointer::new_with_size(0x4567, vec![0x10, 0x20], String::from("game.exe"), PointerScanPointerSize::Pointer64);
        let mut project_item = ProjectItemTypePointer::new_project_item("Ammo Pointer", &pointer, "", "u16");
        ProjectItemTypePointer::set_field_evaluated_pointer_path(&mut project_item, "game.exe+0x4567 -> 0x5000 -> 0x6000");
        let details_projection = ProjectItemDetailsProjection::build(&project_item, "/Ammo Pointer");

        assert!(
            details_projection
                .get_field(&crate::structures::details::DetailsFieldId::new("property.offset"))
                .is_some()
        );
        assert!(
            details_projection
                .get_field(&crate::structures::details::DetailsFieldId::new("property.module"))
                .is_some()
        );
        assert!(
            details_projection
                .get_field(&crate::structures::details::DetailsFieldId::new("property.pointer_offsets"))
                .is_some()
        );
        assert!(
            details_projection
                .get_field(&crate::structures::details::DetailsFieldId::new("property.pointer_size"))
                .is_some()
        );
        assert!(
            details_projection
                .get_field(&crate::structures::details::DetailsFieldId::new("property.evaluated_pointer_path"))
                .is_none()
        );
    }

    #[test]
    fn project_item_details_projection_marks_runtime_value_as_editable() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU16::get_value_from_primitive(0));
        let details_projection = ProjectItemDetailsProjection::build(&project_item, "/Health");
        let runtime_value_field = details_projection
            .get_field(&crate::structures::details::DetailsFieldId::new(format!(
                "property.{}",
                ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE
            )))
            .expect("Expected runtime value field.");

        assert!(!runtime_value_field.get_is_read_only());
        assert_eq!(runtime_value_field.get_editor_hint(), &DetailsEditorHint::Value);
        assert_eq!(
            runtime_value_field.get_source(),
            &DetailsFieldSource::ProjectItemRuntimeValue {
                field_path: vec!["value".to_string()]
            }
        );
    }
}
