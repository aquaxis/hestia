use anyhow::{bail, Result};
use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::FromRawFd;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, SystemTime};
use tokio::process::Command;

mod monitor;

/// Subset of `.hestia/config.toml` consumed by `hestia start`.
///
/// Only the `[agent_cli]` section is read here so we can pass `--provider` /
/// `--model` to `agent-cli run` and prevent the user's global agent-cli
/// config (`~/.config/agent-cli/config.toml`) from silently overriding the
/// project's choice (Phase 24).
#[derive(Debug, Default, Deserialize)]
pub(crate) struct HestiaConfig {
    #[serde(default)]
    agent_cli: AgentCliConfig,
    /// Phase 113 — `[engine]` セクション。peer 駆動エンジンを agent-cli /
    /// claude-cli-shim から選択する。未設定時は `agent_cli` 既定で後方互換。
    #[serde(default)]
    pub(crate) engine: EngineConfig,
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

/// Phase 113 — Engine 切替設定。
///
/// `.hestia/config.toml` の `[engine]` セクションから読込:
/// ```toml
/// [engine]
/// type = "agent_cli"     # "agent_cli" (既定) | "claude_cli_shim"
/// binary = ""            # 省略時は type に応じた既定 path
/// registry_path = ""     # 省略時は engine 既定（library 経由は env で別途）
/// log_path = ""          # 省略時は engine 既定
/// ```
#[derive(Debug, Default, Deserialize)]
pub(crate) struct EngineConfig {
    #[serde(rename = "type", default = "default_engine_type")]
    pub(crate) type_: String,
    #[serde(default)]
    pub(crate) binary: Option<String>,
    #[serde(default)]
    pub(crate) registry_path: Option<PathBuf>,
    #[serde(default)]
    pub(crate) log_path: Option<PathBuf>,
}

fn default_engine_type() -> String {
    "agent_cli".to_string()
}

impl EngineConfig {
    /// engine_binary: explicit override > type 既定 (`agent-cli` / `claude-cli-shim`)
    pub(crate) fn binary_name(&self) -> &str {
        if let Some(b) = self.binary.as_deref() {
            if !b.is_empty() {
                return b;
            }
        }
        match self.type_.as_str() {
            "claude_cli_shim" => "claude-cli-shim",
            _ => "agent-cli",
        }
    }

