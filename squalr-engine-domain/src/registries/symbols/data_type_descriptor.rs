use crate::structures::{data_types::data_type::DataType, data_values::anonymous_value_string_format::AnonymousValueStringFormat, memory::endian::Endian};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataTypeDescriptor {
    data_type_id: String,
    icon_id: String,
    unit_size_in_bytes: u64,
    supported_anonymous_value_string_formats: Vec<AnonymousValueStringFormat>,
    default_anonymous_value_string_format: AnonymousValueStringFormat,
    endian: Endian,
    is_floating_point: bool,
    is_signed: bool,
}

impl DataTypeDescriptor {
    pub fn new(
        data_type_id: String,
        icon_id: String,
        unit_size_in_bytes: u64,
        supported_anonymous_value_string_formats: Vec<AnonymousValueStringFormat>,
        default_anonymous_value_string_format: AnonymousValueStringFormat,
        endian: Endian,
        is_floating_point: bool,
        is_signed: bool,
    ) -> Self {
        Self {
            data_type_id,
            icon_id,
            unit_size_in_bytes,
            supported_anonymous_value_string_formats,
            default_anonymous_value_string_format,
            endian,
            is_floating_point,
            is_signed,
        }
    }

    pub fn from_data_type(data_type: &dyn DataType) -> Self {
        Self::new(
            data_type.get_data_type_id().to_string(),
            data_type.get_icon_id().to_string(),
            data_type.get_unit_size_in_bytes(),
            data_type.get_supported_anonymous_value_string_formats(),
            data_type.get_default_anonymous_value_string_format(),
            data_type.get_endian(),
            data_type.is_floating_point(),
            data_type.is_signed(),
        )
    }

    pub fn get_data_type_id(&self) -> &str {
        &self.data_type_id
    }

    pub fn get_icon_id(&self) -> &str {
        &self.icon_id
    }

    pub fn get_unit_size_in_bytes(&self) -> u64 {
        self.unit_size_in_bytes
    }

    pub fn get_supported_anonymous_value_string_formats(&self) -> &[AnonymousValueStringFormat] {
        &self.supported_anonymous_value_string_formats
    }

    pub fn get_default_anonymous_value_string_format(&self) -> AnonymousValueStringFormat {
        self.default_anonymous_value_string_format
    }

    pub fn get_endian(&self) -> &Endian {
        &self.endian
    }

    pub fn get_is_floating_point(&self) -> bool {
        self.is_floating_point
    }

    pub fn get_is_signed(&self) -> bool {
        self.is_signed
    }
}
