use anyhow::Result;
use clap::Parser;
use conductor_sdk::config::CommonOpts;

/// Hestia Apps Conductor daemon
#[derive(Debug, Parser)]
#[command(name = "hestia-apps-conductor", version)]
struct Opts {
    #[command(flatten)]
    common: CommonOpts,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("hestia-apps-conductor starting");
    tracing::info!("conductor_id = apps");

    // TODO: initialize transport, register adapters, serve RPC
    Ok(())
}