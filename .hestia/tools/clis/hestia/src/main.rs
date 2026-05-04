use anyhow::{bail, Result};
use clap::Parser;
use serde::Deserialize;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::FromRawFd;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

/// Subset of `.hestia/config.toml` consumed by `hestia start`.
///
/// Only the `[agent_cli]` section is read here so we can pass `--provider` /
/// `--model` to `agent-cli run` and prevent the user's global agent-cli
/// config (`~/.config/agent-cli/config.toml`) from silently overriding the
/// project's choice (Phase 24).
#[derive(Debug, Default, Deserialize)]
struct HestiaConfig {
    #[serde(default)]
    agent_cli: AgentCliConfig,
}

#[derive(Debug, Default, Deserialize)]
struct AgentCliConfig {
    /// `claude` / `codex` / `ollama` / `llama_cpp`
    backend: Option<String>,
    model: Option<String>,
}

impl AgentCliConfig {
    /// Map our config key (`backend`) to agent-cli's `--provider` value.
    /// agent-cli expects `llama.cpp` (with dot) but TOML keys can't have dots,
    /// so we accept `llama_cpp` in config and normalize here.
    fn provider_arg(&self) -> Option<String> {
        self.backend.as_deref().map(|b| match b {
            "llama_cpp" => "llama.cpp".to_string(),
            other => other.to_string(),
        })
    }
}

/// Read `.hestia/config.toml` from the current working directory.
/// Returns the parsed config, or an empty default if the file is absent or
/// the section is missing — this preserves backwards compatibility with
/// installations created before Phase 24.
fn load_hestia_config() -> HestiaConfig {
    let path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".hestia/config.toml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return HestiaConfig::default();
    };
    toml::from_str::<HestiaConfig>(&text).unwrap_or_default()
}

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
    /// Tail an agent's structured activity log (Phase 48).
    ///
    /// The workspace `agent.log` only captures agent-cli's banner and
    /// occasional notices because thinking / tool_use events are written
    /// to a separate JSONL log under
    /// `~/.local/share/agent-cli/logs/<agent-id>/`. This subcommand
    /// resolves the latest log for the given domain and tails it so the
    /// user can see real-time orchestrator activity.
    Tail {
        /// Domain name (ai, rtl, fpga, asic, pcb, hal, apps, debug, rag).
        domain: String,
        /// Print the resolved log path instead of streaming.
        #[arg(long)]
        path_only: bool,
    },
    /// (Internal, Phase 49) Mirror an agent-cli structured log into the
    /// workspace `agent.log` so users can `cat .hestia/workspaces/<domain>/agent.log`
    /// and see live orchestrator activity. Spawned automatically by `hestia start`.
    #[command(hide = true)]
    Mirror {
        /// Domain name to mirror.
        domain: String,
    },
    /// (Internal, Phase 55) Spawn a sub-agent agent-cli process with the given
    /// persona file and peer name. Used by `hestia start` to launch planner /
    /// designer sub-agents and by conductor handlers to spawn dynamic coder /
    /// ingest workers (rtl-coder-{module}, hal-coder-{lang}, etc.).
    #[command(hide = true)]
    SpawnSubagent {
        /// Persona filename under `.hestia/personas/` (e.g. `rtl-designer.md`).
        #[arg(long)]
        persona: String,
        /// Peer name for agent-cli `--name` (e.g. `rtl-designer` or `rtl-coder-uart`).
        #[arg(long)]
        name: String,
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

/// Phase 55 — Resident sub-agents launched alongside each conductor.
/// Each entry is `(persona_filename_root, peer_name)`. The persona is loaded
/// from `.hestia/personas/<persona_filename_root>.md`; the peer name is the
/// agent-cli `--name`. Differing entries (e.g. asic-signoff has persona file
/// `asic-signoff-checker.md` but peer name `asic-signoff`, per design HD-033)
/// are encoded explicitly here.
///
/// For Phase 55 we keep this set minimal to the canonical `planner` /
/// `designer` pair across all 9 conductors — these are the agents whose
/// presence directly enables the Phase 53/54 design.v1 delegation path.
/// Specialized sub-agents (synthesizer, implementer, signoff, tester,
/// programmer, schematic, layout, validator, builder, session, analyzer,
/// quality, archivist, search) are reachable via `spawn-subagent` on demand.
const RESIDENT_SUB_AGENTS: &[(&str, &[(&str, &str)])] = &[
    ("ai",    &[("ai-planner", "ai-planner"),    ("ai-designer", "ai-designer")]),
    ("rtl",   &[("rtl-planner", "rtl-planner"),  ("rtl-designer", "rtl-designer")]),
    ("fpga",  &[("fpga-planner", "fpga-planner"),("fpga-designer", "fpga-designer")]),
    ("asic",  &[("asic-planner", "asic-planner"),("asic-designer", "asic-designer")]),
    ("pcb",   &[("pcb-planner", "pcb-planner"),  ("pcb-designer", "pcb-designer")]),
    ("hal",   &[("hal-planner", "hal-planner"),  ("hal-designer", "hal-designer")]),
    ("apps",  &[("apps-planner", "apps-planner"),("apps-designer", "apps-designer")]),
    ("debug", &[("debug-planner", "debug-planner"),("debug-designer", "debug-designer")]),
    ("rag",   &[("rag-planner", "rag-planner"),  ("rag-designer", "rag-designer")]),
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
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".hestia/personas")
        .join(format!("{domain}.md"))
}

