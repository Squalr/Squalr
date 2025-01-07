pub trait ViewDataConverter<From, To> {
    fn convert(
        &self,
        from: From,
    ) -> To;
}
