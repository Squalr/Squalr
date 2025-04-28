use crate::app_provisioner::operations::version_check::version_checker_status::VersionCheckerStatus;

#[derive(Clone)]
pub enum UpdateStatus {
    CheckingVersion(VersionCheckerStatus),
    Cancelled,
    Downloading(f32),
    Installing(f32),
    Complete,
    Error(String),
}
