use crate::structures::data_types::built_in_types::i32::data_type_i32::DataTypeI32;
use crate::structures::data_types::data_type::DataType;
use dashmap::DashMap;
use std::sync::Once;

pub struct DataTypeRegistry {
    registry: DashMap<String, Box<dyn DataType>>,
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

    pub fn get_registry(&self) -> &DashMap<String, Box<dyn DataType>> {
        &self.registry
    }

    fn create_built_in_types() -> DashMap<String, Box<dyn DataType>> {
        let registry: DashMap<String, Box<dyn DataType>> = DashMap::new();
        registry.insert("i32".to_string(), Box::new(DataTypeI32 {}));

        registry
    }
}
