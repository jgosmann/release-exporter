use std::{collections::HashMap, time::Duration};

use futures::StreamExt;
use reqwest::Client;
use tokio_stream::{self as stream};

use crate::providers::{Provider, VersionInfo};

pub struct Release {
    pub versions: Vec<VersionInfo>,
    pub cache_duration: Duration,
}

pub struct ReleaseCollection {
    pub releases: HashMap<String, Release>,
    pub errors: HashMap<String, Box<dyn std::error::Error + Send + Sync>>,
}

impl ReleaseCollection {
    pub async fn collect_from(providers: Vec<Provider>, http_client: &Client) -> Self {
        let results: HashMap<_, _> = stream::iter(providers)
            .map(|p| async move {
                (
                    p.name().to_string(),
                    (p.versions(http_client).await, *p.cache_duration()),
                )
            })
            .buffer_unordered(10)
            .collect()
            .await;
        let (releases, errors): (HashMap<_, _>, HashMap<_, _>) =
            results.into_iter().partition(|(_, (r, _))| r.is_ok());
        Self {
            releases: releases
                .into_iter()
                .map(|(k, (v, cache_duration))| {
                    (
                        k,
                        Release {
                            versions: v.unwrap(),
                            cache_duration,
                        },
                    )
                })
                .collect(),
            errors: errors
                .into_iter()
                .map(|(k, (v, _))| (k, v.unwrap_err()))
                .collect(),
        }
    }
}
