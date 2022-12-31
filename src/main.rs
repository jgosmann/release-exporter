use std::{fs::File, sync::Arc, time::Duration};

use clap::Parser;
use metrics::Metrics;
use prometheus_client::{encoding::text::encode, metrics::info::Info, registry::Registry};
use release_collection::ReleaseCollection;
use reqwest::Client;
use serde::Deserialize;

mod baseurl;
mod checks;
mod metrics;
mod providers;
mod release_collection;
#[cfg(test)]
mod test_config;

use checks::upgrade_pending::UpgradePendingCheck;
use providers::Provider;
use tide::Server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// Configuration file to load
    #[arg(long = "config.file")]
    config: std::path::PathBuf,

    /// Timeout for HTTP requests (seconds)
    #[arg(long = "http.timout", default_value_t = 10)]
    http_timeout_seconds: u64,

    /// Address on which to expose metrics
    #[arg(long = "web.listen-address", default_value_t = String::from("localhost:31343"))]
    listen_address: String,
}

#[derive(Clone, Deserialize)]
struct Config {
    providers: Vec<Provider>,
    upgrade_pending_checks: Vec<UpgradePendingCheck>,
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Clone)]
struct State {
    config: Config,
    http_client: reqwest::Client,
    metrics: Metrics,
    registry: Arc<Registry>,
}

fn create_app(config: Config, http_client: Client) -> Server<State> {
    let mut registry = <Registry>::default();
    registry.register(
        "release_exporter_build",
        "A metric with a constant '1' value labeled by version of the release-exporter",
        Info::new(vec![("version", env!("CARGO_PKG_VERSION"))]),
    );

    let metrics = Metrics::new();
    metrics.register(&mut registry);

    let state = State {
        config,
        http_client,
        metrics,
        registry: Arc::new(registry),
    };

    let mut app = tide::with_state(state);
    app.at("/metrics")
        .get(|req: tide::Request<State>| async move {
            let State {
                config,
                http_client,
                metrics,
                registry,
            } = req.state();

            let releases =
                ReleaseCollection::collect_from(config.providers.clone(), http_client).await;

            for (provider, error) in releases.errors {
                tide::log::error!("Provider {} reported error: {}", provider, error);
            }

            metrics.update(
                config
                    .upgrade_pending_checks
                    .iter()
                    .map(|c| (c.name.as_str(), c.check(&releases.releases))),
            );
            let mut buffer = String::new();
            encode(&mut buffer, registry).unwrap();

            Ok(tide::Response::builder(200)
                .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
                .body(buffer)
                .build())
        });
    app
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config_file = File::open(args.config)?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    let http_client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .timeout(Duration::from_secs(args.http_timeout_seconds))
        .build()?;

    tide::log::start();
    let app = create_app(config, http_client);
    Ok(app.listen(args.listen_address).await?)
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use reqwest::{Client, Url};
    use tide::{
        http::Method,
        http::{Request, Response},
    };

    use crate::{
        checks::upgrade_pending::UpgradePendingCheck,
        create_app,
        providers::{
            github::{self, VersionExtractor},
            prometheus, Provider,
        },
        test_config::{github_api_url, prometheus_api_url},
        Config,
    };

    #[tokio::test]
    async fn test_app_metrics_endpoint() {
        let http_client = Client::new();
        let config = Config {
            providers: vec![
                Provider::LatestGithubRelease {
                    config: github::LatestReleaseProvider {
                        repo: github::GithubRepo {
                            user: "jgosmann".into(),
                            name: "dmarc-metrics-exporter".into(),
                        },
                        version_extractor: VersionExtractor::default(),
                        api_url: github_api_url(),
                    },
                    name: "latest_release".into(),
                },
                Provider::Prometheus {
                    config: prometheus::Provider {
                        query: "dmarc_metrics_exporter_build_info".into(),
                        label: "version".into(),
                        api_url: prometheus_api_url(),
                    },
                    name: "current_release".into(),
                },
            ],
            upgrade_pending_checks: vec![UpgradePendingCheck {
                name: "check_name".into(),
                current: "current_release".into(),
                latest: "latest_release".into(),
            }],
        };

        let app = create_app(config, http_client);
        let mut response: Response = app
            .respond(Request::new(
                Method::Get,
                Url::parse("http://localhost/metrics").unwrap(),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body = response.body_string().await.unwrap();
        println!("{}", body);
        let metric_lines: Vec<&str> = body
            .split('\n')
            .filter(|line| !line.starts_with('#'))
            .collect();
        let expected = Regex::new("^upgrades\\{status=\"up-to-date\",name=\"check_name\",latest_version=\"0\\.8\\.0\",.*\\} 1$").unwrap();
        assert!(metric_lines.iter().any(|line| expected.is_match(line)));
        let expected = Regex::new("^release_exporter_build_info\\{version=\".+\"\\} 1$").unwrap();
        assert!(metric_lines.iter().any(|line| expected.is_match(line)));
    }
}
