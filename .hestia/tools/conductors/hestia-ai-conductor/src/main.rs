use conductor_sdk::agent::ConductorId;
use conductor_sdk::config::{CommonOpts, HestiaClientConfig};
use conductor_sdk::server::ConductorServer;
use clap::Parser;
use hestia_ai_conductor::handler::AiHandler;

#[derive(Debug, Parser)]
#[command(name = "hestia-ai-conductor", version)]
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

    tracing::info!("hestia-ai-conductor starting...");

    let config = HestiaClientConfig::default();
    let handler = AiHandler::new(config);
    let server = ConductorServer::new(ConductorId::Ai, Box::new(handler))?;
    server.run().await?;
    Ok(())
}