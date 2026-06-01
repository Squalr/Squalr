use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebuggerRegisterValue {
    name: String,
    value: u64,
    bit_width: u16,
}

impl DebuggerRegisterValue {
    pub fn new(
        name: impl Into<String>,
        value: u64,
        bit_width: u16,
    ) -> Self {
        Self {
            name: name.into(),
            value,
            bit_width,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self) -> u64 {
        self.value
    }

    pub fn get_bit_width(&self) -> u16 {
        self.bit_width
    }
}
