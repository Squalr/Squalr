use squalr_engine_api::structures::data_values::anonymous_value::AnonymousValue;

#[derive(Clone, PartialEq)]
pub enum ElementScannerResultFrameAction {
    None,
    SetSelectionStart(Option<i32>),
    SetSelectionEnd(Option<i32>),
    FreezeIndex(i32, bool),
    ToggleFreezeSelection(bool),
    AddSelection,
    DeleteSelection,
    CommitValueToSelection(AnonymousValue),
}
