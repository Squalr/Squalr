use crate::structures::data_types::{
    built_in_types::{
        i8::data_type_i8::DataTypeI8, i16::data_type_i16::DataTypeI16, i32::data_type_i32::DataTypeI32, i64::data_type_i64::DataTypeI64,
        u8::data_type_u8::DataTypeU8, u16::data_type_u16::DataTypeU16, u32::data_type_u32::DataTypeU32, u64::data_type_u64::DataTypeU64,
    },
    data_type::DataType,
};
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

        let data_type_i8 = DataTypeI8 {};
        let data_type_i16 = DataTypeI16 {};
        let data_type_i32 = DataTypeI32 {};
        let data_type_i64 = DataTypeI64 {};
        let data_type_u8 = DataTypeU8 {};
        let data_type_u16 = DataTypeU16 {};
        let data_type_u32 = DataTypeU32 {};
        let data_type_u64 = DataTypeU64 {};

        registry.insert(data_type_i8.get_id().to_string(), Arc::new(data_type_i8));
        registry.insert(data_type_i16.get_id().to_string(), Arc::new(data_type_i16));
        registry.insert(data_type_i32.get_id().to_string(), Arc::new(data_type_i32));
        registry.insert(data_type_i64.get_id().to_string(), Arc::new(data_type_i64));
        registry.insert(data_type_u8.get_id().to_string(), Arc::new(data_type_u8));
        registry.insert(data_type_u16.get_id().to_string(), Arc::new(data_type_u16));
        registry.insert(data_type_u32.get_id().to_string(), Arc::new(data_type_u32));
        registry.insert(data_type_u64.get_id().to_string(), Arc::new(data_type_u64));

        registry
    }
}
