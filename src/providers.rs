use std::collections::HashMap;

use serde::Deserialize;

use self::github::LatestReleaseProvider;

pub mod error;
pub mod github;
pub mod prometheus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionInfo {
    version: Option<String>,
    labels: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum Provider {
    LatestGithubRelease(LatestReleaseProvider),
    Prometheus(prometheus::Provider),
}

impl Provider {
    pub async fn versions(
        &self,
        http_client: &reqwest::Client,
    ) -> Result<Vec<VersionInfo>, Box<dyn std::error::Error>> {
        Ok(match self {
            Provider::LatestGithubRelease(provider) => {
                vec![provider.fetch(http_client).await?.into()]
            }
            Provider::Prometheus(provider) => provider.fetch(http_client).await?,
        })
    }
}
