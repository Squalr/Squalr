use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValuedStructError {
    #[error("Cannot resolve symbolic struct for an anonymous valued struct.")]
    AnonymousStructReference,
    #[error("Struct symbol definition `{symbolic_struct_namespace}` is not registered.")]
    SymbolicStructNotRegistered { symbolic_struct_namespace: String },
}

impl ValuedStructError {
    pub fn symbolic_struct_not_registered(symbolic_struct_namespace: impl Into<String>) -> Self {
        Self::SymbolicStructNotRegistered {
            symbolic_struct_namespace: symbolic_struct_namespace.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ValuedStructError;

    #[test]
    fn symbolic_struct_not_registered_formats_namespace() {
        let error = ValuedStructError::symbolic_struct_not_registered("player.position");

        assert_eq!(error.to_string(), "Struct symbol definition `player.position` is not registered.");
    }
}
