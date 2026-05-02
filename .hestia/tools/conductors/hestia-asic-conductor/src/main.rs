use conductor_sdk::agent::ConductorId;
use conductor_sdk::server::ConductorServer;
use hestia_asic_conductor::handler::AsicHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("hestia-asic-conductor starting...");

    let handler = AsicHandler;
    let server = ConductorServer::new(ConductorId::Asic, Box::new(handler))?;
    server.run().await?;
    Ok(())
}