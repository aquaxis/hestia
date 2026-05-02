use conductor_sdk::agent::ConductorId;
use conductor_sdk::server::ConductorServer;
use hestia_debug_conductor::handler::DebugHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("hestia-debug-conductor starting...");

    // Build the plugin registry and register built-in adapters
    let mut registry = debug_plugin_registry::PluginRegistry::new();
    let jtag = debug_adapter_jtag::JtagAdapter::new(debug_adapter_jtag::JtagConfig {
        config_file: String::new(),
        ..Default::default()
    });
    registry.register(Box::new(jtag))?;

    let swd = debug_adapter_swd::SwdAdapter::new(debug_adapter_swd::SwdConfig {
        target: String::new(),
        ..Default::default()
    });
    registry.register(Box::new(swd))?;

    let ila = debug_adapter_ila::IlaAdapter::new(debug_adapter_ila::IlaConfig {
        vendor: debug_adapter_ila::IlaVendor::Xilinx,
        device: String::new(),
        ..Default::default()
    });
    registry.register(Box::new(ila))?;

    tracing::info!(
        "registered {} debug adapter(s)",
        registry.list().len()
    );

    let handler = DebugHandler;
    let server = ConductorServer::new(ConductorId::Debug, Box::new(handler))?;
    server.run().await?;
    Ok(())
}