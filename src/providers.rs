use serde::Deserialize;

use self::github::LatestReleaseProvider;

pub mod error;
pub mod github;

pub trait Release: std::fmt::Debug {
    fn version(&self) -> Option<&str>;
}

#[derive(Debug, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum Provider {
    LatestGithubRelease(LatestReleaseProvider),
}

impl Provider {
    pub async fn release(
        &self,
        http_client: &reqwest::Client,
    ) -> Result<Box<dyn Release>, Box<dyn std::error::Error>> {
        Ok(match self {
            Provider::LatestGithubRelease(provider) => Box::new(provider.fetch(http_client).await?),
        })
    }
}