    /// pgrep / monitor 用に process 名 (PATH を含まない) を返す。
    /// `binary_name` が絶対 path の場合は basename を返す。
    pub(crate) fn binary_basename(&self) -> &str {
        let name = self.binary_name();
        std::path::Path::new(name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(name)
    }

    /// engine subprocess に渡すべき env 変数のリストを構築する。
    /// hestia → claude-cli-shim 等への path override 伝達に使用。
    pub(crate) fn subprocess_env(&self) -> Vec<(&'static str, String)> {
        let mut out = Vec::new();
        out.push(("HESTIA_ENGINE_BINARY", self.binary_name().to_string()));
        if let Some(p) = &self.registry_path {
            out.push((
                "CLAUDE_CLI_SHIM_REGISTRY_PATH",
                p.to_string_lossy().to_string(),
            ));
        }
        if let Some(p) = &self.log_path {
            out.push((
                "CLAUDE_CLI_SHIM_LOG_PATH",
                p.to_string_lossy().to_string(),
            ));
        }
        out
    }
}

/// Read `.hestia/config.toml` from the current working directory.
/// Returns the parsed config, or an empty default if the file is absent or
/// the section is missing — this preserves backwards compatibility with
/// installations created before Phase 24.
pub(crate) fn load_hestia_config() -> HestiaConfig {
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
    /// Show status of all conductor daemons.
    ///
    /// By default, columns `ID NAME STATUS PROVIDER MODEL ROLE` are
    /// displayed. `STATUS` is one of IDLE / BUSY / WAITING / ERROR /
    /// STARTING / UNKNOWN, derived from each agent's recent activity.
    /// Pass `--all` to also include the `SKILLS` column.
    Status {
        /// Include the `SKILLS` column in the output.
        #[arg(long, default_value_t = false)]
        all: bool,
    },
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
    /// Forcefully terminate every hestia-started agent process (SIGKILL).
    ///
    /// More aggressive than `stop`: sends SIGKILL to every running
    /// `agent-cli run ...` child (conductors + sub-agents) and to every
    /// `hestia mirror ...` helper. Use when `stop` doesn't return cleanly,
    /// after a crash leaves stale child processes behind, or when an
    /// immediate full shutdown is required without waiting for graceful
    /// termination. The current `hestia kill` process is excluded so the
    /// command can complete and report a summary.
    Kill,
    /// (Phase 108) Display the live status of every running conductor and
    /// sub-agent on a refresh interval. The default mode redraws the screen
    /// every `--interval` seconds; pass `--once` for a single snapshot
    /// (equivalent to `hestia status`).
    Monitor {
        /// Refresh interval in seconds (clamped to 1..=60).
        #[arg(long, default_value_t = 2)]
        interval: u64,
        /// Print one frame and exit (alias for `hestia status`).
        #[arg(long, default_value_t = false)]
        once: bool,
        /// Include the `SKILLS` column in the output.
        #[arg(long, default_value_t = false)]
        all: bool,
    },
    /// (Internal, Phase 108) Long-running watchdog spawned by `hestia start ai`.
    /// Periodically polls every conductor / sub-agent under ai-conductor and,
    /// when *all* of them are simultaneously stopped with pending tasks
    /// (per `<workspace>/<peer>/task_status.md`), sends a resume instruction
    /// via `agent-cli send`. Not intended to be invoked manually.
    #[command(hide = true, name = "monitor-daemon")]
    MonitorDaemon,
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
// Phase 93: 起動モデル根本再設計 — `hestia start` で ai-conductor のみ起動
// （旧 18 件 / Phase 91 9 件 → 3 件常駐: ai + ai-designer + ai-reviewer）。
// domain conductor (rtl/fpga/asic/pcb/hal/apps/debug/rag) およびその sub-agent は
// ai-conductor が dispatch 時に on-demand 起動する経路に変更。
//
// ai-conductor の常駐サブエージェントは ai-designer + ai-reviewer の 2 件のみ:
// - ai-designer: 人間指示の仕様分解担当
// - ai-reviewer: 仕様の妥当性確認担当
const RESIDENT_SUB_AGENTS: &[(&str, &[(&str, &str)])] = &[
    ("ai", &[
        ("ai-designer", "ai-designer"),
        ("ai-reviewer", "ai-reviewer"),
    ]),
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

/// Phase 81 → Phase 92 — Initialize per-peer hestia workspace.
///
/// Phase 22 P-1 (rules 隔離) → Phase 57 P-2 (`.aiprj/rules` symlink 共有)
/// → Phase 81 **P-3** (`.hestia/rules/` への hestia agent 向け解釈変更版を配置、
/// `.aiprj/` 直接参照を排除) → Phase 89 (`.hestia/rules/` から
/// `<workspace>/instruction.md` 取得記述削除) → Phase 91 (起動規約を
/// combined execution に変更) → **Phase 92** (`<workspace>/instruction.md`
/// placeholder 生成を完全廃止) の進化に伴うロジック変更。
///
/// Phase 92 で workspace ディレクトリ作成のみを担う関数に simplify。agent は
/// 上位指示を peer prompt 経由のみで受信し、3 文書 (`requirements.md` /
/// `design.md` / `tasks.md`) は agent 自身が setup_ai サイクル時に必要に応じて
/// fs_write で作成する（per-agent / 共用ではない / Phase 91 遵守必須化 +
/// Phase 92 明確化）。
///
/// Failures are non-fatal — they're logged but don't block conductor startup.
fn init_hestia_workspace(peer_name: &str) {
    let workspace = workspace_path(peer_name);
    if let Err(e) = std::fs::create_dir_all(&workspace) {
        eprintln!("[warn] failed to create {}: {e}", workspace.display());
    }
    // Phase 92: instruction.md placeholder 生成を廃止。
    // 旧 Phase 81 placeholder は agent が読み込まない dead file 状態だった
    // （Phase 89 で .hestia/rules/ から取得記述削除、Phase 91 で combined execution
    // に変更された結果）。Phase 92 で生成自体を廃止し、filesystem を simplify。
    let _ = peer_name; // peer_name は logging で使われていたが Phase 92 で不要化
}

/// Phase 109 — `agent-cli list` の stdout から既登録 peer 名集合を抽出する純関数。
/// NAME 列（2 列目）を完全一致で集める。`agent-XXX` で始まる ID 行のみ対象。
/// Phase 110 で monitor.rs から呼び出すため `pub(crate)` 化。
pub(crate) fn registered_peer_names(stdout: &str) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    for line in stdout.lines().skip(1) {
        let mut it = line.split_whitespace();
        let Some(id) = it.next() else { continue };
        let Some(name) = it.next() else { continue };
        if id.starts_with("agent-") {
            out.insert(name.to_string());
        }
    }
    out
}

/// Phase 109 — `peer_name` が既に `agent-cli list` 上に登録されているかを判定。
/// agent-cli 子プロセスの実行に失敗した場合は `false`（= 重複なし）を返し、
/// fallback で従来の spawn 経路に進ませる（誤抑制を避ける）。
async fn peer_already_registered(peer_name: &str) -> bool {
    let cfg = load_hestia_config();
    let Ok(out) = Command::new(cfg.engine.binary_name())
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
    else {
        return false;
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    registered_peer_names(&stdout).contains(peer_name)
}

/// Phase 109 — `hestia monitor-daemon` 子プロセスが既に走っているかを `pgrep -f`
/// で判定する。失敗時は `false`（重複なし扱い）。
async fn monitor_daemon_already_running() -> bool {
    Command::new("pgrep")
        .arg("-f")
        .arg("hestia monitor-daemon")
        .output()
        .await
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

/// Phase 55 — Spawn an agent-cli process for a given persona file + peer name.
/// Used by `start_conductor` (for the main conductor and resident sub-agents)
/// and by the hidden `spawn-subagent` subcommand (for handler-driven dynamic
/// sub-agents like `rtl-coder-{module}`). Creates a per-peer workspace under
/// `.hestia/workspaces/<peer>/`, sets up FIFO stdin so the agent doesn't EOF,
/// redirects stdout/stderr to `agent.log`, and (best-effort) spawns a mirror
/// helper for log visibility.
///
/// Phase 109: 関数冒頭で `agent-cli list` を確認し、対象 peer 名が既登録なら
/// warn ログのみで no-op return する（重複 spawn を物理的に防ぐ）。
/// Phase 110: monitor.rs の rescue 経路から呼び出すため `pub(crate)` 化。
pub(crate) async fn spawn_agent_cli(persona_filename_root: &str, peer_name: &str) -> Result<()> {
    let persona = persona_path(persona_filename_root);
    if !persona.exists() {
        bail!("persona file not found: {}", persona.display());
    }

    // Phase 109 — 重複 spawn 防止
    if peer_already_registered(peer_name).await {
        eprintln!(
            "[Phase 109] peer '{peer_name}' is already registered — skipping duplicate spawn"
        );
        return Ok(());
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
    let engine_bin = config.engine.binary_name().to_string();

    println!(
        "Starting {} --name {} --persona {} ...",
        engine_bin, peer_name, persona.display()
    );

    let mut cmd = Command::new(&engine_bin);
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
    for (k, v) in config.engine.subprocess_env() {
        cmd.env(k, v);
    }
    let _child = cmd
        .current_dir(&workdir)
        .stdin(Stdio::from(fifo_stdin))
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_file_stderr))
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn {engine_bin} for {peer_name}: {e}"))?;

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

    // Phase 84 — registry 登録確定まで待機。
    // agent-cli が起動して IPC ready になるまで最大 10 秒 polling。timeout した
    // 場合は warn のみで継続（fire-and-forget セマンティクスは維持しつつ、後続
    // 経路で peer_alive チェックが false になれば fallback / halt が発動する）。
    if !conductor_sdk::workspace::wait_for_registry(peer_name, 10_000) {
        eprintln!(
            "[warn] sub-agent {peer_name} did not register within 10s — \
             check {} for agent-cli startup errors",
            log_path.display()
        );
    }

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

    // Phase 109 — conductor 自身の重複 spawn 防止。
    // 既登録の場合は agent-cli spawn を skip するが、resident sub-agents の起動と
    // monitor-daemon spawn は継続する（個別に各 spawn 内で重複 check が走る）。
    let conductor_already_up = peer_already_registered(domain).await;
    if conductor_already_up {
        eprintln!(
            "[Phase 109] conductor '{domain}' is already registered — \
             skipping duplicate agent-cli spawn (resident sub-agents will still be checked)"
        );
    }

    // Phase 109 — conductor が既登録ならこのブロックをまるごと skip。
    // 既存挙動の hestia_self だけは下流（mirror / monitor-daemon spawn）でも
    // 必要なので block 外で resolve する。
    let hestia_self = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("hestia"));
    let log_path = workdir.join("agent.log");

    if !conductor_already_up {
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
        let log_file = std::fs::File::create(&log_path)
            .map_err(|e| anyhow::anyhow!("failed to create log file {}: {e}", log_path.display()))?;
        let log_file_stderr = log_file.try_clone()
            .map_err(|e| anyhow::anyhow!("failed to dup log file: {e}"))?;

        let config = load_hestia_config();
        let provider = config.agent_cli.provider_arg();
        let model = config.agent_cli.model.as_deref();
        let engine_bin = config.engine.binary_name().to_string();

        let provider_log = provider.as_deref().unwrap_or("(global default)");
        let model_log = model.unwrap_or("(global default)");
        println!(
            "Starting {} --name {} --persona {} --provider {} --model {} ...",
            engine_bin, domain, persona.display(), provider_log, model_log
        );

        let mut cmd = Command::new(&engine_bin);
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
        for (k, v) in config.engine.subprocess_env() {
            cmd.env(k, v);
        }
        let _child = cmd
            .current_dir(&workdir)
            .stdin(Stdio::from(fifo_stdin))
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(log_file_stderr))
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to spawn {engine_bin} for {domain}: {e}"))?;

        // Phase 49: spawn the structured-log mirror as a detached background helper.
        // Resolve hestia's own path (argv[0]) so we always invoke the same binary
        // that started us, regardless of $PATH ordering. Inherit our cwd (the
        // project root) — `mirror_agent_log` calls `workspace_path` which derives
        // the workspace from cwd, so we must NOT change directory here.
        let _mirror = Command::new(&hestia_self)
            .arg("mirror")
            .arg(domain)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to spawn mirror helper for {domain}: {e}"))?;

        // Phase 84 — registry 登録確定まで待機（main conductor）。
        // sub-agent spawn の前に conductor 自身が agent-cli registry に登録済である
        // ことを保証する。タイムアウト時は warn のみで sub-agent spawn に進む。
        if !conductor_sdk::workspace::wait_for_registry(domain, 15_000) {
            eprintln!(
                "[warn] conductor {domain} did not register within 15s — \
                 check {} for agent-cli startup errors",
                log_path.display()
            );
        }
    }

    // Phase 55 — launch resident sub-agents (planner / designer) for this
    // conductor. Failures are logged but not fatal: the conductor itself is
    // already up and Phase 54 design.v1 stubs gracefully fall back when a
    // sub-agent isn't reachable. Phase 84 では `spawn_agent_cli` 内で
    // `wait_for_registry` を呼び出し、起動完了を確認した上で次の peer に進む。
    if let Some((_, agents)) = RESIDENT_SUB_AGENTS.iter().find(|(d, _)| *d == domain) {
        for (persona_root, peer_name) in *agents {
            if let Err(e) = spawn_agent_cli(persona_root, peer_name).await {
                eprintln!("[warn] failed to start sub-agent {peer_name} for {domain}: {e}");
            }
        }
    }

    // Phase 108 — ai conductor 起動時に稼働監視デーモンを子プロセスとして spawn。
    // mirror helper と同じパターンで `hestia monitor-daemon` を background detach。
    // ai conductor は LLM peer として独立稼働し、監視デーモンは別プロセスとして
    // 30 秒周期で配下サブエージェント + 起動中 domain conductor の状態を polling し、
    // 全停止 + タスク残存時のみ `agent-cli send` で再開指示を発行する。
    //
    // Phase 109 — `pgrep` で既存デーモンを確認し、走っていれば spawn を skip する。
    if domain == "ai" {
        if monitor_daemon_already_running().await {
            eprintln!(
                "[Phase 109] hestia monitor-daemon is already running — skipping duplicate spawn"
            );
        } else {
            let monitor = Command::new(&hestia_self)
                .arg("monitor-daemon")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            match monitor {
                Ok(_) => println!("[Phase 108] hestia monitor-daemon spawned alongside ai-conductor"),
                Err(e) => eprintln!("[warn] failed to spawn monitor-daemon: {e}"),
            }
        }
    }

    Ok(())
}

async fn wait_for_ai_readiness() -> Result<()> {
    println!("Waiting for ai-conductor readiness ...");
    let timeout = std::time::Duration::from_secs(AI_READINESS_TIMEOUT_SECS);
    let start = std::time::Instant::now();
    let cfg = load_hestia_config();
    let engine_bin = cfg.engine.binary_name();

    while start.elapsed() < timeout {
        let output = Command::new(engine_bin)
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
    // Phase 93 起動モデル再設計:
    // `hestia start` (引数なし) は ai-conductor のみ起動する。
    // ai-conductor が人間指示を受信した時点で domain conductor (rtl/fpga/asic/...)
    // を on-demand 起動する経路に統一（spawn_conductor_on_demand 経由）。
    // 旧仕様の「全 9 conductor を並列起動」は廃止。手動起動が必要な場合は
    // `hestia start <domain>` で個別 spawn 可能（fallback 経路）。
    start_conductor("ai").await?;
    wait_for_ai_readiness().await?;

    println!("ai-conductor started (Phase 93: ai-conductor only at startup)");
    println!(
        "  → ai-conductor + ai-designer + ai-reviewer の 3 process が常駐起動"
    );
    println!(
        "  → domain conductor (rtl/fpga/asic/pcb/hal/apps/debug/rag) は ai-conductor が"
    );
    println!(
        "    dispatch 時に on-demand 起動 (Phase 93 起動モデル)"
    );
    let _ = GROUP1_DOMAINS; // Phase 93: 起動時 spawn から除外、参照のみ保持
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
    let cfg = load_hestia_config();
    let engine_bin = cfg.engine.binary_name();
    let output = Command::new(engine_bin)
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to run {engine_bin} list: {e}"))?;

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

/// Patterns that identify processes hestia is responsible for killing.
/// `pgrep -f <pattern>` is used to enumerate matching PIDs on the host.
const KILL_PATTERNS: &[&str] = &["agent-cli run", "hestia mirror", "hestia monitor-daemon"];

/// Pure helper: turn a list of `(pattern, pgrep_stdout)` pairs into the ordered
/// list of `(pattern, pid)` targets to SIGKILL, skipping the caller's own PID
/// and de-duplicating PIDs that match multiple patterns.
///
/// The function is intentionally I/O-free so it can be unit-tested without
/// spawning real processes. Lines that don't parse to a `u32` are silently
/// dropped; trailing whitespace and blank lines are tolerated.
fn select_kill_targets(
    pgrep_outputs: &[(&str, &str)],
    self_pid: u32,
) -> Vec<(String, u32)> {
    let mut seen: Vec<u32> = Vec::new();
    let mut targets: Vec<(String, u32)> = Vec::new();
    for (pattern, stdout) in pgrep_outputs {
        for line in stdout.lines() {
            let Ok(pid) = line.trim().parse::<u32>() else {
                continue;
            };
            if pid == self_pid {
                continue;
            }
            if seen.contains(&pid) {
                continue;
            }
            seen.push(pid);
            targets.push(((*pattern).to_string(), pid));
        }
    }
    targets
}

/// Force-terminate every hestia-started agent process (SIGKILL).
///
/// Implementation:
/// 1. Run `pgrep -f <pattern>` for each entry in [`KILL_PATTERNS`] and collect
///    the stdout. A pgrep failure is logged as a warning but doesn't abort the
///    overall command.
/// 2. Compute the kill list with [`select_kill_targets`]: own PID excluded,
///    duplicates removed across patterns.
/// 3. Send SIGKILL to each target via `libc::kill` and tally success / failure.
///    Per-PID failures (already exited, permission denied) are logged and
///    skipped — they don't fail the whole command.
async fn kill_all_processes() -> Result<()> {
    let mut outputs: Vec<(&str, String)> = Vec::with_capacity(KILL_PATTERNS.len());
    for pattern in KILL_PATTERNS {
        let result = Command::new("pgrep")
            .arg("-f")
            .arg(pattern)
            .output()
            .await;
        match result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
                outputs.push((pattern, stdout));
            }
            Err(e) => {
                eprintln!("[warn] pgrep failed for pattern '{pattern}': {e}");
                outputs.push((pattern, String::new()));
            }
        }
    }

    let outputs_ref: Vec<(&str, &str)> =
        outputs.iter().map(|(p, s)| (*p, s.as_str())).collect();
    let targets = select_kill_targets(&outputs_ref, std::process::id());

    let mut ok: u32 = 0;
    let mut ng: u32 = 0;
    for (pattern, pid) in &targets {
        let r = unsafe { libc::kill(*pid as i32, libc::SIGKILL) };
        if r == 0 {
            ok += 1;
            println!("Killed PID {pid} (matches '{pattern}')");
        } else {
            ng += 1;
            let err = std::io::Error::last_os_error();
            eprintln!("[warn] failed to kill PID {pid}: {err}");
        }
    }

    if targets.is_empty() {
        println!("No matching hestia processes found.");
    } else {
        println!("Killed {ok} process(es) (failed: {ng}).");
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

async fn show_status(all: bool) -> Result<()> {
    let cfg = load_hestia_config();
    let engine_bin = cfg.engine.binary_name();
    let output = Command::new(engine_bin)
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to run {engine_bin} list: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let now = SystemTime::now();
    let statuses = collect_agent_statuses(&stdout, now);
    print!("{}", transform_status_listing(&stdout, all, &statuses));

    if !output.status.success() {
        bail!("agent-cli list exited with {}", output.status);
    }
    Ok(())
}

/// Operational status of an agent, derived from its agent-cli structured log
/// and registry membership. Displayed as a `STATUS` column by `hestia status`.
///
/// Phase 110: `Think` ヴァリアントを追加し、`Waiting` の表示を `WAIT` に変更。
/// `thinking` event を `Busy`（tool 実行中）と区別する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentStatus {
    /// Agent is registered, last activity is a completed assistant reply.
    Idle,
    /// Agent log was modified within the busy window and the last event is
    /// `tool_call` / `tool_result` (tool 実行中、Phase 110 で thinking と分離).
    Busy,
    /// (Phase 110) Last event is `thinking` and mtime is recent (思考中).
    Think,
    /// Agent has accepted a user prompt but no assistant reply has appeared.
    Waiting,
    /// Last tool_result reported `ok = false`.
    Error,
    /// Agent registered but its JSONL log is empty / unreadable / absent.
    Starting,
    /// State could not be determined (e.g. log read failure).
    Unknown,
}

impl AgentStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Busy => "BUSY",
            Self::Think => "THINK",
            Self::Waiting => "WAIT",
            Self::Error => "ERROR",
            Self::Starting => "STARTING",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// Width reserved for the `STATUS` column. Equal to the longest status string
/// (`STARTING` = 8) so values are right-padded to a consistent width.
const STATUS_COLUMN_WIDTH: usize = 8;

/// One activity event extracted from an agent-cli structured JSONL log.
/// Only the `kind` and (for `tool_result`) the `ok` flag are needed to
/// classify the agent's status; the original timestamp is read separately
/// from the file's mtime so we don't depend on a date parsing crate.
#[derive(Debug, Clone)]
struct StatusEvent {
    kind: String,
    ok: Option<bool>,
}

/// Pure status classifier — easy to unit-test without touching the filesystem.
///
/// Phase 110: `thinking` event を `Think` に分岐、`tool_call` / `tool_result` のみ
/// `Busy` を返すよう細分化。
fn derive_status_from_log(events: &[StatusEvent], mtime_age: Duration) -> AgentStatus {
    let Some(last) = events.last() else {
        return AgentStatus::Starting;
    };
    if last.kind == "tool_result" && last.ok == Some(false) {
        return AgentStatus::Error;
    }
    if mtime_age < Duration::from_secs(30) {
        match last.kind.as_str() {
            "thinking" => return AgentStatus::Think,
            "tool_call" | "tool_result" => return AgentStatus::Busy,
            _ => {}
        }
    }
    match last.kind.as_str() {
        "user" => AgentStatus::Waiting,
        "assistant" => AgentStatus::Idle,
        _ => AgentStatus::Idle,
    }
}

/// Resolve the latest `*.jsonl` file under `~/.local/share/agent-cli/logs/<agent_id>/`
/// and classify the agent's current status from its tail. Returns
/// [`AgentStatus::Starting`] when the directory or jsonl is missing,
/// [`AgentStatus::Unknown`] only for unexpected I/O errors.
fn derive_agent_status(agent_id: &str, now: SystemTime) -> AgentStatus {
    let Some(home) = dirs::home_dir() else {
        return AgentStatus::Unknown;
    };
    let log_dir = home.join(".local/share/agent-cli/logs").join(agent_id);
    let Ok(entries) = std::fs::read_dir(&log_dir) else {
        return AgentStatus::Starting;
    };
    let latest = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".jsonl"))
        .filter_map(|e| {
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((mtime, e.path()))
        })
        .max_by_key(|(t, _)| *t);
    let Some((mtime, path)) = latest else {
        return AgentStatus::Starting;
    };
    let mtime_age = now.duration_since(mtime).unwrap_or(Duration::ZERO);
    let tail = match read_tail_string(&path, 8192) {
        Ok(s) => s,
        Err(_) => return AgentStatus::Unknown,
    };
    let events = parse_status_events(&tail);
    derive_status_from_log(&events, mtime_age)
}

/// Read the trailing `max_bytes` of `path` as UTF-8. The boundary is realigned
/// to the next valid char boundary so a partial multi-byte sequence at the
/// start is dropped silently.
fn read_tail_string(path: &Path, max_bytes: u64) -> std::io::Result<String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path)?;
    let len = f.metadata()?.len();
    let from = len.saturating_sub(max_bytes);
    f.seek(SeekFrom::Start(from))?;
    let mut buf = Vec::with_capacity(max_bytes.min(len) as usize);
    f.read_to_end(&mut buf)?;
    let mut start = 0usize;
    while start < buf.len() && (buf[start] & 0xC0) == 0x80 {
        start += 1;
    }
    Ok(String::from_utf8_lossy(&buf[start..]).into_owned())
}

