use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::endian::Endian;
use std::fmt::Debug;

/// Defines a generic scannable data type. This is the primary trait for both built-in types and plugin-defined types.
pub trait DataType: Debug + Send + Sync + ScalarComparable + VectorComparable {
    fn get_id(&self) -> &str;
    fn get_icon_id(&self) -> &str;
    fn get_size_in_bytes(&self) -> u64;
    fn get_endian(&self) -> Endian;
    fn get_default_value(&self) -> Box<dyn DataValue>;
}
