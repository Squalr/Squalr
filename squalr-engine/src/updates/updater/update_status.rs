use semver::Version;

#[derive(Clone, PartialEq)]
pub enum UpdateStatus {
    CheckingForUpdates,
    Cancelled,
    UpdateAvailable(Version),
    NoUpdateRequired,
    Downloading(f32),
    Installing(f32),
    Complete,
    Error(String),
}
