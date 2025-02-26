#[derive(PartialEq, Eq)]
pub enum PageRetrievalMode {
    FromSettings,
    FromUserMode,
    FromNonModules,
    FromModules,
}