fn workspace_path(domain: &str) -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".hestia/workspaces")
        .join(domain)
}

/// Phase 81 — Initialize per-peer hestia workspace (replaces Phase 57's
/// `init_aiprj_workspace`).
///
/// Phase 22 P-1 (rules 隔離) → Phase 57 P-2 (`.aiprj/rules` symlink 共有)
/// → Phase 81 **P-3** (`.hestia/rules/` への hestia agent 向け解釈変更版を配置、
/// `.aiprj/` 直接参照を排除) の進化に伴う改名 + ロジック変更。
///
/// 各 peer (conductor or sub-agent) に `<workspace>/instruction.md` placeholder
/// のみ生成。`.aiprj/rules` への symlink は生成しない — agent は project-root
/// の `<root>/.hestia/rules/` を共有参照する。これにより hestia ランタイムは
/// `.aiprj/` 不在環境（end user 配布版、CI 環境等）でも完全動作可能となる。
///
/// Failures are non-fatal — they're logged but don't block conductor startup.
fn init_hestia_workspace(peer_name: &str) {
    let workspace = workspace_path(peer_name);
    if let Err(e) = std::fs::create_dir_all(&workspace) {
        eprintln!("[warn] failed to create {}: {e}", workspace.display());
        return;
    }

    let instruction = workspace.join("instruction.md");
    if !instruction.exists() {
        let placeholder = format!(
            "# Instruction for peer `{peer_name}`\n\n\
             Phase 81 placeholder — populated by ai-conductor or upstream peers \
             when delegation occurs. The agent self-executes setup_ai / \
             update_ai / exec_job / close_ai cycles by referencing \
             `<root>/.hestia/rules/` (Phase 81 P-3).\n"
        );
        if let Err(e) = std::fs::write(&instruction, placeholder) {
            eprintln!("[warn] failed to write {}: {e}", instruction.display());
        }
    }
}

