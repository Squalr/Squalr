use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_container::AnonymousValueContainer;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::data_values::display_value_type::DisplayValueType;
use crate::structures::data_values::display_values::DisplayValues;
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
    fn validate_value(
        &self,
        anonymous_value_container: &AnonymousValueContainer,
    ) -> bool;

    /// Attempts to interpret an anonymous value as this data type, returning a `DataValue` on success.
    fn deanonymize_value(
        &self,
        anonymous_value_container: &AnonymousValueContainer,
    ) -> Result<DataValue, DataTypeError>;

    /// Creates all supported display values for this data types (ie bin/dec/hex).
    fn create_display_values(
        &self,
        value_bytes: &[u8],
    ) -> Result<DisplayValues, DataTypeError>;

    /// Gets all supported display types that this data type can be shown as.
    fn get_supported_display_types(&self) -> Vec<DisplayValueType>;

    /// Gets the default display type that this data type can be shown as.
    fn get_default_display_type(&self) -> DisplayValueType;

    /// Gets the endianness of this data type.
    fn get_endian(&self) -> Endian;

    /// Gets a value indicating whether this value is discrete, ie non-floating point.
    fn is_floating_point(&self) -> bool;

    /// Gets a value indicating whether this value is unsigned.
    fn is_signed(&self) -> bool;

    /// Gets a value indicating whether this scan should use byte array scans internally.
    /// For complex data types, this is almost always the case.
    // fn is_scan_remapped_to_byte_array(&self) -> bool;

    fn get_default_value(
        &self,
        data_type_ref: DataTypeRef,
    ) -> DataValue;

    fn get_ref(&self) -> DataTypeRef {
        DataTypeRef::new(self.get_data_type_id())
    }
}
