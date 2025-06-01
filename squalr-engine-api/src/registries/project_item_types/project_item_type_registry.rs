use crate::structures::projects::project_items::{
    built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
        project_item_type_pointer::ProjectItemTypePointer,
    },
    project_item_type::ProjectItemType,
};
use std::{
    collections::HashMap,
    sync::{Arc, Once, RwLock},
};

pub struct ProjectItemTypeRegistry {
    registry: RwLock<HashMap<String, Arc<dyn ProjectItemType>>>,
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

    pub fn get_registry(&self) -> &RwLock<HashMap<String, Arc<dyn ProjectItemType>>> {
        &self.registry
    }

    fn create_built_in_types() -> RwLock<HashMap<String, Arc<dyn ProjectItemType>>> {
        let mut registry: HashMap<String, Arc<dyn ProjectItemType>> = HashMap::new();

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

        RwLock::new(registry)
    }
}
