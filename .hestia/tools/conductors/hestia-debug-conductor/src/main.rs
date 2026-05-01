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

    info!("hestia-debug-conductor starting...");

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

    info!(
        "registered {} debug adapter(s)",
        registry.list().len()
    );

    // Create the session state machine
    let _state_machine = debug_conductor_core::SessionStateMachine::new();

    info!("hestia-debug-conductor ready");
    Ok(())
}