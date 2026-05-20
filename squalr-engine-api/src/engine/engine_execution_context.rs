use crate::{
    engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings,
    registries::symbols::{data_type_descriptor::DataTypeDescriptor, symbol_registry_error::SymbolRegistryError},
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, data_value::DataValue},
        projects::project_context::ProjectContext,
        structs::symbolic_struct_definition::SymbolicStructDefinition,
    },
};
use std::sync::{Arc, RwLock};

/// Abstraction for unprivileged session state required by command dispatch/execution paths.
pub trait EngineExecutionContext: Send + Sync {
    /// Gets the engine bindings used to dispatch privileged and unprivileged commands.
    fn get_bindings(&self) -> &Arc<RwLock<dyn EngineApiUnprivilegedBindings>>;

    /// Gets the project context owned by the interactive unprivileged session.
    fn get_project_manager(&self) -> Arc<dyn ProjectContext>;

    /// Gets the registered data type references.
    fn get_registered_data_type_refs(&self) -> Vec<DataTypeRef>;

    /// Gets the default display format for a value type.
    fn get_default_anonymous_value_string_format(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> AnonymousValueStringFormat;

    /// Formats a typed value into the requested anonymous string format.
    fn anonymize_value(
        &self,
        data_value: &DataValue,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> Result<AnonymousValueString, SymbolRegistryError>;

    /// Parses an anonymous string value into a typed data value.
    fn deanonymize_value_string(
        &self,
        data_type_ref: &DataTypeRef,
        anonymous_value_string: &AnonymousValueString,
    ) -> Result<DataValue, SymbolRegistryError>;

    /// Creates the default value for a data type.
    fn get_default_value(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Option<DataValue>;

    /// Gets the registered descriptor for a data type.
    fn get_data_type_descriptor(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Option<DataTypeDescriptor>;

    /// Gets the registered unit size for a data type.
    fn get_unit_size_in_bytes(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> u64;

    /// Resolves a struct layout definition by namespace.
    fn resolve_struct_layout_definition(
        &self,
        symbolic_struct_ref_id: &str,
    ) -> Option<SymbolicStructDefinition>;
}
