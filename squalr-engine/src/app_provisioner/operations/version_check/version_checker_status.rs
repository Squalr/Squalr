use crate::app_provisioner::operations::version_check::github_release_info::GitHubReleaseInfo;

#[derive(Clone)]
pub enum VersionCheckerStatus {
    CheckingForVersions,
    Cancelled,
    LatestVersionFound(GitHubReleaseInfo),
    Error(String),
}
