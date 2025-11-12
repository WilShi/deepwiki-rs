use crate::generator::workflow::launch;
use anyhow::Result;
use clap::Parser;

mod cache;
mod cli;
mod config;
mod generator;
mod i18n;
mod llm;
mod memory;
mod types;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();
    let config = args.into_config();

    launch(&config).await
}