/// Phase 55 — Spawn an agent-cli process for a given persona file + peer name.
/// Used by `start_conductor` (for the main conductor and resident sub-agents)
/// and by the hidden `spawn-subagent` subcommand (for handler-driven dynamic
/// sub-agents like `rtl-coder-{module}`). Creates a per-peer workspace under
/// `.hestia/workspaces/<peer>/`, sets up FIFO stdin so the agent doesn't EOF,
/// redirects stdout/stderr to `agent.log`, and (best-effort) spawns a mirror
/// helper for log visibility.
async fn spawn_agent_cli(persona_filename_root: &str, peer_name: &str) -> Result<()> {
    let persona = persona_path(persona_filename_root);
    if !persona.exists() {
        bail!("persona file not found: {}", persona.display());
    }

    let workdir = workspace_path(peer_name);
    if !workdir.exists() {
        std::fs::create_dir_all(&workdir)?;
    }

    // Phase 81 — set up per-peer hestia workspace before agent-cli starts.
    init_hestia_workspace(peer_name);

    let fifo_path = workdir.join("stdin.pipe");
    let _ = std::fs::remove_file(&fifo_path);
    let c_path = std::ffi::CString::new(fifo_path.as_os_str().as_bytes())
        .map_err(|e| anyhow::anyhow!("invalid fifo path: {e}"))?;
    let mkfifo_result = unsafe { libc::mkfifo(c_path.as_ptr(), 0o600) };
    if mkfifo_result != 0 {
        bail!("failed to create FIFO {}: {}", fifo_path.display(), std::io::Error::last_os_error());
    }
    let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR) };
    if fd < 0 {
        bail!("failed to open FIFO {}: {}", fifo_path.display(), std::io::Error::last_os_error());
    }
    let fifo_stdin = unsafe { std::fs::File::from_raw_fd(fd) };

    let log_path = workdir.join("agent.log");
    let log_file = std::fs::File::create(&log_path)
        .map_err(|e| anyhow::anyhow!("failed to create log file {}: {e}", log_path.display()))?;
    let log_file_stderr = log_file.try_clone()
        .map_err(|e| anyhow::anyhow!("failed to dup log file: {e}"))?;

    let config = load_hestia_config();
    let provider = config.agent_cli.provider_arg();
    let model = config.agent_cli.model.as_deref();

    println!(
        "Starting agent-cli --name {} --persona {} ...",
        peer_name, persona.display()
    );

    let mut cmd = Command::new("agent-cli");
    cmd.arg("run")
        .arg("--persona")
        .arg(&persona)
        .arg("--name")
        .arg(peer_name)
        .arg("--auto-approve-tools");
    if let Some(p) = provider.as_deref() {
        cmd.arg("--provider").arg(p);
    }
    if let Some(m) = model {
        cmd.arg("--model").arg(m);
    }
    let _child = cmd
        .current_dir(&workdir)
        .stdin(Stdio::from(fifo_stdin))
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_file_stderr))
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn agent-cli for {peer_name}: {e}"))?;

    // Best-effort mirror helper (Phase 49). Mirror walks workspace agent-cli
    // logs by domain name; for sub-agents the mirror still runs against the
    // peer name, which has a matching workspace dir.
    let hestia_self = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("hestia"));
    let _mirror = Command::new(&hestia_self)
        .arg("mirror")
        .arg(peer_name)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    Ok(())
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

    // Phase 81 — set up per-peer hestia workspace for the main conductor.
    init_hestia_workspace(domain);

    // Create a FIFO for stdin so agent-cli doesn't exit on EOF.
    // Opening with O_RDWR means the child is both reader and writer,
    // so stdin never gets EOF and the process stays alive.
    let fifo_path = workdir.join("stdin.pipe");
    let _ = std::fs::remove_file(&fifo_path);
    let c_path = std::ffi::CString::new(fifo_path.as_os_str().as_bytes())
        .map_err(|e| anyhow::anyhow!("invalid fifo path: {e}"))?;
    let mkfifo_result = unsafe { libc::mkfifo(c_path.as_ptr(), 0o600) };
    if mkfifo_result != 0 {
        bail!("failed to create FIFO {}: {}", fifo_path.display(), std::io::Error::last_os_error());
    }

    // Open FIFO with O_RDWR — this does NOT block on FIFOs and ensures
    // the read end never gets EOF (the child itself is the writer).
    let fd = unsafe {
        libc::open(c_path.as_ptr(), libc::O_RDWR)
    };
    if fd < 0 {
        bail!("failed to open FIFO {}: {}", fifo_path.display(), std::io::Error::last_os_error());
    }
    let fifo_stdin = unsafe { std::fs::File::from_raw_fd(fd) };

    // Redirect stdout/stderr to log file
    let log_path = workdir.join("agent.log");
    let log_file = std::fs::File::create(&log_path)
        .map_err(|e| anyhow::anyhow!("failed to create log file {}: {e}", log_path.display()))?;
    let log_file_stderr = log_file.try_clone()
        .map_err(|e| anyhow::anyhow!("failed to dup log file: {e}"))?;

    let config = load_hestia_config();
    let provider = config.agent_cli.provider_arg();
    let model = config.agent_cli.model.as_deref();

    let provider_log = provider.as_deref().unwrap_or("(global default)");
    let model_log = model.unwrap_or("(global default)");
    println!(
        "Starting agent-cli --name {} --persona {} --provider {} --model {} ...",
        domain, persona.display(), provider_log, model_log
    );

    let mut cmd = Command::new("agent-cli");
    cmd.arg("run")
        .arg("--persona")
        .arg(&persona)
        .arg("--name")
        .arg(domain)
        .arg("--auto-approve-tools");
    if let Some(p) = provider.as_deref() {
        cmd.arg("--provider").arg(p);
    }
    if let Some(m) = model {
        cmd.arg("--model").arg(m);
    }
    let _child = cmd
        .current_dir(&workdir)
        .stdin(Stdio::from(fifo_stdin))
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_file_stderr))
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn agent-cli for {domain}: {e}"))?;

    // Phase 49: spawn the structured-log mirror as a detached background helper.
    // Resolve hestia's own path (argv[0]) so we always invoke the same binary
    // that started us, regardless of $PATH ordering. Inherit our cwd (the
    // project root) — `mirror_agent_log` calls `workspace_path` which derives
    // the workspace from cwd, so we must NOT change directory here.
    let hestia_self = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("hestia"));
    let _mirror = Command::new(&hestia_self)
        .arg("mirror")
        .arg(domain)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn mirror helper for {domain}: {e}"))?;

    // Phase 55 — launch resident sub-agents (planner / designer) for this
    // conductor. Failures are logged but not fatal: the conductor itself is
    // already up and Phase 54 design.v1 stubs gracefully fall back when a
    // sub-agent isn't reachable.
    if let Some((_, agents)) = RESIDENT_SUB_AGENTS.iter().find(|(d, _)| *d == domain) {
        for (persona_root, peer_name) in *agents {
            if let Err(e) = spawn_agent_cli(persona_root, peer_name).await {
                eprintln!("[warn] failed to start sub-agent {peer_name} for {domain}: {e}");
            }
        }
    }

    Ok(())
}

