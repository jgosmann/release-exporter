use std::collections::HashMap;

use serde::Deserialize;

use self::github::LatestReleaseProvider;

pub mod error;
pub mod github;
pub mod prometheus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionInfo {
    pub version: Option<String>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum Provider {
    LatestGithubRelease {
        #[serde(flatten)]
        config: LatestReleaseProvider,
        name: String,
    },
    Prometheus {
        #[serde(flatten)]
        config: prometheus::Provider,
        name: String,
    },
}

impl Provider {
    pub fn name(&self) -> &str {
        match self {
            Provider::LatestGithubRelease { config: _, name } => name,
            Provider::Prometheus { config: _, name } => name,
        }
    }

    pub async fn versions(
        &self,
        http_client: &reqwest::Client,
    ) -> Result<Vec<VersionInfo>, Box<dyn std::error::Error>> {
        Ok(match self {
            Provider::LatestGithubRelease { config, name: _ } => {
                vec![config.fetch(http_client).await?.into()]
            }
            Provider::Prometheus { config, name: _ } => config.fetch(http_client).await?,
        })
    }
}
