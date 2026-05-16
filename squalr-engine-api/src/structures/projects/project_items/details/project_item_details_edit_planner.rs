use crate::structures::{
    details::{DetailsEdit, DetailsEditOperation, DetailsEditPlan, DetailsFieldSource, DetailsValue},
    projects::project_items::{
        built_in_types::{
            project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
            project_item_type_pointer::ProjectItemTypePointer,
        },
        details::ProjectItemDetailsProjection,
        project_item::ProjectItem,
    },
};

pub struct ProjectItemDetailsEditPlanner;

impl ProjectItemDetailsEditPlanner {
    pub fn plan_edit(
        project_item: &ProjectItem,
        details_edit: &DetailsEdit,
    ) -> DetailsEditPlan {
        if details_edit.get_target().get_target_kind() != ProjectItemDetailsProjection::TARGET_KIND_PROJECT_ITEM {
            return DetailsEditPlan::reject("Details edit target is not a project item.");
        }

        let field_id = details_edit.get_field_id().get_field_id();
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if field_id == ProjectItemDetailsProjection::FIELD_ID_ADDRESS_TARGET_POINTER_SIZE {
            return Self::plan_address_target_update(details_edit, "pointer_size");
        }

        if field_id == ProjectItemDetailsProjection::FIELD_ID_ADDRESS_TARGET_POINTER_OFFSETS {
            return Self::plan_address_target_update(details_edit, "pointer_offsets");
        }

        let Some(property_name) = field_id.strip_prefix(ProjectItemDetailsProjection::FIELD_ID_PROPERTY_PREFIX) else {
            return DetailsEditPlan::reject(format!("Unknown project item details field id: {}.", field_id));
        };

        if Self::is_runtime_value_property(project_item_type_id, property_name) {
            return DetailsEditPlan::new(vec![
                DetailsEditOperation::WriteRuntimeValue {
                    target: details_edit.get_target().clone(),
                    field_id: details_edit.get_field_id().clone(),
                    value: details_edit.get_value().clone(),
                },
                DetailsEditOperation::RefreshProjection {
                    target: details_edit.get_target().clone(),
                },
            ]);
        }

        if property_name == ProjectItem::PROPERTY_NAME && project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            return DetailsEditPlan::noop(Some("Directory item names are owned by their directory path.".to_string()));
        }

        let mut operations = vec![DetailsEditOperation::UpdateStoredField {
            target: details_edit.get_target().clone(),
            source: DetailsFieldSource::ProjectItemProperty {
                property_name: property_name.to_string(),
            },
            value: details_edit.get_value().clone(),
        }];

        if property_name == ProjectItem::PROPERTY_NAME {
            if let Some(name) = Self::details_value_to_text(details_edit.get_value()) {
                operations.push(DetailsEditOperation::RenameTarget {
                    target: details_edit.get_target().clone(),
                    name,
                });
            }
        }

        operations.push(DetailsEditOperation::RefreshProjection {
            target: details_edit.get_target().clone(),
        });

