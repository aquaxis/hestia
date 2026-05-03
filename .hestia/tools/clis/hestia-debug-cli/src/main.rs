//! hestia-debug-cli -- Debug conductor CLI client (in-process Handler invocation)

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::config::CommonOpts;
use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_debug_conductor::handler::DebugHandler;

#[derive(Parser)]
#[command(name = "hestia-debug-cli", version, about = "Debug domain CLI (in-process)")]
struct Cli {
    #[command(flatten)]
    common: CommonOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a debug session
    Create,
    /// Connect to a debug target
    Connect,
    /// Disconnect from a debug target
    Disconnect,
    /// Program the target device
    Program,
    /// Capture signal waveforms
    Capture {
        #[command(subcommand)]
        capture_command: CaptureCommands,
    },
    /// List available signals
    Signals,
    /// Set a trigger condition
    Trigger,
    /// Reset the target
    Reset,
    /// UART loopback / send-and-receive test
    UartLoopback {
        /// Serial device path (default: /dev/ttyUSB1)
        #[arg(long, default_value = "/dev/ttyUSB1")]
        device: String,
        /// Baud rate
        #[arg(long, default_value_t = 115200)]
        baud: u32,
        /// Pattern to send (UTF-8 string; use --pattern-hex for binary)
        #[arg(long, default_value = "ABCD")]
        pattern: String,
        /// Read back bytes for true loopback verification
        #[arg(long)]
        read_back: bool,
        /// Per-read timeout in ms
        #[arg(long, default_value_t = 500)]
        read_timeout_ms: u64,
        /// Skip actual execution; emit plan only
        #[arg(long)]
        no_execute: bool,
    },
    /// Show Debug conductor status
    Status,
}

#[derive(Subcommand)]
enum CaptureCommands {
    /// Start signal capture
    Start,
    /// Stop signal capture
    Stop,
}

fn build_request(method: &str, params: serde_json::Value) -> Request {
    Request {
        kind: "prompt".to_string(),
        from: "cli".to_string(),
        method: method.to_string(),
        params,
        id: MessageId::new(),
        trace_id: None,
    }
}

fn emit(common: &CommonOpts, label: &str, value: &serde_json::Value, is_error: bool) -> Result<()> {
    let json = serde_json::to_string(value)?;
    if common.output == "json" {
        if is_error {
            eprintln!("{}", json);
        } else {
            println!("{}", json);
        }
    } else if is_error {
        eprintln!("[{label}] error: {json}");
    } else {
        println!("[{label}] {json}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.common.verbose {
        let _ = tracing_subscriber::fmt::try_init();
    }

    let (method, params) = match &cli.command {
        Commands::Create => ("debug.create", serde_json::json!({})),
        Commands::Connect => ("debug.connect", serde_json::json!({})),
        Commands::Disconnect => ("debug.disconnect", serde_json::json!({})),
        Commands::Program => ("debug.program", serde_json::json!({})),
        Commands::Capture { capture_command } => match capture_command {
            CaptureCommands::Start => ("debug.startCapture", serde_json::json!({})),
            CaptureCommands::Stop => ("debug.stopCapture", serde_json::json!({})),
        },
        Commands::Signals => ("debug.read_signals", serde_json::json!({})),
        Commands::Trigger => ("debug.set_trigger", serde_json::json!({})),
        Commands::Reset => ("debug.reset", serde_json::json!({})),
        Commands::UartLoopback { device, baud, pattern, read_back, read_timeout_ms, no_execute } => (
            "debug.uart_loopback",
            serde_json::json!({
                "device": device,
                "baud": baud,
                "pattern": pattern,
                "read_back": read_back,
                "read_timeout_ms": read_timeout_ms,
                "execute": !no_execute,
            }),
        ),
        Commands::Status => ("debug.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let handler = DebugHandler;
    match handler.handle_request(request).await {
        Response::Success(s) => {
            emit(&cli.common, method, &s.result, false)?;
            Ok(())
        }
        Response::Error(e) => {
            let err_value = serde_json::to_value(&e.error)?;
            emit(&cli.common, method, &err_value, true)?;
            std::process::exit(1);
        }
    }
}
