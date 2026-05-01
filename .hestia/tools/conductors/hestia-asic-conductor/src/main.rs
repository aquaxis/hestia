use asic_conductor_core::daemon::AsicDaemon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("hestia-asic-conductor starting...");
    let mut daemon = AsicDaemon::new();
    daemon.run().await
}