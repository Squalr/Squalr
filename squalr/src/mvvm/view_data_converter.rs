// Defines the conversion of model structs into slint view structs, automatically mapping necessary fields.
pub trait ViewDataConverter<From, To> {
    fn convert(
        &self,
        from: From,
    ) -> To;
}
