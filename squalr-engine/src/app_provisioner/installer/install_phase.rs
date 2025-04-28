#[derive(Clone, Copy, PartialEq)]
pub enum InstallPhase {
    Download,
    Extraction,
    Complete,
}
