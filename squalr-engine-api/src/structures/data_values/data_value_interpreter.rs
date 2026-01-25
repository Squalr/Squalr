use crate::structures::data_values::{container_type::ContainerType, data_value_interpretation_format::DataValueInterpretationFormat};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataValueInterpreter {
    display_string: String,
    data_value_interpretation_format: DataValueInterpretationFormat,
    container_type: ContainerType,
}

impl DataValueInterpreter {
    pub fn new(
        display_string: String,
        data_value_interpretation_format: DataValueInterpretationFormat,
        container_type: ContainerType,
    ) -> Self {
        Self {
            display_string,
            data_value_interpretation_format,
            container_type,
        }
    }

    pub fn get_data_value_interpretation_format(&self) -> &DataValueInterpretationFormat {
        &self.data_value_interpretation_format
    }

    pub fn set_data_value_interpretation_format(
        &mut self,
        data_value_interpreter: DataValueInterpretationFormat,
    ) {
        self.data_value_interpretation_format = data_value_interpreter
    }

    pub fn get_container_type(&self) -> &ContainerType {
        &self.container_type
    }

    pub fn get_display_string(&self) -> &str {
        &self.display_string
    }

    pub fn set_display_string(
        &mut self,
        display_string: String,
    ) {
        self.display_string = display_string;
    }
}

impl fmt::Display for DataValueInterpreter {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            formatter,
            "{}{}: {}",
            self.data_value_interpretation_format, self.container_type, self.display_string
        )
    }
}
