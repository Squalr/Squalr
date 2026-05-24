use crate::structures::projects::project_items::{
    built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
        project_item_type_script::ProjectItemTypeScript,
    },
    project_item_type::ProjectItemType,
};
use std::{collections::HashMap, sync::Arc};

pub struct ProjectItemTypeRegistry {
    registry: HashMap<String, Arc<dyn ProjectItemType>>,
}

impl ProjectItemTypeRegistry {
    pub fn new() -> Self {
        Self {
            registry: Self::create_built_in_types(),
        }
    }

    pub fn get(
        &self,
        project_item_type_id: &str,
    ) -> Option<Arc<dyn ProjectItemType>> {
        self.registry.get(project_item_type_id).cloned()
    }

    pub fn get_registry(&self) -> &HashMap<String, Arc<dyn ProjectItemType>> {
        &self.registry
    }

    pub fn register(
        &mut self,
        project_item_type: Arc<dyn ProjectItemType>,
    ) -> Result<(), String> {
        let project_item_type_id = project_item_type.get_project_item_type_id().trim().to_string();

        if project_item_type_id.is_empty() {
            return Err("Project item type id cannot be empty.".to_string());
        }

        if self.registry.contains_key(&project_item_type_id) {
            return Err(format!("Project item type is already registered: {}", project_item_type_id));
        }

        self.registry.insert(project_item_type_id, project_item_type);

        Ok(())
    }

    fn create_built_in_types() -> HashMap<String, Arc<dyn ProjectItemType>> {
        let mut registry: HashMap<String, Arc<dyn ProjectItemType>> = HashMap::new();

        let built_in_project_item_types: Vec<Arc<dyn ProjectItemType>> = vec![
            Arc::new(ProjectItemTypeDirectory {}),
            Arc::new(ProjectItemTypeAddress {}),
            Arc::new(ProjectItemTypeScript {}),
        ];

        for built_in_project_item_type in built_in_project_item_types.into_iter() {
            registry.insert(
                built_in_project_item_type
                    .get_project_item_type_id()
                    .to_string(),
                built_in_project_item_type,
            );
        }

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemTypeRegistry;
    use crate::structures::projects::project_items::built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
        project_item_type_script::ProjectItemTypeScript,
    };

    #[test]
    fn built_in_project_item_types_are_directory_address_and_script() {
        let project_item_type_registry = ProjectItemTypeRegistry::new();
        let registry = project_item_type_registry.get_registry();

        assert!(registry.contains_key(ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID));
        assert!(registry.contains_key(ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID));
        assert!(registry.contains_key(ProjectItemTypeScript::PROJECT_ITEM_TYPE_ID));
        assert!(!registry.contains_key("item"));
        assert!(!registry.contains_key("plugin"));
    }
}
