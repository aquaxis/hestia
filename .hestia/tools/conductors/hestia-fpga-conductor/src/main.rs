use conductor_sdk::agent::ConductorId;
use conductor_sdk::server::ConductorServer;
use hestia_fpga_conductor::handler::FpgaHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("hestia-fpga-conductor starting...");

    let handler = FpgaHandler;
    let server = ConductorServer::new(ConductorId::Fpga, Box::new(handler))?;
    server.run().await?;
    Ok(())
}