// Defines the conversion of model structs to/from slint view structs, automatically mapping necessary fields.
pub trait ViewDataConverter<From, To> {
    fn convert_to_view_data(from: &From) -> To;
    fn convert_from_view_data(to: &To) -> From;
}
