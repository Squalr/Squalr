#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageRetrievalMode {
    FromSettings,
    FromUserMode,
    FromNonModules,
    FromModules,
}
