use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstructionSyntaxError {
    message: String,
}

impl InstructionSyntaxError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for InstructionSyntaxError {
    fn fmt(
        &self,
        formatter: &mut Formatter<'_>,
    ) -> FmtResult {
        formatter.write_str(self.message())
    }
}

impl Error for InstructionSyntaxError {}
