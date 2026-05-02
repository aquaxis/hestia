use conductor_sdk::agent::ConductorId;
use conductor_sdk::server::ConductorServer;
use hestia_pcb_conductor::handler::PcbHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("hestia-pcb-conductor starting...");

    let handler = PcbHandler;
    let server = ConductorServer::new(ConductorId::Pcb, Box::new(handler))?;
    server.run().await?;
    Ok(())
}