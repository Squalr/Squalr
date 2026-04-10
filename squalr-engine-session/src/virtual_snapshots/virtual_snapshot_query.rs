use squalr_engine_api::structures::{memory::pointer::Pointer, structs::symbolic_struct_definition::SymbolicStructDefinition};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VirtualSnapshotQuery {
    Address {
        query_id: String,
        address: u64,
        module_name: String,
        symbolic_struct_definition: SymbolicStructDefinition,
    },
    Pointer {
        query_id: String,
        pointer: Pointer,
        symbolic_struct_definition: SymbolicStructDefinition,
    },
}

impl VirtualSnapshotQuery {
    pub fn get_query_id(&self) -> &str {
        match self {
            Self::Address { query_id, .. } | Self::Pointer { query_id, .. } => query_id,
        }
    }

    pub fn get_symbolic_struct_definition(&self) -> &SymbolicStructDefinition {
        match self {
            Self::Address {
                symbolic_struct_definition, ..
            }
            | Self::Pointer {
                symbolic_struct_definition, ..
            } => symbolic_struct_definition,
        }
    }
}
