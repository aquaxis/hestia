use anyhow::{bail, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Hestia -- unified runner for domain conductors and CLIs
#[derive(Parser)]
#[command(name = "hestia", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Initialize .hestia/ directory in the current project
    Init,
    /// Start all or a specific conductor daemon
    Start {
        /// Domain name (ai, rtl, fpga, asic, pcb, hal, apps, debug, rag).
        /// Omit to start all conductors.
        domain: Option<String>,
    },
    /// Stop all or a specific conductor daemon
    Stop {
        /// Domain name. Omit to stop all conductors.
        domain: Option<String>,
    },
    /// Show status of all conductor daemons
    Status,
    /// Dispatch to hestia-ai-cli
    Ai {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Dispatch to hestia-rtl-cli
    Rtl {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Dispatch to hestia-fpga-cli
    Fpga {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Dispatch to hestia-asic-cli
    Asic {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Dispatch to hestia-pcb-cli
    Pcb {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Dispatch to hestia-hal-cli
    Hal {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Dispatch to hestia-apps-cli
    Apps {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Dispatch to hestia-debug-cli
    Debug {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Dispatch to hestia-rag-cli
    Rag {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

/// Domain names that have a corresponding conductor.
const DOMAINS: &[&str] = &[
    "ai", "rtl", "fpga", "asic", "pcb", "hal", "apps", "debug", "rag",
];

/// Group 1 domain names (all except ai).
const GROUP1_DOMAINS: &[&str] = &[
    "rtl", "fpga", "asic", "pcb", "hal", "apps", "debug", "rag",
];

/// Maximum time to wait for ai-conductor readiness (seconds).
const AI_READINESS_TIMEOUT_SECS: u64 = 30;

/// Default content written to .hestia/config.toml on init.
const DEFAULT_CONFIG: &str = r#"[hestia]
version = "0.1.0"

[conductor]
# Start-up delay between conductors (ms)
stagger_ms = 500

[agent_cli]
# LLM バックエンド: claude / codex / ollama / llama_cpp
backend = "ollama"
model = "glm-5.1:cloud"
"#;

fn dispatch_cli(domain: &str, args: &[String]) -> Result<()> {
    let bin = format!("hestia-{domain}-cli");
    let status = std::process::Command::new(&bin)
        .args(args)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to execute {bin}: {e}"))?;
    if !status.success() {
        bail!("{bin} exited with {}", status);
    }
    Ok(())
}

fn persona_path(domain: &str) -> PathBuf {
    Path::new(".hestia/personas").join(format!("{domain}.md"))
}

fn workspace_path(domain: &str) -> PathBuf {
    Path::new(".hestia/workspaces").join(domain)
}

async fn start_conductor(domain: &str) -> Result<()> {
    let persona = persona_path(domain);
    if !persona.exists() {
        bail!("persona file not found: {}", persona.display());
    }

    let workdir = workspace_path(domain);
    if !workdir.exists() {
        std::fs::create_dir_all(&workdir)?;
    }

    println!("Starting agent-cli --name {} --persona {} ...", domain, persona.display());
    let _child = Command::new("agent-cli")
        .arg("run")
        .arg("--persona")
        .arg(&persona)
        .arg("--name")
        .arg(domain)
        .arg("--auto-approve-tools")
        .current_dir(&workdir)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn agent-cli for {domain}: {e}"))?;

    Ok(())
}

async fn wait_for_ai_readiness() -> Result<()> {
    println!("Waiting for ai-conductor readiness ...");
    let timeout = std::time::Duration::from_secs(AI_READINESS_TIMEOUT_SECS);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        let output = Command::new("agent-cli")
            .arg("send")
            .arg("ai")
            .arg("{\"method\":\"system.health.v1\",\"id\":\"readiness-check\"}")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await;

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("\"online\"") || stdout.contains("\"status\":\"online\"") {
                println!("ai-conductor is online");
                return Ok(());
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    bail!("ai-conductor did not become online within {AI_READINESS_TIMEOUT_SECS}s");
}

async fn start_all_conductors() -> Result<()> {
    // Group 0: ai-conductor を最優先で起動し、readiness を待機
    start_conductor("ai").await?;
    wait_for_ai_readiness().await?;

    // Group 1: 残り 8 conductor を並列起動
    let mut handles = Vec::new();
    for domain in GROUP1_DOMAINS {
        let h = tokio::spawn(async move { start_conductor(domain).await });
        handles.push(h);
    }

    for h in handles {
        h.await.map_err(|e| anyhow::anyhow!("task join error: {e}"))??;
    }

    println!("All conductors started (running in background via agent-cli)");
    Ok(())
}

async fn stop_conductor(domain: &str) -> Result<()> {
    println!("Stopping {} conductor ...", domain);
    let output = Command::new("agent-cli")
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to run agent-cli list: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with(domain) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let pid_str = parts[1].trim_end_matches(',');
                if let Ok(pid) = pid_str.parse::<u32>() {
                    let _ = Command::new("kill")
                        .arg(pid.to_string())
                        .output()
                        .await;
                    println!("Stopped {domain} (pid {pid})");
                    return Ok(());
                }
            }
        }
    }

    println!("{domain} conductor not found in running peers");
    Ok(())
}

async fn stop_all_conductors() -> Result<()> {
    for domain in DOMAINS {
        stop_conductor(domain).await?;
    }
    Ok(())
}

fn init_hestia_dir() -> Result<()> {
    let base = Path::new(".hestia");
    let dirs = [
        base.join("spec"),
        base.join("log"),
        base.join("common/rules"),
        base.join("personas"),
        base.join("workspaces"),
    ];

    if base.exists() {
        bail!(".hestia/ already exists in the current directory");
    }

    std::fs::create_dir_all(base)?;
    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    let config_path = base.join("config.toml");
    std::fs::write(&config_path, DEFAULT_CONFIG)?;

    println!("Initialized .hestia/ directory");
    Ok(())
}

async fn show_status() -> Result<()> {
    let output = Command::new("agent-cli")
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to run agent-cli list: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{stdout}");

    if !output.status.success() {
        bail!("agent-cli list exited with {}", output.status);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init_hestia_dir()?,
        Commands::Start { domain } => match domain {
            Some(d) => start_conductor(&d).await?,
            None => start_all_conductors().await?,
        },
        Commands::Stop { domain } => match domain {
            Some(d) => stop_conductor(&d).await?,
            None => stop_all_conductors().await?,
        },
        Commands::Status => show_status().await?,
        Commands::Ai { args } => dispatch_cli("ai", &args)?,
        Commands::Rtl { args } => dispatch_cli("rtl", &args)?,
        Commands::Fpga { args } => dispatch_cli("fpga", &args)?,
        Commands::Asic { args } => dispatch_cli("asic", &args)?,
        Commands::Pcb { args } => dispatch_cli("pcb", &args)?,
        Commands::Hal { args } => dispatch_cli("hal", &args)?,
        Commands::Apps { args } => dispatch_cli("apps", &args)?,
        Commands::Debug { args } => dispatch_cli("debug", &args)?,
        Commands::Rag { args } => dispatch_cli("rag", &args)?,
    }

    Ok(())
}