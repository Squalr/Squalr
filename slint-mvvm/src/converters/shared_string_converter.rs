use crate::view_data_converter::ViewDataConverter;
use slint::SharedString;

pub struct SharedStringConverter;

/// Converts a string into a SharedString.
impl ViewDataConverter<String, SharedString> for SharedStringConverter {
    fn convert_to_view_data(string: &String) -> SharedString {
        SharedString::from(string)
    }

    fn convert_from_view_data(string: &SharedString) -> String {
        string.into()
    }
}
