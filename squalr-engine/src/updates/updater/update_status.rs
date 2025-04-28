use crate::updates::version_checker::version_checker_status::VersionCheckerStatus;

#[derive(Clone, PartialEq)]
pub enum UpdateStatus {
    CheckingVersion(VersionCheckerStatus),
    Cancelled,
    Downloading(f32),
    Installing(f32),
    Complete,
    Error(String),
}
