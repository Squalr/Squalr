// Defines the conversion of model structs from slint view structs, automatically mapping necessary fields.
pub trait ConvertFromViewData<From, To> {
    fn convert_from_view_data(
        &self,
        to: &To,
    ) -> From;
}