        DetailsEditPlan::new(operations)
    }

    fn plan_address_target_update(
        details_edit: &DetailsEdit,
        property_name: &str,
    ) -> DetailsEditPlan {
        DetailsEditPlan::new(vec![
            DetailsEditOperation::UpdateStoredField {
                target: details_edit.get_target().clone(),
                source: DetailsFieldSource::ProjectItemAddressTarget {
                    property_name: property_name.to_string(),
                },
                value: details_edit.get_value().clone(),
            },
            DetailsEditOperation::RefreshProjection {
                target: details_edit.get_target().clone(),
            },
        ])
    }

    fn is_runtime_value_property(
        project_item_type_id: &str,
        property_name: &str,
    ) -> bool {
        matches!(
            (project_item_type_id, property_name),
            (
                ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID,
                ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE
            ) | (
                ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID,
                ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE
            )
        )
    }

    fn details_value_to_text(details_value: &DetailsValue) -> Option<String> {
        match details_value {
            DetailsValue::Text(text) => Some(text.clone()),
            DetailsValue::DataValue(data_value) => String::from_utf8(data_value.get_value_bytes().clone()).ok(),
            DetailsValue::AnonymousValue(anonymous_value_string) => Some(anonymous_value_string.get_anonymous_value_string().to_string()),
            DetailsValue::Boolean(value) => Some(value.to_string()),
            DetailsValue::UnsignedInteger(value) => Some(value.to_string()),
            DetailsValue::SignedInteger(value) => Some(value.to_string()),
            DetailsValue::Empty => Some(String::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemDetailsEditPlanner;
    use crate::structures::{
        data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u8::data_type_u8::DataTypeU8, u64::data_type_u64::DataTypeU64},
        details::{DetailsEdit, DetailsEditOperation, DetailsFieldId, DetailsFieldSource, DetailsTarget, DetailsValue},
        projects::project_items::{
            built_in_types::{project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory},
            details::ProjectItemDetailsProjection,
            project_item::ProjectItem,
            project_item_ref::ProjectItemRef,
        },
    };
    use std::path::PathBuf;

    #[test]
    fn plan_edit_returns_runtime_write_for_project_item_value() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let details_edit = DetailsEdit::new(
            DetailsTarget::new(ProjectItemDetailsProjection::TARGET_KIND_PROJECT_ITEM, "/Health"),
            DetailsFieldId::new(format!(
                "{}{}",
                ProjectItemDetailsProjection::FIELD_ID_PROPERTY_PREFIX,
                ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE
            )),
            DetailsValue::DataValue(DataTypeU64::get_value_from_primitive(500)),
        );
        let edit_plan = ProjectItemDetailsEditPlanner::plan_edit(&project_item, &details_edit);

        assert!(matches!(
            edit_plan.get_operations().first(),
            Some(DetailsEditOperation::WriteRuntimeValue { .. })
        ));
    }

    #[test]
    fn plan_edit_updates_address_target_pointer_size() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let details_edit = DetailsEdit::new(
            DetailsTarget::new(ProjectItemDetailsProjection::TARGET_KIND_PROJECT_ITEM, "/Health"),
            DetailsFieldId::new(ProjectItemDetailsProjection::FIELD_ID_ADDRESS_TARGET_POINTER_SIZE),
            DetailsValue::Text("u64".to_string()),
        );
        let edit_plan = ProjectItemDetailsEditPlanner::plan_edit(&project_item, &details_edit);

        assert_eq!(
            edit_plan.get_operations().first(),
            Some(&DetailsEditOperation::UpdateStoredField {
                target: details_edit.get_target().clone(),
                source: DetailsFieldSource::ProjectItemAddressTarget {
                    property_name: "pointer_size".to_string()
                },
                value: DetailsValue::Text("u64".to_string())
            })
        );
    }

    #[test]
    fn plan_edit_includes_rename_for_file_item_name() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let details_edit = DetailsEdit::new(
            DetailsTarget::new(ProjectItemDetailsProjection::TARGET_KIND_PROJECT_ITEM, "/Health"),
            DetailsFieldId::new(format!(
                "{}{}",
                ProjectItemDetailsProjection::FIELD_ID_PROPERTY_PREFIX,
                ProjectItem::PROPERTY_NAME
            )),
            DetailsValue::DataValue(DataTypeStringUtf8::get_value_from_primitive_string("New Health")),
        );
        let edit_plan = ProjectItemDetailsEditPlanner::plan_edit(&project_item, &details_edit);

        assert!(
            edit_plan
                .get_operations()
                .iter()
                .any(|operation| matches!(operation, DetailsEditOperation::RenameTarget { name, .. } if name == "New Health"))
        );
    }

    #[test]
    fn plan_edit_ignores_directory_item_name() {
        let project_item_ref = ProjectItemRef::new(PathBuf::from("Root"));
        let project_item = ProjectItemTypeDirectory::new_project_item(&project_item_ref);
        let details_edit = DetailsEdit::new(
            DetailsTarget::new(ProjectItemDetailsProjection::TARGET_KIND_PROJECT_ITEM, "/Root"),
            DetailsFieldId::new(format!(
                "{}{}",
                ProjectItemDetailsProjection::FIELD_ID_PROPERTY_PREFIX,
                ProjectItem::PROPERTY_NAME
            )),
            DetailsValue::DataValue(DataTypeStringUtf8::get_value_from_primitive_string("New Root")),
        );
        let edit_plan = ProjectItemDetailsEditPlanner::plan_edit(&project_item, &details_edit);

        assert!(matches!(edit_plan.get_operations().first(), Some(DetailsEditOperation::Noop { .. })));
    }

    #[test]
    fn plan_edit_updates_regular_stored_property() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        project_item.set_field_icon_id(DataTypeU8::DATA_TYPE_ID);
        let details_edit = DetailsEdit::new(
            DetailsTarget::new(ProjectItemDetailsProjection::TARGET_KIND_PROJECT_ITEM, "/Health"),
            DetailsFieldId::new(format!(
                "{}{}",
                ProjectItemDetailsProjection::FIELD_ID_PROPERTY_PREFIX,
                ProjectItem::PROPERTY_ICON_ID
            )),
            DetailsValue::DataValue(DataTypeStringUtf8::get_value_from_primitive_string("u64")),
        );
        let edit_plan = ProjectItemDetailsEditPlanner::plan_edit(&project_item, &details_edit);

        assert!(matches!(
            edit_plan.get_operations().first(),
            Some(DetailsEditOperation::UpdateStoredField {
                source: DetailsFieldSource::ProjectItemProperty { property_name },
                ..
            }) if property_name == ProjectItem::PROPERTY_ICON_ID
        ));
    }
}
