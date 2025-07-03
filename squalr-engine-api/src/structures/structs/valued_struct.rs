use crate::structures::data_values::data_value::DataValue;
use crate::structures::structs::symbolic_struct_ref::SymbolicStructRef;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValuedStruct {
    symbolic_struct_ref: SymbolicStructRef,
    values: Vec<DataValue>,
}

impl ValuedStruct {
    pub fn new(
        symbolic_struct_ref: SymbolicStructRef,
        values: Vec<DataValue>,
    ) -> Self {
        ValuedStruct { symbolic_struct_ref, values }
    }

    pub fn get_size_in_bytes(&self) -> u64 {
        self.values
            .iter()
            .map(|data_value| data_value.get_size_in_bytes())
            .sum()
    }

    pub fn copy_from_bytes(
        &mut self,
        bytes: &[u8],
    ) -> bool {
        let mut accumulated_size = 0u64;
        let total_size = bytes.len() as u64;
        let expected_size = self.get_size_in_bytes();

        if expected_size != expected_size {
            return false;
        }

        for data_value in self.values.iter_mut() {
            let next_size = data_value.get_size_in_bytes();

            if accumulated_size + next_size > expected_size {
                return false;
            }

            let accumulated_size_end = accumulated_size + next_size;
            data_value.copy_from_bytes(&bytes[accumulated_size as usize..accumulated_size_end as usize]);
            accumulated_size = accumulated_size_end as u64;
        }

        debug_assert!(accumulated_size == total_size);

        true
    }
}

impl FromStr for ValuedStruct {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let fields: Result<Vec<DataValue>, Self::Err> = string
            .split(';')
            .filter(|&data_value_string| !data_value_string.is_empty())
            .map(|data_value_string| DataValue::from_str(data_value_string))
            .collect();

        Ok(ValuedStruct::new(SymbolicStructRef::new("".to_string()), fields?))
    }
}
