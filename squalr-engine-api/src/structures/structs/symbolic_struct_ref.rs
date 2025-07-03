use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolicStructRef {
    symbolic_struct_namespace: String,
}

impl SymbolicStructRef {
    pub fn new(symbolic_struct_namespace: String) -> Self {
        SymbolicStructRef { symbolic_struct_namespace }
    }
}

impl FromStr for SymbolicStructRef {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(SymbolicStructRef::new(string.to_string()))
    }
}
