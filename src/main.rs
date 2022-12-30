use std::{fs::File, time::Duration};

use clap::Parser;
use metrics::Metrics;
use prometheus_client::{encoding::text::encode, registry::Registry};
use release_collection::ReleaseCollection;
use serde::Deserialize;

mod baseurl;
mod checks;
mod metrics;
mod providers;
mod release_collection;

use checks::upgrade_pending::UpgradePendingCheck;
use providers::Provider;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// Configuration file to load
    #[arg(long = "config.file")]
    config: std::path::PathBuf,

    /// Timeout for HTTP requests (seconds)
    #[arg(long = "http.timout", default_value_t = 10)]
    http_timeout_seconds: u64,
}

#[derive(Deserialize)]
struct Config {
    providers: Vec<Provider>,
    upgrade_pending_checks: Vec<UpgradePendingCheck>,
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config_file = File::open(args.config)?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    let http_client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .timeout(Duration::from_secs(args.http_timeout_seconds))
        .build()?;

    let releases = ReleaseCollection::collect_from(&config.providers, &http_client).await;

    let metrics = Metrics::new();
    let mut registry = <Registry>::default();
    metrics.register(&mut registry);

    metrics.update(
        config
            .upgrade_pending_checks
            .iter()
            .map(|c| (c.name.as_str(), c.check(&releases.releases))),
    );

    let mut buffer = String::new();
    encode(&mut buffer, &registry).unwrap();
    println!("{}", buffer);

    Ok(())
}
