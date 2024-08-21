use crate::values::data_type::DataType;
use crate::values::data_value::DataValue;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct AnonymousValue {
    value_str: String,
}

impl AnonymousValue {
    pub fn new(
        value: &str
    ) -> Self {
        AnonymousValue {
            value_str: value.to_string(),
        }
    }

    pub fn deanonymize_type(
        &self,
        target_type: &DataType,
    ) -> Result<DataValue, String> {
        let value_and_type_str = format!("{}={}", self.value_str, target_type);

        return value_and_type_str.parse::<DataValue>();
    }
}

impl FromStr for AnonymousValue {
    type Err = String;

    fn from_str(
        s: &str
    ) -> Result<Self, Self::Err> {
        return Ok(AnonymousValue::new(s));
    }
}
