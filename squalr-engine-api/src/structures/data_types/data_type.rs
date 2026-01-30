use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_string::AnonymousValueString;
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::endian::Endian;
use std::fmt::Debug;

/// Defines a generic scannable data type. This is the primary trait for both built-in types and plugin-defined types.
pub trait DataType: Debug + Send + Sync + ScalarComparable + VectorComparable {
    /// Gets the unique identifier for this data type.
    fn get_data_type_id(&self) -> &str;

    /// Gets the identifier for the icon associated with this data type.
    fn get_icon_id(&self) -> &str;

    /// Gets the default size of this data type. For variable sized types, this is often 1.
    fn get_unit_size_in_bytes(&self) -> u64;

    /// Determines if an anonymous value can be interpreted as this data type.
    fn validate_value_string(
        &self,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool;

    /// Attempts to interpret an anonymous value as this data type, returning a `DataValue` on success.
    fn deanonymize_value_string(
        &self,
        anonymous_value_string: &AnonymousValueString,
    ) -> Result<DataValue, DataTypeError>;

    /// Attempts to interpret raw bytes as this data type in the specified format, returning an `AnonymousValueString` on success.
    /// In other words, this converts bytes in this data type to a plaintext string representation.
    fn anonymize_value_bytes(
        &self,
        value_bytes: &[u8],
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, DataTypeError>;

    /// Gets all supported display formats that this data type can be shown as.
    fn get_supported_anonymous_value_string_formats(&self) -> Vec<AnonymousValueStringFormat>;

    /// Gets the default display format that this data type can be shown as.
    fn get_default_anonymous_value_string_format(&self) -> AnonymousValueStringFormat;

    /// Gets the endianness of this data type.
    fn get_endian(&self) -> Endian;

    /// Gets a value indicating whether this value is discrete, ie non-floating point.
    fn is_floating_point(&self) -> bool;

    /// Gets a value indicating whether this value is unsigned.
    fn is_signed(&self) -> bool;

    fn get_default_value(
        &self,
        data_type_ref: DataTypeRef,
    ) -> DataValue;

    fn get_ref(&self) -> DataTypeRef {
        DataTypeRef::new(self.get_data_type_id())
    }
}
