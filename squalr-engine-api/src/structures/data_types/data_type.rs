use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::data_values::display_value::DisplayValue;
use crate::structures::memory::endian::Endian;
use std::fmt::Debug;

/// Defines a generic scannable data type. This is the primary trait for both built-in types and plugin-defined types.
pub trait DataType: Debug + Send + Sync + ScalarComparable + VectorComparable {
    fn get_data_type_id(&self) -> &str;

    fn get_icon_id(&self) -> &str;

    fn get_default_size_in_bytes(&self) -> u64;

    fn validate_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> bool;

    fn deanonymize_value(
        &self,
        anonymous_value: &AnonymousValue,
        data_type_ref: DataTypeRef,
    ) -> Result<DataValue, DataTypeError>;

    fn create_display_values(
        &self,
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
    ) -> Result<Vec<DisplayValue>, DataTypeError>;

    fn get_endian(&self) -> Endian;

    fn is_discrete(&self) -> bool;

    fn get_default_value(
        &self,
        data_type_ref: DataTypeRef,
    ) -> DataValue;

    fn get_default_meta_data(&self) -> DataTypeMetaData;

    fn get_meta_data_for_anonymous_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> DataTypeMetaData;

    fn get_meta_data_from_string(
        &self,
        string: &str,
    ) -> Result<DataTypeMetaData, String>;

    fn get_ref(&self) -> DataTypeRef {
        DataTypeRef::new(self.get_data_type_id(), self.get_default_meta_data())
    }
}
