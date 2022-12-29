use std::collections::HashMap;

use serde::Deserialize;

use super::VersionInfo;

fn default_prometheus_url() -> String {
    "http://localhost:9090/api".into()
}

fn default_version_label() -> String {
    "version".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct Provider {
    query: String,

    #[serde(default = "default_version_label")]
    label: String,

    #[serde(default = "default_prometheus_url")]
    api_url: String,
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
    fn normalized_api_url(&self) -> &str {
        if self.api_url.ends_with('/') {
            &self.api_url[..self.api_url.len() - 1]
        } else {
            self.api_url.as_str()
        }
    }

    pub async fn fetch(
        &self,
        http_client: &reqwest::Client,
    ) -> super::error::Result<Vec<VersionInfo>> {
        let url = format!(
            "{}/v1/query?query={}",
            self.normalized_api_url(),
            self.query // FIXME url encode
        );
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

    use crate::providers::{
        prometheus::{default_version_label, Provider},
        VersionInfo,
    };

    fn api_url() -> String {
        static DEFAULT_TEST_API_URL: &str = "http://localhost:8080/prometheus";
        std::env::var("TEST_PROMETHEUS_API_URL")
            .or_else(|_| {
                Ok(format!(
                    "{}/{}",
                    std::env::var("TEST_API_URL")?,
                    "prometheus"
                ))
            })
            .unwrap_or_else(|_: VarError| DEFAULT_TEST_API_URL.into())
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
