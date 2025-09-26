use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SymbolicStructRef {
    symbolic_struct_namespace: String,
}

impl SymbolicStructRef {
    pub fn new(symbolic_struct_namespace: String) -> Self {
        SymbolicStructRef { symbolic_struct_namespace }
    }

    pub fn new_anonymous() -> Self {
        SymbolicStructRef {
            symbolic_struct_namespace: String::new(),
        }
    }

    pub fn get_symbolic_struct_namespace(&self) -> &str {
        &self.symbolic_struct_namespace
    }
}

impl FromStr for SymbolicStructRef {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(SymbolicStructRef::new(string.to_string()))
    }
}

impl fmt::Display for SymbolicStructRef {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}", self.symbolic_struct_namespace)
    }
}
