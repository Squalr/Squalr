use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct GitHubLatestVersionInfo {
    pub tag_name: String,
}