/// Parse newline-delimited JSON event lines from a JSONL tail. The very first
/// line is dropped if its parse fails, since a tail-seek can split a line in
/// the middle. Subsequent unparseable lines are skipped silently.
fn parse_status_events(tail: &str) -> Vec<StatusEvent> {
    let mut events = Vec::new();
    let mut lines = tail.lines();
    if let Some(first) = lines.next() {
        if let Some(ev) = parse_event_line(first) {
            events.push(ev);
        }
    }
    for line in lines {
        if let Some(ev) = parse_event_line(line) {
            events.push(ev);
        }
    }
    events
}

fn parse_event_line(line: &str) -> Option<StatusEvent> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let kind = v.get("kind")?.as_str()?.to_string();
    let ok = v.get("ok").and_then(|x| x.as_bool());
    Some(StatusEvent { kind, ok })
}

/// Resolve the `STATUS` value for every data row in `agent-cli list` output by
/// reading each agent's structured log. Header / blank / non-`agent-*` rows
/// are ignored.
fn collect_agent_statuses(stdout: &str, now: SystemTime) -> HashMap<String, AgentStatus> {
    let mut out = HashMap::new();
    for line in stdout.lines().skip(1) {
        let id = line.split_whitespace().next().unwrap_or("");
        if id.starts_with("agent-") && !out.contains_key(id) {
            let status = derive_agent_status(id, now);
            out.insert(id.to_string(), status);
        }
    }
    out
}

