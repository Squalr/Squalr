// Defines the conversion of model structs into slint view structs, automatically mapping necessary fields.
pub trait ViewDataComparer<ViewType> {
    fn compare(
        a: &ViewType,
        b: &ViewType,
    ) -> bool;
}
