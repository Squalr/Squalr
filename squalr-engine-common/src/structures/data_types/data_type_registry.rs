use crate::structures::data_types::built_in_types::i32::data_type_i32::DataTypeI32;
use crate::structures::data_types::data_type::DataType;
use std::collections::HashMap;

type DataTypeConstructor = Box<dyn Fn() -> Box<dyn DataType> + Send + Sync>;

pub struct DataTypeRegistry {}

impl DataTypeRegistry {
    pub fn get_registry() -> HashMap<&'static str, DataTypeConstructor> {
        let mut registry: HashMap<&'static str, DataTypeConstructor> = HashMap::new();
        registry.insert("i32", Box::new(|| Box::new(DataTypeI32 {})));
        registry
    }
}
