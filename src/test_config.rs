use std::env::VarError;

use crate::baseurl::BaseUrl;

pub fn github_api_url() -> BaseUrl {
    static DEFAULT_TEST_API_URL: &str = "http://localhost:8080/github";
    BaseUrl::parse(
        std::env::var("TEST_GITHUB_API_URL")
            .or_else(|_| Ok(format!("{}/{}", std::env::var("TEST_API_URL")?, "github")))
            .unwrap_or_else(|_: VarError| DEFAULT_TEST_API_URL.into())
            .as_str(),
    )
    .unwrap()
}

pub fn prometheus_api_url() -> BaseUrl {
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
    BaseUrl::parse(&url).unwrap()
}
