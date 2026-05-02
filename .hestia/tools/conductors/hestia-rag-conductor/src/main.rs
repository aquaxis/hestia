use conductor_sdk::agent::ConductorId;
use conductor_sdk::server::ConductorServer;
use hestia_rag_conductor::handler::RagHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("hestia-rag-conductor starting...");

    let _conductor = rag_conductor_core::RagConductor::new();

    let handler = RagHandler;
    let server = ConductorServer::new(ConductorId::Rag, Box::new(handler))?;
    server.run().await?;
    Ok(())
}