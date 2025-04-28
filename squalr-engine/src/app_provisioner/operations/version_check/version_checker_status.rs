use crate::app_provisioner::operations::version_check::github_latest_version_info::GitHubLatestVersionInfo;

#[derive(Clone)]
pub enum VersionCheckerStatus {
    CheckingForVersions,
    Cancelled,
    LatestVersionFound(GitHubLatestVersionInfo),
    Error(String),
}
