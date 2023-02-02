use std::collections::HashMap;

use serde::Deserialize;

use crate::baseurl::BaseUrl;

use super::{version_extractor::VersionExtractor, VersionInfo};

fn default_prometheus_url() -> BaseUrl {
    BaseUrl::parse("http://localhost:9090/api/").unwrap()
}

fn default_version_label() -> String {
    "version".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct Provider {
    pub query: String,

    #[serde(default = "default_version_label")]
    pub label: String,

    #[serde(flatten)]
    pub version_extractor: VersionExtractor,

    #[serde(default = "default_prometheus_url")]
    pub api_url: BaseUrl,
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
        url.extend(["v1", "query"]);
        url.query_pairs_mut().append_pair("query", &self.query);

        let api_response: QueryResponse = http_client
            .get(url.into_url())
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
                let version = result
                    .metric
                    .labels
                    .remove(&self.label)
                    .and_then(|v| self.version_extractor.extract(&v));
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
    use std::collections::HashMap;

    use crate::{
        providers::{
            prometheus::{default_version_label, Provider},
            version_extractor::VersionExtractor,
            VersionInfo,
        },
        test_config::prometheus_api_url,
    };

    #[tokio::test]
    async fn test_fetch_prometheus_versions() {
        let client = reqwest::Client::new();
        let provider = Provider {
            query: "dmarc_metrics_exporter_build_info".into(),
            label: default_version_label(),
            version_extractor: VersionExtractor::default(),
            api_url: prometheus_api_url(),
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
