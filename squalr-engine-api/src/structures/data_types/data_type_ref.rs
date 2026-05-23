use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    str::FromStr,
};

/// Represents a handle to a data type. This is kept as a weak reference, as DataTypes can be registered/unregistered by plugins.
/// As such, `DataType` is a `Box<dyn>` type, so it is much easier to abstract them behind `DataTypeRef` and just pass around handles.
/// This is also important for serialization/deserialization, as if a plugin that defines a type is disabled, we can still deserialize it.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataTypeRef {
    data_type_id: String,
}

impl DataTypeRef {
    /// Creates a new reference to a registered `DataType` with the explicit.
    pub fn new(data_type_id: &str) -> Self {
        Self {
            data_type_id: data_type_id.to_string(),
        }
    }

    pub fn get_data_type_id(&self) -> &str {
        &self.data_type_id
    }

    pub fn get_base_data_type_id(&self) -> &str {
        self.data_type_id
            .split_once('{')
            .map(|(base_data_type_id, _)| base_data_type_id.trim())
            .filter(|base_data_type_id| !base_data_type_id.is_empty())
            .unwrap_or_else(|| self.data_type_id.trim())
    }

    pub fn has_flag(
        &self,
        flag_name: &str,
    ) -> bool {
        let Some(parameter_text) = self
            .data_type_id
            .split_once('{')
            .and_then(|(_, parameter_text)| parameter_text.strip_suffix('}'))
        else {
            return false;
        };

        parameter_text
            .split(',')
            .map(str::trim)
            .any(|parameter_name| parameter_name == flag_name)
    }

    pub fn with_flag(
        &self,
        flag_name: &str,
    ) -> Self {
        if self.has_flag(flag_name) {
            return self.clone();
        }

        let trimmed_data_type_id = self.data_type_id.trim();
        let updated_data_type_id = if let Some((base_data_type_id, parameter_text)) = trimmed_data_type_id.split_once('{') {
            let existing_flags = parameter_text.trim_end_matches('}').trim();

            if existing_flags.is_empty() {
                format!("{base_data_type_id}{{{flag_name}}}")
            } else {
                format!("{base_data_type_id}{{{existing_flags}, {flag_name}}}")
            }
        } else {
            format!("{trimmed_data_type_id}{{{flag_name}}}")
        };

        Self {
            data_type_id: updated_data_type_id,
        }
    }
}

impl Hash for DataTypeRef {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.data_type_id.hash(state);
    }
}

impl FromStr for DataTypeRef {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let data_type_id = string;

        Ok(DataTypeRef::new(data_type_id))
    }
}

impl fmt::Display for DataTypeRef {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "{}", self.get_data_type_id())
    }
}

#[cfg(test)]
mod tests {
    use super::DataTypeRef;

    #[test]
    fn get_base_data_type_id_strips_parameter_flags() {
        let data_type_ref = DataTypeRef::new("string_utf8{null_terminated}");

        assert_eq!(data_type_ref.get_base_data_type_id(), "string_utf8");
    }

    #[test]
    fn has_flag_detects_parameter_flag() {
        let data_type_ref = DataTypeRef::new("string_utf8{null_terminated, ascii_safe}");

        assert!(data_type_ref.has_flag("null_terminated"));
        assert!(data_type_ref.has_flag("ascii_safe"));
        assert!(!data_type_ref.has_flag("missing"));
    }

    #[test]
    fn with_flag_appends_flag_without_duplication() {
        let data_type_ref = DataTypeRef::new("string_utf8");
        let updated_data_type_ref = data_type_ref.with_flag("null_terminated");

        assert_eq!(updated_data_type_ref.get_data_type_id(), "string_utf8{null_terminated}");
        assert_eq!(
            updated_data_type_ref
                .with_flag("null_terminated")
                .get_data_type_id(),
            "string_utf8{null_terminated}"
        );
    }
}
