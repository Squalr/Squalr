use crate::structures::data_types::{
    built_in_types::{
        bool8::data_type_bool8::DataTypeBool8, bool32::data_type_bool32::DataTypeBool32, data_type_ref::data_type_data_type_ref::DataTypeRefDataType,
        f32::data_type_f32::DataTypeF32, f32be::data_type_f32be::DataTypeF32be, f64::data_type_f64::DataTypeF64, f64be::data_type_f64be::DataTypeF64be,
        i8::data_type_i8::DataTypeI8, i16::data_type_i16::DataTypeI16, i16be::data_type_i16be::DataTypeI16be, i32::data_type_i32::DataTypeI32,
        i32be::data_type_i32be::DataTypeI32be, i64::data_type_i64::DataTypeI64, i64be::data_type_i64be::DataTypeI64be,
        string::utf8::data_type_string_utf8::DataTypeStringUtf8, u8::data_type_u8::DataTypeU8, u16::data_type_u16::DataTypeU16,
        u16be::data_type_u16be::DataTypeU16be, u32::data_type_u32::DataTypeU32, u32be::data_type_u32be::DataTypeU32be, u64::data_type_u64::DataTypeU64,
        u64be::data_type_u64be::DataTypeU64be,
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

        let built_in_data_types: Vec<Arc<dyn DataType>> = vec![
            Arc::new(DataTypeRefDataType {}),
            Arc::new(DataTypeBool8 {}),
            Arc::new(DataTypeBool32 {}),
            Arc::new(DataTypeI8 {}),
            Arc::new(DataTypeI16 {}),
            Arc::new(DataTypeI16be {}),
            Arc::new(DataTypeI32 {}),
            Arc::new(DataTypeI32be {}),
            Arc::new(DataTypeI64 {}),
            Arc::new(DataTypeI64be {}),
            Arc::new(DataTypeU8 {}),
            Arc::new(DataTypeU16 {}),
            Arc::new(DataTypeU16be {}),
            Arc::new(DataTypeU32 {}),
            Arc::new(DataTypeU32be {}),
            Arc::new(DataTypeU64 {}),
            Arc::new(DataTypeU64be {}),
            Arc::new(DataTypeF32 {}),
            Arc::new(DataTypeF32be {}),
            Arc::new(DataTypeF64 {}),
            Arc::new(DataTypeF64be {}),
            Arc::new(DataTypeStringUtf8 {}),
        ];

        for built_in_data_type in built_in_data_types.into_iter() {
            registry.insert(built_in_data_type.get_data_type_id().to_string(), built_in_data_type);
        }

        registry
    }
}
