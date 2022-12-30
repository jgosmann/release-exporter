use std::collections::HashMap;

use regex::Regex;
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer,
};

use crate::baseurl::BaseUrl;

use super::VersionInfo;

fn github_api_url() -> BaseUrl {
    BaseUrl::parse("https://api.github.com").unwrap()
}

fn default_tag_name_regex() -> Regex {
    Regex::new(r"^v?(.*)$").unwrap()
}

fn default_version_fmt() -> String {
    "${1}".into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GithubRelease {
    version: Option<String>,
}

impl Into<VersionInfo> for GithubRelease {
    fn into(self) -> VersionInfo {
        VersionInfo {
            version: self.version,
            labels: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct VersionExtractor {
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
    pub repo: GithubRepo,

    #[serde(flatten)]
    pub version_extractor: VersionExtractor,

    #[serde(default = "github_api_url")]
    pub api_url: BaseUrl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GithubRepo {
    pub user: String,
    pub name: String,
}

impl<'de> Deserialize<'de> for GithubRepo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(GithubRepoVisitor)
    }
}

struct GithubRepoVisitor;

impl<'de> Visitor<'de> for GithubRepoVisitor {
    type Value = GithubRepo;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a Github repository of the format username/repo")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let (user, name) = s
            .split_once('/')
            .ok_or_else(|| serde::de::Error::invalid_value(Unexpected::Str(s), &self))?
            .into();
        Ok(GithubRepo {
            user: user.into(),
            name: name.into(),
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct LatestReleaseResponse {
    tag_name: String,
}

impl LatestReleaseProvider {
    pub async fn fetch(
        &self,
        http_client: &reqwest::Client,
    ) -> super::error::Result<GithubRelease> {
        let mut url = self.api_url.clone();
        url.extend([
            "repos",
            &self.repo.user,
            &self.repo.name,
            "releases",
            "latest",
        ]);

        let api_response = http_client
            .get(url.into_url())
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
    use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};

    use crate::{providers::github::GithubRelease, test_config::github_api_url};

    use super::{GithubRepo, LatestReleaseProvider, LatestReleaseResponse, VersionExtractor};

    #[tokio::test]
    async fn test_fetch_latest_github_release() {
        let client = reqwest::Client::new();
        let provider = LatestReleaseProvider {
            repo: GithubRepo {
                user: "jgosmann".into(),
                name: "dmarc-metrics-exporter".into(),
            },
            api_url: github_api_url(),
            version_extractor: VersionExtractor::default(),
        };
        let release = provider.fetch(&client).await.unwrap();
        assert_eq!(
            release,
            GithubRelease {
                version: Some("0.8.0".into()),
            }
        )
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

    #[test]
    fn test_deserialize_github_repo() {
        let repo = GithubRepo {
            user: "user".into(),
            name: "name".into(),
        };
        assert_de_tokens(&repo, &[Token::Str("user/name")]);
    }

    #[test]
    fn test_deserialize_github_repo_multiple_slashes() {
        let repo = GithubRepo {
            user: "user".into(),
            name: "name/foo".into(),
        };
        assert_de_tokens(&repo, &[Token::Str("user/name/foo")]);
    }

    #[test]
    fn test_deserialize_github_repo_no_slashes() {
        assert_de_tokens_error::<GithubRepo>(&[Token::Str("foo")], "invalid value: string \"foo\", expected a Github repository of the format username/repo");
    }
}
