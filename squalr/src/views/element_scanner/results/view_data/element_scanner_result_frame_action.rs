use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;

#[derive(Clone, PartialEq)]
pub enum ElementScannerResultFrameAction {
    None,
    SetSelectionStart(Option<i32>),
    SetSelectionEnd(Option<i32>),
    FreezeIndex(i32, bool),
    ToggleFreezeSelection(bool),
    AddSelection,
    AddScanResult(i32),
    DeleteSelection,
    CommitValueToSelection(AnonymousValueString),
}