/// Drop the `SKILLS` column when `all` is false, then insert a `STATUS`
/// column right after `NAME`. The end result for the default mode is
/// `ID NAME STATUS PROVIDER MODEL ROLE`; with `--all` the `SKILLS` column is
/// kept and we get `ID NAME STATUS PROVIDER MODEL ROLE SKILLS`.
///
/// Column geometry is inferred from "≥2 consecutive spaces" inter-column gaps
/// (single spaces only occur inside a column value), so multi-byte ROLE
/// values don't throw off the cut points.
///
/// `statuses` maps an agent ID to its derived [`AgentStatus`]; rows whose ID
/// is not present (e.g. malformed) are rendered as `UNKNOWN`. The header row
/// is always rendered with literal `STATUS`. If the header is missing `ID` or
/// has fewer than two column separators, the input is returned unmodified.
fn transform_status_listing(
    stdout: &str,
    all: bool,
    statuses: &HashMap<String, AgentStatus>,
) -> String {
    if stdout.is_empty() {
        return String::new();
    }
    let header = stdout.lines().next().unwrap_or("");
    if !header.contains("ID") {
        return stdout.to_string();
    }
    let intermediate = strip_skills_column(stdout, all);
    insert_status_column(&intermediate, statuses)
}

/// First half of [`transform_status_listing`]: drop the trailing `SKILLS`
/// column when `all` is false, otherwise pass the input through unchanged.
fn strip_skills_column(stdout: &str, all: bool) -> String {
    if all || !stdout.lines().next().unwrap_or("").contains("SKILLS") {
        return stdout.to_string();
    }
    let header = stdout.lines().next().unwrap_or("");
    let separators = count_column_separators(header);
    if separators == 0 {
        return stdout.to_string();
    }
    let trailing_newline = stdout.ends_with('\n');
    let mut out = String::with_capacity(stdout.len());
    for line in stdout.lines() {
        out.push_str(cut_before_nth_separator(line, separators));
        out.push('\n');
    }
    if !trailing_newline && out.ends_with('\n') {
        out.pop();
    }
    out
}

