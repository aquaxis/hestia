use anyhow::Result;
use tracing::info;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("hestia-rag-conductor starting...");

    // Create the RAG conductor
    let _conductor = rag_conductor_core::RagConductor::new();

    info!("hestia-rag-conductor ready");
    Ok(())
}