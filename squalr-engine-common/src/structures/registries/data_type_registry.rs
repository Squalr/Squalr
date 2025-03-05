use crate::structures::data_types::built_in_types::i32::data_type_i32::DataTypeI32;
use crate::structures::data_types::data_type::DataType;
use dashmap::DashMap;
use std::sync::{Arc, Once};

pub struct DataTypeRegistry {
    registry: DashMap<String, Arc<dyn DataType>>,
}

impl DataTypeRegistry {
    pub fn get_instance() -> &'static DataTypeRegistry {
        static mut INSTANCE: Option<DataTypeRegistry> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = DataTypeRegistry::new();
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

    pub fn get_registry(&self) -> &DashMap<String, Arc<dyn DataType>> {
        &self.registry
    }

    fn create_built_in_types() -> DashMap<String, Arc<dyn DataType>> {
        let registry: DashMap<String, Arc<dyn DataType>> = DashMap::new();
        registry.insert("i32".to_string(), Arc::new(DataTypeI32 {}));

        registry
    }
}
