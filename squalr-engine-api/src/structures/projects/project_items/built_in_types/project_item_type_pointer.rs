use crate::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use crate::registries::registry_context::RegistryContext;
use crate::structures::processes::opened_process_info::OpenedProcessInfo;
use crate::structures::projects::project_items::project_item_ref::ProjectItemRef;
use crate::structures::projects::project_items::project_item_type::ProjectItemType;
use crate::structures::{
    data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8,
    projects::project_items::{project_item::ProjectItem, project_item_type_ref::ProjectItemTypeRef},
    structs::valued_struct_field::ValuedStructFieldData,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize)]
pub struct ProjectItemTypePointer {}

impl ProjectItemTypePointer {
    pub const PROJECT_ITEM_TYPE_ID: &str = "pointer";
    pub const DEFAULT_PROJECT_ITEM_NAME: &str = "New Pointer";
    pub const PROPERTY_FREEZE_DISPLAY_VALUE: &str = "freeze_data_value_interpreter";

    pub fn new_project_item(
        name: &str,
        description: &str,
        freeze_data_value_interpreter: &str,
    ) -> ProjectItem {
        let project_item_type_ref = ProjectItemTypeRef::new(Self::PROJECT_ITEM_TYPE_ID.to_string());
        let project_item_name = if name.trim().is_empty() { Self::DEFAULT_PROJECT_ITEM_NAME } else { name };
        let mut project_item = ProjectItem::new(project_item_type_ref, project_item_name);

        project_item.set_field_description(description);
        Self::set_field_freeze_data_value_interpreter(&mut project_item, freeze_data_value_interpreter);

        project_item
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

    #[test]
    fn new_project_item_uses_new_pointer_for_empty_name() {
        let project_item = ProjectItemTypePointer::new_project_item("", "", "");

        assert_eq!(project_item.get_field_name(), ProjectItemTypePointer::DEFAULT_PROJECT_ITEM_NAME);
    }

    #[test]
    fn new_project_item_uses_supplied_name_when_non_empty() {
        let project_item = ProjectItemTypePointer::new_project_item("Pointer Name", "", "");

        assert_eq!(project_item.get_field_name(), "Pointer Name");
    }
}
