use anyhow::Result;
use clap::Parser;
use conductor_sdk::config::CommonOpts;

/// Hestia RTL Conductor daemon
#[derive(Debug, Parser)]
#[command(name = "hestia-rtl-conductor", version)]
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

    tracing::info!("hestia-rtl-conductor starting");
    tracing::info!("conductor_id = rtl");

    // TODO: initialize transport, register adapters, serve RPC
    Ok(())
}