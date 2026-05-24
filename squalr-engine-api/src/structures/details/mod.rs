pub mod details_edit;
pub mod details_edit_plan;
pub mod details_field;
pub mod details_projection;
pub mod details_target;

pub use details_edit::DetailsEdit;
pub use details_edit_plan::{DetailsEditOperation, DetailsEditPlan};
pub use details_field::{DetailsEditorHint, DetailsField, DetailsFieldId, DetailsFieldSource, DetailsValue};
pub use details_projection::DetailsProjection;
pub use details_target::DetailsTarget;

#[cfg(test)]
mod tests {
    use super::{
        DetailsEdit, DetailsEditOperation, DetailsEditPlan, DetailsEditorHint, DetailsField, DetailsFieldId, DetailsFieldSource, DetailsProjection,
        DetailsTarget, DetailsValue,
    };
    use crate::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    };

    #[test]
    fn details_projection_round_trips_stable_field_ids() {
        let target = DetailsTarget::new("project_item", "/Health");
        let field_id = DetailsFieldId::new("runtime.value");
        let details_value = DetailsValue::AnonymousValue(AnonymousValueString::new(
            "100".to_string(),
            AnonymousValueStringFormat::Decimal,
            ContainerType::None,
        ));
        let field = DetailsField::new(
            field_id.clone(),
            "Health",
            details_value,
            false,
            DetailsEditorHint::Value,
            Some(DataTypeRef::new("u32")),
            ContainerType::None,
            DetailsFieldSource::ProjectItemRuntimeValue {
                field_path: vec!["value".to_string()],
            },
        );
        let projection = DetailsProjection::new(target, "Health Address", vec![field]);

        let serialized_projection = serde_json::to_string(&projection).expect("Expected details projection to serialize.");
        let deserialized_projection: DetailsProjection = serde_json::from_str(&serialized_projection).expect("Expected details projection to deserialize.");

        let deserialized_field = deserialized_projection
            .get_field(&field_id)
            .expect("Expected stable field id to find the projected field.");

        assert_eq!(deserialized_projection.get_title(), "Health Address");
        assert_eq!(deserialized_field.get_label(), "Health");
        assert_eq!(deserialized_field.get_editor_hint(), &DetailsEditorHint::Value);
        assert_eq!(deserialized_field.get_validation_data_type_ref(), Some(&DataTypeRef::new("u32")));
    }

    #[test]
    fn details_edit_routes_by_target_and_field_id() {
        let target = DetailsTarget::new("project_symbol", "Player.Health");
        let edit = DetailsEdit::new_with_source(
            target.clone(),
            DetailsFieldId::new("runtime.value"),
            DetailsFieldSource::ProjectSymbolRuntimeValue {
                field_path: vec!["value".to_string()],
            },
            DetailsValue::UnsignedInteger(255),
        );

        assert_eq!(edit.get_target(), &target);
        assert_eq!(edit.get_field_id().get_field_id(), "runtime.value");
        assert_eq!(
            edit.get_source(),
            &DetailsFieldSource::ProjectSymbolRuntimeValue {
                field_path: vec!["value".to_string()]
            }
        );
        assert_eq!(edit.get_value(), &DetailsValue::UnsignedInteger(255));
    }

    #[test]
    fn details_edit_plan_keeps_semantic_operations_ordered() {
        let target = DetailsTarget::new("project_item", "/Health");
        let edit_plan = DetailsEditPlan::new(vec![
            DetailsEditOperation::UpdateStoredField {
                target: target.clone(),
                source: DetailsFieldSource::ProjectItemProperty {
                    property_name: "name".to_string(),
                },
                value: DetailsValue::Text("Health".to_string()),
            },
            DetailsEditOperation::RefreshProjection { target },
        ]);

        assert_eq!(edit_plan.get_operations().len(), 2);
        assert!(matches!(
            edit_plan.get_operations().first(),
            Some(DetailsEditOperation::UpdateStoredField { .. })
        ));
        assert!(matches!(
            edit_plan.get_operations().last(),
            Some(DetailsEditOperation::RefreshProjection { .. })
        ));
    }
}
