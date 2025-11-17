#[derive(Copy, Clone, PartialEq)]
pub enum ElementScannerResultFrameAction {
    None,
    SetSelectionStart(Option<i32>),
    SetSelectionEnd(Option<i32>),
    FreezeIndex(i32, bool),
    FreezeSelection,
    AddSelection,
    DeleteSelection,
}
