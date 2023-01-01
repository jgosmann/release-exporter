use std::collections::HashMap;
use std::time::Duration;

use serde::{de::Visitor, Deserialize, Deserializer};

use self::github::LatestReleaseProvider;

pub mod error;
pub mod github;
pub mod prometheus;

struct DurationSecsVisitor;

impl<'de> Visitor<'de> for DurationSecsVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a non-negative number of seconds")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Duration::from_secs(v))
    }
}

fn deserialize_duration_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_u64(DurationSecsVisitor)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionInfo {
    pub version: Option<String>,
    pub labels: HashMap<String, String>,
}

fn default_github_cache_duration() -> Duration {
    Duration::from_secs(4 * 60 * 60)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum Provider {
    LatestGithubRelease {
        #[serde(flatten)]
        config: LatestReleaseProvider,
        name: String,
        #[serde(
            rename = "cache_seconds",
            deserialize_with = "deserialize_duration_secs",
            default = "default_github_cache_duration"
        )]
        cache_duration: Duration,
    },
    Prometheus {
        #[serde(flatten)]
        config: prometheus::Provider,
        name: String,
        #[serde(
            rename = "cache_seconds",
            deserialize_with = "deserialize_duration_secs",
            default = "Duration::default"
        )]
        cache_duration: Duration,
    },
}

impl Provider {
    pub fn name(&self) -> &str {
        match self {
            Provider::LatestGithubRelease {
                config: _,
                name,
                cache_duration: _,
            } => name,
            Provider::Prometheus {
                config: _,
                name,
                cache_duration: _,
            } => name,
        }
    }

    pub fn cache_duration(&self) -> &Duration {
        match self {
            Provider::LatestGithubRelease {
                config: _,
                name: _,
                cache_duration,
            } => cache_duration,
            Provider::Prometheus {
                config: _,
                name: _,
                cache_duration,
            } => cache_duration,
        }
    }

    pub async fn versions(
        &self,
        http_client: &reqwest::Client,
    ) -> Result<Vec<VersionInfo>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(match self {
            Provider::LatestGithubRelease {
                config,
                name: _,
                cache_duration: _,
            } => {
                vec![config.fetch(http_client).await?.into()]
            }
            Provider::Prometheus {
                config,
                name: _,
                cache_duration: _,
            } => config.fetch(http_client).await?,
        })
    }
}