/// Second half of [`transform_status_listing`]: insert a `STATUS` column at
/// the position of the column **after** the `NAME` column (i.e. the byte
/// offset where `PROVIDER` would start). The original `NAME` column padding
/// is kept intact so `STATUS` values line up vertically across rows even when
/// `NAME` values have differing display widths.
fn insert_status_column(text: &str, statuses: &HashMap<String, AgentStatus>) -> String {
    let trailing_newline = text.ends_with('\n');
    let mut out = String::with_capacity(text.len() + 256);
    for (idx, line) in text.lines().enumerate() {
        match nth_separator_end(line, 2) {
            Some(p) => {
                let status_str = if idx == 0 {
                    "STATUS"
                } else {
                    let id = line.split_whitespace().next().unwrap_or("");
                    statuses
                        .get(id)
                        .map(|s| s.as_str())
                        .unwrap_or(AgentStatus::Unknown.as_str())
                };
                out.push_str(&line[..p]);
                out.push_str(status_str);
                let pad = STATUS_COLUMN_WIDTH.saturating_sub(status_str.len());
                for _ in 0..pad {
                    out.push(' ');
                }
                out.push_str("  ");
                out.push_str(&line[p..]);
            }
            None => out.push_str(line),
        }
        out.push('\n');
    }
    if !trailing_newline && out.ends_with('\n') {
        out.pop();
    }
    out
}

