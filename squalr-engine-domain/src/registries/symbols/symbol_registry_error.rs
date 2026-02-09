use crate::structures::data_types::data_type_error::DataTypeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SymbolRegistryError {
    #[error("Cannot {operation_context}: data type `{data_type_id}` is not registered.")]
    DataTypeNotRegistered { operation_context: &'static str, data_type_id: String },
    #[error("Failed while {operation_context}: {source}.")]
    DataTypeOperationFailed {
        operation_context: &'static str,
        #[source]
        source: DataTypeError,
    },
}

impl SymbolRegistryError {
    pub fn data_type_not_registered(
        operation_context: &'static str,
        data_type_id: impl Into<String>,
    ) -> Self {
        Self::DataTypeNotRegistered {
            operation_context,
            data_type_id: data_type_id.into(),
        }
    }

    pub fn data_type_operation_failed(
        operation_context: &'static str,
        source: DataTypeError,
    ) -> Self {
        Self::DataTypeOperationFailed { operation_context, source }
    }
}

#[cfg(test)]
mod tests {
    use super::SymbolRegistryError;

    #[test]
    fn data_type_not_registered_includes_operation_and_type_id() {
        let error = SymbolRegistryError::data_type_not_registered("deanonymizing value string", "u128");

        assert_eq!(error.to_string(), "Cannot deanonymizing value string: data type `u128` is not registered.");
    }
}