async fn wait_for_ai_readiness() -> Result<()> {
    println!("Waiting for ai-conductor readiness ...");
    let timeout = std::time::Duration::from_secs(AI_READINESS_TIMEOUT_SECS);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        let output = Command::new("agent-cli")
            .arg("list")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await;

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // agent-cli list output format: ID  NAME  PROVIDER  MODEL  ...
            // Check if any data row contains "ai" as a NAME column value
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("ID") || trimmed.starts_with('-') {
                    continue; // skip header and separator lines
                }
                // Split by whitespace and check if NAME column is "ai"
                let fields: Vec<&str> = trimmed.split_whitespace().collect();
                if fields.len() >= 2 && fields[1] == "ai" {
                    println!("ai-conductor is online");
                    return Ok(());
                }
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
    println!(
        "[Phase 48] Activity logs: workspace agent.log captures only the agent-cli banner."
    );
    println!(
        "[Phase 48] Use `hestia tail <domain>` to stream the LLM's thinking/tool_use events"
    );
    println!(
        "[Phase 48] (or `hestia tail ai --path-only` to discover the underlying JSONL path)."
    );
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
        base.join("rules"),
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

    // Copy persona files from share directory or repo directory
    let personas_dir = base.join("personas");
    let share_dir = home_share_dir();
    let src_dirs = [
        share_dir.join("personas"),
        dirs::home_dir()
            .map(|h| PathBuf::from(h).join(".hestia/src/hestia/.hestia/personas"))
            .unwrap_or_default(),
    ];

    let mut copied = 0u32;
    for src in &src_dirs {
        if src.is_dir() {
            if let Ok(entries) = std::fs::read_dir(src) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "md") {
                        let name = path.file_name().unwrap();
                        let dest = personas_dir.join(name);
                        if !dest.exists() {
                            std::fs::copy(&path, &dest)?;
                            copied += 1;
                        }
                    }
                }
            }
            break; // Use first found source directory
        }
    }

    println!("Initialized .hestia/ directory");
    if copied > 0 {
        println!("Copied {copied} persona files from share directory");
    } else {
        eprintln!(
            "Warning: No persona files found. Run install.sh first or set HESTIA_SHARE_DIR."
        );
    }

    // Phase 81 — copy hestia agent rules from share to project .hestia/rules/.
    // Without these, the agent self-execution loop (Article 4 of setup_project.md)
    // can't find its rule files and degrades to idle.
    let rules_dir = base.join("rules");
    let rules_src_dirs = [
        share_dir.join("rules"),
        dirs::home_dir()
            .map(|h| PathBuf::from(h).join(".hestia/src/hestia/.hestia/rules"))
            .unwrap_or_default(),
    ];
    let mut rules_copied = 0u32;
    for src in &rules_src_dirs {
        if src.is_dir() {
            if let Ok(entries) = std::fs::read_dir(src) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "md") {
                        let name = path.file_name().unwrap();
                        let dest = rules_dir.join(name);
                        if !dest.exists() {
                            std::fs::copy(&path, &dest)?;
                            rules_copied += 1;
                        }
                    }
                }
            }
            break;
        }
    }
    if rules_copied > 0 {
        println!("Copied {rules_copied} hestia rule files from share directory");
    }
    Ok(())
}

