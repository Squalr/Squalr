use crate::structures::{
    details::{DetailsEdit, DetailsEditOperation, DetailsEditPlan, DetailsFieldSource},
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

        match details_edit.get_source() {
            DetailsFieldSource::ProjectItemAddressTarget { property_name } => {
                return Self::plan_address_target_update(details_edit, property_name);
            }
            DetailsFieldSource::ProjectItemRuntimeValue { .. } => {
                return Self::plan_runtime_value_update(details_edit, details_edit.get_source().clone());
            }
            DetailsFieldSource::ProjectItemProperty { property_name } => {
                return Self::plan_project_item_property_update(project_item_type_id, details_edit, property_name);
            }
            DetailsFieldSource::Unknown
            | DetailsFieldSource::ProjectSymbolRuntimeValue { .. }
            | DetailsFieldSource::SymbolLayoutMetadata { .. }
            | DetailsFieldSource::SymbolResolverMetadata { .. } => {}
        }

        if field_id == ProjectItemDetailsProjection::FIELD_ID_ADDRESS_TARGET_POINTER_SIZE {
            return Self::plan_address_target_update(details_edit, "pointer_size");
        }

        if field_id == ProjectItemDetailsProjection::FIELD_ID_ADDRESS_TARGET_POINTER_OFFSETS {
            return Self::plan_address_target_update(details_edit, "pointer_offsets");
        }

        let Some(property_name) = field_id.strip_prefix(ProjectItemDetailsProjection::FIELD_ID_PROPERTY_PREFIX) else {
            return DetailsEditPlan::reject(format!("Unknown project item details field id: {}.", field_id));
        };

        Self::plan_project_item_property_update(project_item_type_id, details_edit, property_name)
    }

    fn plan_project_item_property_update(
        project_item_type_id: &str,
        details_edit: &DetailsEdit,
        property_name: &str,
    ) -> DetailsEditPlan {
        if Self::is_runtime_value_property(project_item_type_id, property_name) {
            return Self::plan_runtime_value_update(
                details_edit,
                DetailsFieldSource::ProjectItemRuntimeValue {
                    field_path: vec!["value".to_string()],
                },
            );
        }

        if property_name == ProjectItem::PROPERTY_NAME && project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            return DetailsEditPlan::noop(Some("Directory item names are owned by their directory path.".to_string()));
        }

        DetailsEditPlan::new(vec![
            DetailsEditOperation::UpdateStoredField {
                target: details_edit.get_target().clone(),
                source: DetailsFieldSource::ProjectItemProperty {
                    property_name: property_name.to_string(),
                },
                value: details_edit.get_value().clone(),
            },
            DetailsEditOperation::RefreshProjection {
                target: details_edit.get_target().clone(),
            },
        ])
    }

    fn plan_runtime_value_update(
        details_edit: &DetailsEdit,
        runtime_value_source: DetailsFieldSource,
    ) -> DetailsEditPlan {
        DetailsEditPlan::new(vec![
            DetailsEditOperation::WriteRuntimeValue {
                target: details_edit.get_target().clone(),
                field_id: details_edit.get_field_id().clone(),
                source: runtime_value_source,
                value: details_edit.get_value().clone(),
            },
            DetailsEditOperation::RefreshProjection {
                target: details_edit.get_target().clone(),
            },
        ])
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
    fn plan_edit_preserves_runtime_value_source_path() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let details_edit = DetailsEdit::new_with_source(
            DetailsTarget::new(ProjectItemDetailsProjection::TARGET_KIND_PROJECT_ITEM, "/Health"),
            DetailsFieldId::new(format!(
                "{}{}",
                ProjectItemDetailsProjection::FIELD_ID_PROPERTY_PREFIX,
                ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE
            )),
            DetailsFieldSource::ProjectItemRuntimeValue {
                field_path: vec![String::from("nested"), String::from("health")],
            },
            DetailsValue::DataValue(DataTypeU64::get_value_from_primitive(500)),
        );
        let edit_plan = ProjectItemDetailsEditPlanner::plan_edit(&project_item, &details_edit);

        assert!(matches!(
            edit_plan.get_operations().first(),
            Some(DetailsEditOperation::WriteRuntimeValue {
                source: DetailsFieldSource::ProjectItemRuntimeValue { field_path },
                ..
            }) if field_path == &vec![String::from("nested"), String::from("health")]
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
    fn plan_edit_routes_address_target_by_source_without_field_id_namespace() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        let details_edit = DetailsEdit::new_with_source(
            DetailsTarget::new(ProjectItemDetailsProjection::TARGET_KIND_PROJECT_ITEM, "/Health"),
            DetailsFieldId::new("view_field_0"),
            DetailsFieldSource::ProjectItemAddressTarget {
                property_name: "pointer_size".to_string(),
            },
            DetailsValue::Text("u32".to_string()),
        );
        let edit_plan = ProjectItemDetailsEditPlanner::plan_edit(&project_item, &details_edit);

        assert_eq!(
            edit_plan.get_operations().first(),
            Some(&DetailsEditOperation::UpdateStoredField {
                target: details_edit.get_target().clone(),
                source: DetailsFieldSource::ProjectItemAddressTarget {
                    property_name: "pointer_size".to_string()
                },
                value: DetailsValue::Text("u32".to_string())
            })
        );
    }

    #[test]
    fn plan_edit_routes_property_by_source_without_property_prefix() {
        let mut project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));
        project_item.set_field_icon_id(DataTypeU8::DATA_TYPE_ID);
        let details_edit = DetailsEdit::new_with_source(
            DetailsTarget::new(ProjectItemDetailsProjection::TARGET_KIND_PROJECT_ITEM, "/Health"),
            DetailsFieldId::new("view_field_1"),
            DetailsFieldSource::ProjectItemProperty {
                property_name: ProjectItem::PROPERTY_ICON_ID.to_string(),
            },
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

    #[test]
    fn plan_edit_updates_file_item_name_without_renaming_file() {
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
                .any(|operation| matches!(operation, DetailsEditOperation::UpdateStoredField {
                    source: DetailsFieldSource::ProjectItemProperty { property_name },
                    ..
                } if property_name == ProjectItem::PROPERTY_NAME))
        );
        assert!(
            !edit_plan
                .get_operations()
                .iter()
                .any(|operation| matches!(operation, DetailsEditOperation::Reject { .. } | DetailsEditOperation::Noop { .. }))
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
