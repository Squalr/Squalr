use crate::structures::{
    data_types::data_type_ref::DataTypeRef, data_values::data_value::DataValue, structs::symbolic_struct_definition::SymbolicStructDefinition,
};
use std::sync::Arc;

/// Resolves symbolic structs and data type defaults for struct materialization operations.
pub trait SymbolResolver {
    fn get_default_value(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Option<DataValue>;

    fn get_unit_size_in_bytes(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> u64;

    fn get_symbolic_struct(
        &self,
        symbolic_struct_namespace: &str,
    ) -> Option<Arc<SymbolicStructDefinition>>;
}