/// Byte offset just after the `n`-th inter-column gap — i.e. the first
/// byte of the (`n` + 1)-th column. Counts only gaps followed by content
/// (mirrors [`count_column_separators`]).
fn nth_separator_end(line: &str, n: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut seen = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' {
            let start = i;
            while i < bytes.len() && bytes[i] == b' ' {
                i += 1;
            }
            if i - start >= 2 && i < bytes.len() {
                seen += 1;
                if seen == n {
                    return Some(i);
                }
            }
        } else {
            i += 1;
        }
    }
    None
}

/// Number of 2+space runs in `line` that are followed by non-whitespace
/// content. Trailing whitespace runs are excluded so the header (which is
/// often padded out with spaces after `SKILLS`) reports the true column count
/// minus one (= number of inter-column separators).
fn count_column_separators(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' {
            let start = i;
            while i < bytes.len() && bytes[i] == b' ' {
                i += 1;
            }
            if i - start >= 2 && i < bytes.len() {
                count += 1;
            }
        } else {
            i += 1;
        }
    }
    count
}

/// Cut `line` at the start of its `n`-th 2+space run and trim trailing
/// whitespace from the head. If `line` has fewer than `n` such runs, the
/// whole line (trimmed) is returned.
fn cut_before_nth_separator(line: &str, n: usize) -> &str {
    let bytes = line.as_bytes();
    let mut seen = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' {
            let start = i;
            while i < bytes.len() && bytes[i] == b' ' {
                i += 1;
            }
            if i - start >= 2 {
                seen += 1;
                if seen == n {
                    return line[..start].trim_end();
                }
            }
        } else {
            i += 1;
        }
    }
    line.trim_end()
}