fn home_share_dir() -> PathBuf {
    std::env::var("HESTIA_SHARE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(".hestia/share"))
                .unwrap_or_else(|| PathBuf::from(".hestia/share"))
        })
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

/// Resolve the latest agent-cli structured-log path for `domain` (Phase 48).
///
/// Looks up the agent-id of the running agent whose `name` column matches
/// `domain` via `agent-cli list`, then locates the most recently modified
/// `*.jsonl` under `~/.local/share/agent-cli/logs/<agent-id>/`.
async fn resolve_agent_log_path(domain: &str) -> Result<PathBuf> {
    let output = Command::new("agent-cli")
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to run agent-cli list: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut agent_id: Option<String> = None;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("ID") || trimmed.starts_with('-') {
            continue;
        }
        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        if fields.len() >= 2 && fields[1] == domain {
            agent_id = Some(fields[0].to_string());
            break;
        }
    }
    let agent_id = agent_id.ok_or_else(|| {
        anyhow::anyhow!(
            "no running agent named '{domain}' found via 'agent-cli list'. Did you run 'hestia start'?"
        )
    })?;

    let log_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not resolve $HOME"))?
        .join(".local/share/agent-cli/logs")
        .join(&agent_id);
    if !log_dir.exists() {
        bail!(
            "log directory not found for agent '{domain}' ({agent_id}): {}",
            log_dir.display()
        );
    }

    let mut entries: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(&log_dir)
        .map_err(|e| anyhow::anyhow!("readdir {}: {e}", log_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".jsonl"))
        .filter_map(|e| {
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((mtime, e.path()))
        })
        .collect();
    entries.sort_by_key(|(t, _)| *t);
    let latest = entries
        .into_iter()
        .next_back()
        .map(|(_, p)| p)
        .ok_or_else(|| anyhow::anyhow!("no .jsonl files under {}", log_dir.display()))?;
    Ok(latest)
}

/// Detect whether an agent for `domain` is registered with agent-cli (Phase 49).
async fn is_agent_alive(domain: &str) -> bool {
    let output = Command::new("agent-cli")
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await;
    let Ok(out) = output else { return false; };
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("ID") || trimmed.starts_with('-') {
            continue;
        }
        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        if fields.len() >= 2 && fields[1] == domain {
            return true;
        }
    }
    false
}

