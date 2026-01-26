use crate::structures::data_values::data_value_interpretation_format::DataValueInterpretationFormat;
use crate::structures::data_values::data_value_interpreter::DataValueInterpreter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataValueInterpreters {
    data_value_interpreters: Vec<DataValueInterpreter>,
    default_data_value_interpretation_format: DataValueInterpretationFormat,
    active_data_value_interpretation_format: DataValueInterpretationFormat,
}

impl DataValueInterpreters {
    pub fn new(
        data_value_interpreters: Vec<DataValueInterpreter>,
        default_data_value_interpretation_format: DataValueInterpretationFormat,
    ) -> Self {
        let active_data_value_interpretation_format = default_data_value_interpretation_format.clone();

        Self {
            data_value_interpreters,
            default_data_value_interpretation_format,
            active_data_value_interpretation_format,
        }
    }

    pub fn set_active_data_value_interpretation_format(
        &mut self,
        active_data_value_interpretation_format: DataValueInterpretationFormat,
    ) {
        self.active_data_value_interpretation_format = active_data_value_interpretation_format
    }

    pub fn get_active_data_value_interpretation_format(&self) -> DataValueInterpretationFormat {
        self.active_data_value_interpretation_format
    }

    pub fn get_default_data_value_interpretation_format(&self) -> DataValueInterpretationFormat {
        self.default_data_value_interpretation_format
    }

    pub fn get_data_value_interpreters(&self) -> &Vec<DataValueInterpreter> {
        &self.data_value_interpreters
    }

    pub fn get_default_data_value_interpreter_string(&self) -> &str {
        self.get_data_value_interpreter_string(&self.default_data_value_interpretation_format)
    }

    pub fn get_default_data_value_interpreter(&self) -> Option<&DataValueInterpreter> {
        self.get_data_value_interpreter(&self.default_data_value_interpretation_format)
    }

    pub fn get_active_data_value_interpreter(&self) -> Option<&DataValueInterpreter> {
        self.get_data_value_interpreter(&self.active_data_value_interpretation_format)
    }

    pub fn get_data_value_interpreter_string(
        &self,
        data_value_interpretation_format: &DataValueInterpretationFormat,
    ) -> &str {
        for data_value_interpreter in &self.data_value_interpreters {
            if data_value_interpreter.get_data_value_interpretation_format() == data_value_interpretation_format {
                return data_value_interpreter.get_display_string();
            }
        }

        "??"
    }

    pub fn get_data_value_interpreter(
        &self,
        data_value_interpretation_format: &DataValueInterpretationFormat,
    ) -> Option<&DataValueInterpreter> {
        for data_value_interpreter in &self.data_value_interpreters {
            if data_value_interpreter.get_data_value_interpretation_format() == data_value_interpretation_format {
                return Some(data_value_interpreter);
            }
        }

        None
    }
}
