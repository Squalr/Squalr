use crate::structures::data_types::{
    built_in_types::{
        byte_array::data_type_byte_array::DataTypeByteArray, f32::data_type_f32::DataTypeF32, f32be::data_type_f32be::DataTypeF32be,
        f64::data_type_f64::DataTypeF64, f64be::data_type_f64be::DataTypeF64be, i8::data_type_i8::DataTypeI8, i16::data_type_i16::DataTypeI16,
        i16be::data_type_i16be::DataTypeI16be, i32::data_type_i32::DataTypeI32, i32be::data_type_i32be::DataTypeI32be, i64::data_type_i64::DataTypeI64,
        i64be::data_type_i64be::DataTypeI64be, u8::data_type_u8::DataTypeU8, u16::data_type_u16::DataTypeU16, u16be::data_type_u16be::DataTypeU16be,
        u32::data_type_u32::DataTypeU32, u32be::data_type_u32be::DataTypeU32be, u64::data_type_u64::DataTypeU64, u64be::data_type_u64be::DataTypeU64be,
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
        let data_type_i16be = DataTypeI16be {};
        let data_type_i32 = DataTypeI32 {};
        let data_type_i32be = DataTypeI32be {};
        let data_type_i64 = DataTypeI64 {};
        let data_type_i64be = DataTypeI64be {};
        let data_type_u8 = DataTypeU8 {};
        let data_type_u16 = DataTypeU16 {};
        let data_type_u16be = DataTypeU16be {};
        let data_type_u32 = DataTypeU32 {};
        let data_type_u32be = DataTypeU32be {};
        let data_type_u64 = DataTypeU64 {};
        let data_type_u64be = DataTypeU64be {};
        let data_type_f32 = DataTypeF32 {};
        let data_type_f32be = DataTypeF32be {};
        let data_type_f64 = DataTypeF64 {};
        let data_type_f64be = DataTypeF64be {};
        let data_type_byte_array = DataTypeByteArray {};

        registry.insert(data_type_i8.get_id().to_string(), Arc::new(data_type_i8));
        registry.insert(data_type_i16.get_id().to_string(), Arc::new(data_type_i16));
        registry.insert(data_type_i16be.get_id().to_string(), Arc::new(data_type_i16be));
        registry.insert(data_type_i32.get_id().to_string(), Arc::new(data_type_i32));
        registry.insert(data_type_i32be.get_id().to_string(), Arc::new(data_type_i32be));
        registry.insert(data_type_i64.get_id().to_string(), Arc::new(data_type_i64));
        registry.insert(data_type_i64be.get_id().to_string(), Arc::new(data_type_i64be));
        registry.insert(data_type_u8.get_id().to_string(), Arc::new(data_type_u8));
        registry.insert(data_type_u16.get_id().to_string(), Arc::new(data_type_u16));
        registry.insert(data_type_u16be.get_id().to_string(), Arc::new(data_type_u16be));
        registry.insert(data_type_u32.get_id().to_string(), Arc::new(data_type_u32));
        registry.insert(data_type_u32be.get_id().to_string(), Arc::new(data_type_u32be));
        registry.insert(data_type_u64.get_id().to_string(), Arc::new(data_type_u64));
        registry.insert(data_type_u64be.get_id().to_string(), Arc::new(data_type_u64be));
        registry.insert(data_type_f32.get_id().to_string(), Arc::new(data_type_f32));
        registry.insert(data_type_f32be.get_id().to_string(), Arc::new(data_type_f32be));
        registry.insert(data_type_f64.get_id().to_string(), Arc::new(data_type_f64));
        registry.insert(data_type_f64be.get_id().to_string(), Arc::new(data_type_f64be));
        registry.insert(data_type_byte_array.get_id().to_string(), Arc::new(data_type_byte_array));

        registry
    }
}