/// Mirror an agent-cli structured JSONL log into the workspace agent.log (Phase 49).
///
/// The workspace `agent.log` is normally just a redirect of agent-cli's stdout
/// (banner + occasional notices). agent-cli writes its real activity (thinking,
/// tool_call, tool_result, peer_prompt, assistant) into a separate JSONL file
/// under `~/.local/share/agent-cli/logs/<agent-id>/`. This task polls that
/// JSONL and appends human-readable summary lines to the workspace agent.log
/// so users who run `cat .hestia/workspaces/ai/agent.log` see live activity.
async fn mirror_agent_log(domain: &str) -> Result<()> {
    use std::io::Write;
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    let workspace_log = workspace_path(domain).join("agent.log");

    // Wait for agent-cli to register and produce its structured log.
    let timeout = std::time::Duration::from_secs(60);
    let start = std::time::Instant::now();
    let log_path = loop {
        if let Ok(p) = resolve_agent_log_path(domain).await {
            break p;
        }
        if start.elapsed() > timeout {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    };

    // Announce the mirror is active (one-line marker).
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&workspace_log)
    {
        let _ = writeln!(f, "[mirror] phase49 active, source={}", log_path.display());
    }

    let mut last_pos: u64 = 0;
    let mut thinking_count: u64 = 0;
    let mut thinking_last_emit: u64 = 0;
    let mut buf = String::new();

    loop {
        if !is_agent_alive(domain).await {
            if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&workspace_log) {
                let _ = writeln!(f, "[mirror] agent stopped, exiting");
            }
            return Ok(());
        }

        let size = match std::fs::metadata(&log_path) {
            Ok(m) => m.len(),
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                continue;
            }
        };

        if size > last_pos {
            let mut file = match tokio::fs::File::open(&log_path).await {
                Ok(f) => f,
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    continue;
                }
            };
            let _ = file.seek(std::io::SeekFrom::Start(last_pos)).await;
            buf.clear();
            let _ = file.read_to_string(&mut buf).await;
            last_pos = size;

            let Ok(mut out) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&workspace_log)
            else {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                continue;
            };

            for line in buf.lines() {
                let Ok(ev) = serde_json::from_str::<serde_json::Value>(line) else { continue; };
                let kind = ev.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
                match kind {
                    "thinking" => {
                        thinking_count += 1;
                        if thinking_count - thinking_last_emit >= 50 {
                            let snippet: String = ev
                                .get("text")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .chars()
                                .take(80)
                                .collect();
                            let _ = writeln!(
                                out,
                                "[mirror][thinking#{}] {}",
                                thinking_count,
                                snippet.trim()
                            );
                            thinking_last_emit = thinking_count;
                        }
                    }
                    "tool_call" => {
                        let name = ev.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let args_text = ev.get("args").map(|v| v.to_string()).unwrap_or_default();
                        let args_short: String = args_text.chars().take(160).collect();
                        let _ = writeln!(out, "[mirror][tool_call] {} args={}", name, args_short);
                    }
                    "tool_result" => {
                        let name = ev.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let ok = ev.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                        let _ = writeln!(out, "[mirror][tool_result] {} ok={}", name, ok);
                    }
                    "peer_prompt" => {
                        let from = ev.get("from").and_then(|v| v.as_str()).unwrap_or("?");
                        let _ = writeln!(out, "[mirror][peer_prompt] from={}", from);
                    }
                    "assistant" => {
                        let snippet: String = ev
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .chars()
                            .take(160)
                            .collect();
                        let _ = writeln!(out, "[mirror][assistant] {}", snippet.trim());
                    }
                    _ => {}
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

/// Tail an agent's structured log with simple human-readable formatting (Phase 48).
async fn tail_agent_log(domain: &str, path_only: bool) -> Result<()> {
    let path = resolve_agent_log_path(domain).await?;
    if path_only {
        println!("{}", path.display());
        return Ok(());
    }
    eprintln!("[hestia tail {domain}] streaming {}", path.display());
    eprintln!("[hestia tail {domain}] (Ctrl+C to stop)");

    // Use `tail -F -n +1` to read from the start and follow.
    let status = Command::new("tail")
        .arg("-F")
        .arg("-n")
        .arg("+1")
        .arg(&path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("failed to spawn tail: {e}"))?;
    if !status.success() {
        bail!("tail exited with {status}");
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
        Commands::Tail { domain, path_only } => tail_agent_log(&domain, path_only).await?,
        Commands::Mirror { domain } => mirror_agent_log(&domain).await?,
        Commands::SpawnSubagent { persona, name } => {
            // Persona arg may be either a bare filename root (e.g. "rtl-coder")
            // or a full filename ("rtl-coder.md"). Strip the .md suffix so
            // persona_path() can re-add it consistently.
            let persona_root = persona.strip_suffix(".md").unwrap_or(&persona);
            spawn_agent_cli(persona_root, &name).await?;
        }
    }

    Ok(())
}