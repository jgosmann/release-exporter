use std::{fs::File, time::Duration};

use clap::Parser;
use serde::Deserialize;

mod providers;
mod serde_url;

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

    for provider in config.providers {
        for version in provider.versions(&http_client).await? {
            println!("{:?}", version);
        }
    }
    Ok(())
}
