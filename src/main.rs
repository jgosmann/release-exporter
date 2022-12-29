use std::fs::File;

use clap::Parser;
use serde::Deserialize;

mod providers;

use providers::Provider;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(long = "config.file")]
    config: std::path::PathBuf,
}

#[derive(Deserialize)]
struct Config {
    providers: Vec<Provider>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config_file = File::open(args.config)?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    let http_client = reqwest::Client::new();

    for provider in config.providers {
        println!("{:?}", provider.release(&http_client).await?.version());
    }
    Ok(())
}
