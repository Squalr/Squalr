// Defines the conversion of model structs to slint view structs, automatically mapping necessary fields.
pub trait ConvertToViewData<From, To> {
    fn convert_collection(
        &self,
        from: &Vec<From>,
    ) -> Vec<To>;
    fn convert_to_view_data(
        &self,
        from: &From,
    ) -> To;
}
