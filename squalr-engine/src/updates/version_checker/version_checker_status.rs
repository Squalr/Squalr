use crate::updates::version_checker::github_latest_version_info::GitHubLatestVersionInfo;

#[derive(Clone)]
pub enum VersionCheckerStatus {
    CheckingForVersions,
    Cancelled,
    LatestVersionFound(GitHubLatestVersionInfo),
    Error(String),
}
