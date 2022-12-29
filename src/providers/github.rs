use regex::Regex;
use serde::Deserialize;

use super::Release;

fn github_api_url() -> String {
    "https://api.github.com".into()
}

fn default_tag_name_regex() -> Regex {
    Regex::new(r"^v?(.*)$").unwrap()
}

fn default_version_fmt() -> String {
    "${1}".into()
}

#[derive(Clone, Debug)]
pub struct GithubRelease {
    version: Option<String>,
}

impl Release for GithubRelease {
    fn version(&self) -> Option<&str> {
        self.version.as_ref().map(String::as_str)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct VersionExtractor {
    #[serde(default = "default_tag_name_regex", with = "serde_regex")]
    tag_name_regex: Regex,

    #[serde(default = "default_version_fmt")]
    version_fmt: String,
}

impl Default for VersionExtractor {
    fn default() -> Self {
        Self {
            tag_name_regex: default_tag_name_regex(),
            version_fmt: default_version_fmt(),
        }
    }
}

impl VersionExtractor {
    pub fn extract(&self, response: &LatestReleaseResponse) -> Option<String> {
        self.tag_name_regex
            .find(&response.tag_name)
            .map(|version_match| {
                self.tag_name_regex
                    .replace(
                        &response.tag_name[version_match.start()..version_match.end()],
                        &self.version_fmt,
                    )
                    .into()
            })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct LatestReleaseProvider {
    repo: String,

    #[serde(flatten)]
    version_extractor: VersionExtractor,

    #[serde(default = "github_api_url")]
    api_url: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct LatestReleaseResponse {
    tag_name: String,
}

impl LatestReleaseProvider {
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
    ) -> super::error::Result<GithubRelease> {
        let url = format!(
            "{}/repos/{}/releases/latest",
            self.normalized_api_url(),
            self.repo
        );
        let api_response = http_client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let version = self.version_extractor.extract(&api_response);
        Ok(GithubRelease { version })
    }
}

#[cfg(test)]
mod tests {
    use crate::providers::Release;

    use super::{LatestReleaseProvider, LatestReleaseResponse, VersionExtractor};

    fn api_url() -> String {
        static DEFAULT_TEST_API_URL: &str = "http://localhost:8080";
        std::env::var("TEST_GITHUB_API_URL")
            .or_else(|_| std::env::var("TEST_API_URL"))
            .unwrap_or_else(|_| DEFAULT_TEST_API_URL.into())
    }

    #[tokio::test]
    async fn test_fetch_latest_github_release() {
        let client = reqwest::Client::new();
        let provider = LatestReleaseProvider {
            repo: "jgosmann/dmarc-metrics-exporter".into(),
            api_url: api_url(),
            version_extractor: VersionExtractor::default(),
        };
        let release = provider.fetch(&client).await.unwrap();
        assert_eq!(release.version(), Some("0.8.0"))
    }

    #[test]
    fn test_extract_version() {
        let response = LatestReleaseResponse {
            tag_name: "1.2.3".into(),
        };
        assert_eq!(
            VersionExtractor::default().extract(&response).unwrap(),
            String::from("1.2.3")
        );

        let response = LatestReleaseResponse {
            tag_name: "v1.2.3".into(),
        };
        assert_eq!(
            VersionExtractor::default().extract(&response).unwrap(),
            String::from("1.2.3")
        );
    }
}
