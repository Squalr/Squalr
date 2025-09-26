use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct GitHubReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: Option<bool>,
    pub prerelease: Option<bool>,
    pub created_at: Option<String>,
    pub published_at: Option<String>,
    pub assets: Option<Vec<GitHubReleaseAsset>>,
}

#[derive(Clone, Deserialize)]
pub struct GitHubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}
