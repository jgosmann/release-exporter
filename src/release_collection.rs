use std::collections::HashMap;

use futures::StreamExt;
use reqwest::Client;
use tokio_stream::{self as stream};

use crate::providers::{Provider, VersionInfo};

pub struct ReleaseCollection {
    pub releases: HashMap<String, Vec<VersionInfo>>,
    pub errors: HashMap<String, Box<dyn std::error::Error>>,
}

impl ReleaseCollection {
    pub async fn collect_from(providers: &[Provider], http_client: &Client) -> Self {
        let results: HashMap<_, _> = stream::iter(providers)
            .map(|p| async { (p.name().to_string(), p.versions(http_client).await) })
            .buffer_unordered(10)
            .collect()
            .await;
        let (releases, errors): (HashMap<_, _>, HashMap<_, _>) =
            results.into_iter().partition(|item| item.1.is_ok());
        Self {
            releases: releases.into_iter().map(|(k, v)| (k, v.unwrap())).collect(),
            errors: errors
                .into_iter()
                .map(|(k, v)| (k, v.unwrap_err()))
                .collect(),
        }
    }
}
