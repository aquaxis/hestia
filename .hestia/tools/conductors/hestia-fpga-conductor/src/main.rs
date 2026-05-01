use fpga_conductor_core::daemon::FpgaDaemon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("hestia-fpga-conductor starting...");
    let mut daemon = FpgaDaemon::new();
    daemon.run().await
}