use std::collections::HashMap;

use reqwest::Url;
use serde::Deserialize;

use super::VersionInfo;

fn default_prometheus_url() -> Url {
    Url::parse("http://localhost:9090/api/").unwrap()
}

fn default_version_label() -> String {
    "version".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct Provider {
    query: String,

    #[serde(default = "default_version_label")]
    label: String,

    #[serde(default = "default_prometheus_url", with = "crate::serde_url")]
    api_url: Url,
}

#[derive(Clone, Debug, Deserialize)]
struct QueryResponse {
    data: QueryResponseData,
}

#[derive(Clone, Debug, Deserialize)]
struct QueryResponseData {
    result: Vec<QueryResultItem>,
}

#[derive(Clone, Debug, Deserialize)]
struct QueryResultItem {
    metric: Metric,
}

#[derive(Clone, Debug, Deserialize)]
struct Metric {
    __name__: String,
    #[serde(flatten)]
    labels: HashMap<String, String>,
}

impl Provider {
    pub async fn fetch(
        &self,
        http_client: &reqwest::Client,
    ) -> super::error::Result<Vec<VersionInfo>> {
        let mut url = self.api_url.clone();
        url.path_segments_mut()
            .map_err(|_| url::ParseError::RelativeUrlWithCannotBeABaseBase)?
            .pop_if_empty()
            .extend(["v1", "query"]);
        url.query_pairs_mut().append_pair("query", &self.query);

        let api_response: QueryResponse = http_client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(api_response
            .data
            .result
            .into_iter()
            .map(|mut result| {
                let version = result.metric.labels.remove(&self.label);
                VersionInfo {
                    version,
                    labels: result.metric.labels,
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, env::VarError};

    use reqwest::Url;

    use crate::providers::{
        prometheus::{default_version_label, Provider},
        VersionInfo,
    };

    fn api_url() -> Url {
        static DEFAULT_TEST_API_URL: &str = "http://localhost:8080/prometheus/";
        let url = std::env::var("TEST_PROMETHEUS_API_URL")
            .or_else(|_| {
                Ok(format!(
                    "{}/{}",
                    std::env::var("TEST_API_URL")?,
                    "prometheus"
                ))
            })
            .unwrap_or_else(|_: VarError| DEFAULT_TEST_API_URL.into());
        Url::parse(&url).unwrap()
    }

    #[tokio::test]
    async fn test_fetch_prometheus_versions() {
        let client = reqwest::Client::new();
        let provider = Provider {
            api_url: api_url(),
            query: "dmarc_metrics_exporter_build_info".into(),
            label: default_version_label(),
        };
        let releases = provider.fetch(&client).await.unwrap();

        let mut expected_labels = HashMap::new();
        expected_labels.insert("instance".into(), "localhost:9797".into());
        expected_labels.insert("job".into(), "dmarc-metrics-exporter".into());

        assert_eq!(
            releases,
            vec![VersionInfo {
                version: Some("0.8.0".into()),
                labels: expected_labels
            }]
        );
    }
}
