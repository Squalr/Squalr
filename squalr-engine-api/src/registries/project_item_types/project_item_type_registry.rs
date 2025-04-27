use crate::structures::projects::project_items::{
    built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
        project_item_type_pointer::ProjectItemTypePointer,
    },
    project_item_type::ProjectItemType,
};
use dashmap::DashMap;
use std::sync::{Arc, Once};

pub struct ProjectItemTypeRegistry {
    registry: DashMap<String, Arc<dyn ProjectItemType>>,
}

impl ProjectItemTypeRegistry {
    pub fn get_instance() -> &'static ProjectItemTypeRegistry {
        static mut INSTANCE: Option<ProjectItemTypeRegistry> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ProjectItemTypeRegistry::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    fn new() -> Self {
        Self {
            registry: Self::create_built_in_types(),
        }
    }

    pub fn get_registry(&self) -> &DashMap<String, Arc<dyn ProjectItemType>> {
        &self.registry
    }

    fn create_built_in_types() -> DashMap<String, Arc<dyn ProjectItemType>> {
        let registry: DashMap<String, Arc<dyn ProjectItemType>> = DashMap::new();

        let built_in_project_item_types: Vec<Arc<dyn ProjectItemType>> = vec![
            Arc::new(ProjectItemTypeDirectory {}),
            Arc::new(ProjectItemTypeAddress {}),
            Arc::new(ProjectItemTypePointer {}),
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
