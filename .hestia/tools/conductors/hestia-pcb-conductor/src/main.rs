use pcb_conductor_core::daemon::PcbDaemon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("hestia-pcb-conductor starting...");
    let mut daemon = PcbDaemon::new();
    daemon.run().await
}