#[derive(Copy, Clone, PartialEq)]
pub enum ElementScannerResultFrameAction {
    None,
    SetSelectionStart(i32),
    SetSelectionEnd(i32),
    FreezeIndex(i32, bool),
    FreezeSelection,
    AddSelection,
    DeleteSelection,
}
