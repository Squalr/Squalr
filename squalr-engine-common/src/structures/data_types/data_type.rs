use crate::structures::data_types::comparisons::scalar_comparable::ScalarComparable;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_values::anonymous_value::AnonymousValue;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::endian::Endian;
use std::fmt::Debug;

use super::data_type_ref::DataTypeRef;

/// Defines a generic scannable data type. This is the primary trait for both built-in types and plugin-defined types.
pub trait DataType: Debug + Send + Sync + ScalarComparable + VectorComparable {
    fn get_id(&self) -> &str;

    fn get_icon_id(&self) -> &str;

    fn get_default_size_in_bytes(&self) -> u64;

    fn deanonymize_value(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> Vec<u8>;

    /// Deanonymizes the given `AnonymousValue`, but forces the type to be `Endian::Little` by reversing the bytes if necessary.
    fn deanonymize_value_little_endian(
        &self,
        anonymous_value: &AnonymousValue,
    ) -> Vec<u8> {
        let mut value: Vec<u8> = self.deanonymize_value(anonymous_value);

        if self.get_endian() == Endian::Big {
            value.reverse();
        }

        value
    }

    fn create_display_value(
        &self,
        value_bytes: &[u8],
    ) -> Option<String>;

    fn get_endian(&self) -> Endian;

    fn get_default_value(&self) -> DataValue;

    fn get_ref(&self) -> DataTypeRef {
        DataTypeRef::new(self.get_id())
    }
}
