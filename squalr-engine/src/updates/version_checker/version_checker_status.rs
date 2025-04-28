use semver::Version;

#[derive(Clone, PartialEq)]
pub enum VersionCheckerStatus {
    CheckingForVersions,
    Cancelled,
    LatestVersionFound(Version),
    Error(String),
}
