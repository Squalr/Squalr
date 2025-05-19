use crate::convert_from_view_data::ConvertFromViewData;
use crate::convert_to_view_data::ConvertToViewData;
use slint::SharedString;

pub struct SharedStringConverter;

impl SharedStringConverter {
    pub fn new() -> Self {
        Self {}
    }
}

/// Converts a string into a SharedString.
impl ConvertToViewData<String, SharedString> for SharedStringConverter {
    fn convert_collection(
        &self,
        strings: &Vec<String>,
    ) -> Vec<SharedString> {
        strings
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        string: &String,
    ) -> SharedString {
        SharedString::from(string)
    }
}

/// Converts a SharedString into a string.
impl ConvertFromViewData<String, SharedString> for SharedStringConverter {
    fn convert_from_view_data(
        &self,
        string: &SharedString,
    ) -> String {
        string.into()
    }
}
