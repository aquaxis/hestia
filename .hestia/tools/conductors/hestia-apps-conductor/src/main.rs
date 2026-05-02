use conductor_sdk::agent::ConductorId;
use conductor_sdk::server::ConductorServer;
use conductor_sdk::config::CommonOpts;
use clap::Parser;
use hestia_apps_conductor::handler::AppsHandler;

#[derive(Debug, Parser)]
#[command(name = "hestia-apps-conductor", version)]
struct Opts {
    #[command(flatten)]
    common: CommonOpts,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("hestia-apps-conductor starting");
    tracing::info!("conductor_id = apps");

    let handler = AppsHandler;
    let server = ConductorServer::new(ConductorId::Apps, Box::new(handler))?;
    server.run().await?;
    Ok(())
}