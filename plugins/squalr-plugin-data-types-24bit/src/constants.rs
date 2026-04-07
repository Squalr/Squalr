use crate::{DataTypeI24, DataTypeI24be, DataTypeU24, DataTypeU24be};

pub const TWENTY_FOUR_BIT_PLUGIN_ID: &str = "builtin.data-type.24bit-integers";
pub const TWENTY_FOUR_BIT_PLUGIN_DISPLAY_NAME: &str = "24-bit Integers";
pub const TWENTY_FOUR_BIT_PLUGIN_DESCRIPTION: &str = "Adds u24, u24be, i24, and i24be scan and formatting support.";
pub const TWENTY_FOUR_BIT_DATA_TYPE_IDS: [&str; 4] = [
    DataTypeU24::DATA_TYPE_ID,
    DataTypeU24be::DATA_TYPE_ID,
    DataTypeI24::DATA_TYPE_ID,
    DataTypeI24be::DATA_TYPE_ID,
];