/// Resolve the latest agent-cli structured-log path for `domain` (Phase 48).
///
/// Looks up the agent-id of the running agent whose `name` column matches
/// `domain` via `agent-cli list`, then locates the most recently modified
/// `*.jsonl` under `~/.local/share/agent-cli/logs/<agent-id>/`.
async fn resolve_agent_log_path(domain: &str) -> Result<PathBuf> {
    let cfg = load_hestia_config();
    let engine_bin = cfg.engine.binary_name();
    let output = Command::new(engine_bin)
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to run {engine_bin} list: {e}"))?;
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
    let cfg = load_hestia_config();
    let engine_bin = cfg.engine.binary_name();
    let output = Command::new(engine_bin)
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
        Commands::Status { all } => show_status(all).await?,
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
        Commands::Kill => kill_all_processes().await?,
        Commands::Monitor { interval, once, all } => {
            monitor::run_monitor(interval, once, all).await?
        }
        Commands::MonitorDaemon => monitor::run_monitor_daemon().await?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        derive_status_from_log, parse_status_events, registered_peer_names,
        select_kill_targets, transform_status_listing, AgentStatus, StatusEvent,
    };
    use std::collections::HashMap;
    use std::time::Duration;

    const HEADER: &str =
        "ID                                NAME       PROVIDER  MODEL          ROLE             SKILLS\n";
    const ROW_LONG: &str =
        "agent-01KQX72WJY3Z59RN77YXB9Z02P  ai         ollama    glm-5.1:cloud  Hestia メタ      指示テキスト解析, DAG 構築\n";
    const ROW_NO_SKILLS: &str =
        "agent-01KQX72WT3DY2GKWSDDXA9QK0K  ai-review  ollama    glm-5.1:cloud  AI reviewer      \n";

    fn statuses_for(pairs: &[(&str, AgentStatus)]) -> HashMap<String, AgentStatus> {
        pairs.iter().map(|(k, v)| ((*k).to_string(), *v)).collect()
    }

    // ─── transform_status_listing ────────────────────────────────────────

    #[test]
    fn transform_inserts_status_and_drops_skills_by_default() {
        let input = format!("{HEADER}{ROW_LONG}{ROW_NO_SKILLS}");
        let map = statuses_for(&[
            ("agent-01KQX72WJY3Z59RN77YXB9Z02P", AgentStatus::Busy),
            ("agent-01KQX72WT3DY2GKWSDDXA9QK0K", AgentStatus::Idle),
        ]);
        let out = transform_status_listing(&input, false, &map);
        assert!(!out.contains("SKILLS"), "SKILLS header dropped");
        assert!(!out.contains("DAG 構築"), "SKILLS payload dropped");
        assert!(out.contains("STATUS"), "STATUS header inserted");
        assert!(out.contains("BUSY"), "BUSY status row visible");
        assert!(out.contains("IDLE"), "IDLE status row visible");
        assert!(out.contains("Hestia メタ"), "ROLE value retained");
        assert!(out.ends_with('\n'), "trailing newline preserved");
    }

    #[test]
    fn transform_keeps_skills_column_when_all() {
        let input = format!("{HEADER}{ROW_LONG}");
        let map = statuses_for(&[(
            "agent-01KQX72WJY3Z59RN77YXB9Z02P",
            AgentStatus::Idle,
        )]);
        let out = transform_status_listing(&input, true, &map);
        assert!(out.contains("SKILLS"), "SKILLS header kept");
        assert!(out.contains("DAG 構築"), "SKILLS payload kept");
        assert!(out.contains("STATUS"), "STATUS still inserted");
        assert!(out.contains("IDLE"));
    }

    #[test]
    fn transform_uses_unknown_for_missing_id() {
        let input = format!("{HEADER}{ROW_LONG}");
        let map: HashMap<String, AgentStatus> = HashMap::new();
        let out = transform_status_listing(&input, false, &map);
        assert!(out.contains("UNKNOWN"), "missing ID falls back to UNKNOWN");
    }

    #[test]
    fn transform_passes_through_when_header_missing_id() {
        let input = "no header here\nrow\n";
        let map: HashMap<String, AgentStatus> = HashMap::new();
        let out = transform_status_listing(input, false, &map);
        assert_eq!(out, input);
    }

    #[test]
    fn transform_handles_empty_input() {
        let map: HashMap<String, AgentStatus> = HashMap::new();
        assert_eq!(transform_status_listing("", false, &map), "");
        assert_eq!(transform_status_listing("", true, &map), "");
    }

    #[test]
    fn transform_preserves_no_trailing_newline() {
        let raw = format!("{HEADER}{ROW_LONG}");
        let input = raw.trim_end_matches('\n');
        let map = statuses_for(&[(
            "agent-01KQX72WJY3Z59RN77YXB9Z02P",
            AgentStatus::Idle,
        )]);
        let out = transform_status_listing(input, false, &map);
        assert!(!out.ends_with('\n'), "no trailing newline preserved");
        assert!(out.contains("STATUS"));
    }

    // ─── derive_status_from_log ──────────────────────────────────────────

    fn ev(kind: &str, ok: Option<bool>) -> StatusEvent {
        StatusEvent {
            kind: kind.to_string(),
            ok,
        }
    }

    #[test]
    fn derive_starting_when_no_events() {
        assert_eq!(
            derive_status_from_log(&[], Duration::from_secs(0)),
            AgentStatus::Starting
        );
    }

    #[test]
    fn derive_idle_when_last_assistant_and_old() {
        let events = vec![ev("user", None), ev("assistant", None)];
        assert_eq!(
            derive_status_from_log(&events, Duration::from_secs(120)),
            AgentStatus::Idle
        );
    }

    #[test]
    fn derive_waiting_when_last_user() {
        let events = vec![ev("assistant", None), ev("user", None)];
        assert_eq!(
            derive_status_from_log(&events, Duration::from_secs(120)),
            AgentStatus::Waiting
        );
    }

    #[test]
    fn derive_busy_when_recent_tool_call() {
        // Phase 110: tool_call は引き続き Busy。
        let events = vec![ev("user", None), ev("thinking", None), ev("tool_call", None)];
        assert_eq!(
            derive_status_from_log(&events, Duration::from_secs(5)),
            AgentStatus::Busy
        );
    }

    #[test]
    fn derive_think_when_recent_thinking() {
        // Phase 110: 末尾が thinking かつ mtime recent なら Think.
        let events = vec![ev("user", None), ev("thinking", None)];
        assert_eq!(
            derive_status_from_log(&events, Duration::from_secs(5)),
            AgentStatus::Think
        );
    }

    #[test]
    fn derive_error_when_tool_result_failed() {
        let events = vec![ev("tool_call", None), ev("tool_result", Some(false))];
        assert_eq!(
            derive_status_from_log(&events, Duration::from_secs(1)),
            AgentStatus::Error
        );
    }

    #[test]
    fn derive_think_overrides_old_assistant_with_recent_thinking() {
        // Phase 110: mtime recent + 末尾 thinking → Think (旧テストでは Busy だった).
        let events = vec![
            ev("assistant", None),
            ev("user", None),
            ev("thinking", None),
        ];
        assert_eq!(
            derive_status_from_log(&events, Duration::from_secs(10)),
            AgentStatus::Think
        );
    }

    #[test]
    fn agent_status_as_str_uses_wait_and_think_phase110() {
        // Phase 110: Waiting の表記は WAIT / Think は THINK / 他は据え置き。
        assert_eq!(AgentStatus::Idle.as_str(), "IDLE");
        assert_eq!(AgentStatus::Busy.as_str(), "BUSY");
        assert_eq!(AgentStatus::Think.as_str(), "THINK");
        assert_eq!(AgentStatus::Waiting.as_str(), "WAIT");
        assert_eq!(AgentStatus::Error.as_str(), "ERROR");
        assert_eq!(AgentStatus::Starting.as_str(), "STARTING");
        assert_eq!(AgentStatus::Unknown.as_str(), "UNKNOWN");
    }

    // ─── parse_status_events ─────────────────────────────────────────────

    #[test]
    fn parse_skips_truncated_first_line() {
        let tail = "{\"kind\": broken,\n{\"ts\":\"x\",\"kind\":\"assistant\",\"text\":\"hi\"}\n";
        let events = parse_status_events(tail);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "assistant");
    }

    #[test]
    fn parse_extracts_ok_field_for_tool_result() {
        let tail = "{\"kind\":\"tool_result\",\"ok\":false}\n";
        let events = parse_status_events(tail);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "tool_result");
        assert_eq!(events[0].ok, Some(false));
    }

    // ─── select_kill_targets ─────────────────────────────────────────────

    #[test]
    fn select_kill_targets_returns_empty_for_empty_input() {
        let targets = select_kill_targets(&[], 999);
        assert!(targets.is_empty());
    }

    #[test]
    fn select_kill_targets_excludes_self_pid() {
        let inputs = [("agent-cli run", "999\n1000\n")];
        let targets = select_kill_targets(&inputs, 999);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].0, "agent-cli run");
        assert_eq!(targets[0].1, 1000);
    }

    #[test]
    fn select_kill_targets_dedups_pids_across_patterns() {
        // PID 100 matches both patterns; should appear only once, paired with
        // the pattern that found it first.
        let inputs = [
            ("agent-cli run", "100\n200\n"),
            ("hestia mirror", "100\n300\n"),
        ];
        let targets = select_kill_targets(&inputs, 999);
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0], ("agent-cli run".to_string(), 100));
        assert_eq!(targets[1], ("agent-cli run".to_string(), 200));
        assert_eq!(targets[2], ("hestia mirror".to_string(), 300));
    }

    #[test]
    fn select_kill_targets_ignores_non_numeric_lines_and_whitespace() {
        let inputs = [("agent-cli run", "  300  \nfoo\n\n400\n")];
        let targets = select_kill_targets(&inputs, 999);
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].1, 300);
        assert_eq!(targets[1].1, 400);
    }

    // ─── registered_peer_names (Phase 109) ─────────────────────────────────

    #[test]
    fn registered_peer_names_empty_input() {
        assert!(registered_peer_names("").is_empty());
    }

    #[test]
    fn registered_peer_names_header_only() {
        let input = "ID  NAME  PROVIDER  MODEL  ROLE\n";
        assert!(registered_peer_names(input).is_empty());
    }

    #[test]
    fn registered_peer_names_picks_agent_rows() {
        let input = "\
ID                              NAME         PROVIDER  MODEL          ROLE
agent-AAA                       ai           ollama    glm-5.1:cloud  meta
agent-BBB                       ai-designer  ollama    glm-5.1:cloud  designer
agent-CCC                       ai-reviewer  ollama    glm-5.1:cloud  reviewer
";
        let got = registered_peer_names(input);
        assert_eq!(got.len(), 3);
        assert!(got.contains("ai"));
        assert!(got.contains("ai-designer"));
        assert!(got.contains("ai-reviewer"));
    }

    #[test]
    fn registered_peer_names_dedups_repeated_names() {
        // Phase 109 で防止すべき状態の入力（同名 peer が複数登録）。
        // 集合化することで重複は 1 件に収束する。
        let input = "\
ID                              NAME         PROVIDER  MODEL          ROLE
agent-AAA                       ai-reviewer  ollama    glm-5.1:cloud  reviewer
agent-BBB                       ai-reviewer  ollama    glm-5.1:cloud  reviewer
agent-CCC                       ai-reviewer  ollama    glm-5.1:cloud  reviewer
";
        let got = registered_peer_names(input);
        assert_eq!(got.len(), 1);
        assert!(got.contains("ai-reviewer"));
    }

    #[test]
    fn registered_peer_names_ignores_non_agent_rows() {
        let input = "\
ID                              NAME         PROVIDER  MODEL          ROLE
0001                            ai           ollama    glm-5.1:cloud  meta
agent-BBB                       ai-designer  ollama    glm-5.1:cloud  designer
\n
";
        let got = registered_peer_names(input);
        assert_eq!(got.len(), 1);
        assert!(got.contains("ai-designer"));
    }

    #[test]
    fn registered_peer_names_handles_skills_column() {
        // SKILLS 列があっても NAME (2 列目) のみ拾うので影響なし。
        let input = "\
ID                              NAME         PROVIDER  MODEL          ROLE             SKILLS
agent-AAA                       ai-reviewer  ollama    glm-5.1:cloud  reviewer         a, b, c
";
        let got = registered_peer_names(input);
        assert_eq!(got.len(), 1);
        assert!(got.contains("ai-reviewer"));
    }
}