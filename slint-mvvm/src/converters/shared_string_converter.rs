use crate::view_data_converter::ViewDataConverter;
use slint::SharedString;

pub struct SharedStringConverter;

impl SharedStringConverter {
    pub fn new() -> Self {
        Self {}
    }
}

/// Converts a string into a SharedString.
impl ViewDataConverter<String, SharedString> for SharedStringConverter {
    fn convert_collection(
        &self,
        strings: &Vec<String>,
    ) -> Vec<SharedString> {
        return strings
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect();
    }

    fn convert_to_view_data(
        &self,
        string: &String,
    ) -> SharedString {
        SharedString::from(string)
    }

    fn convert_from_view_data(
        &self,
        string: &SharedString,
    ) -> String {
        string.into()
    }
}
