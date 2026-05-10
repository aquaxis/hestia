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
    /// Phase 128 — `[concurrency]` セクション。`hestia start` が子 conductor
    /// process の env として export することで Phase 126 の
    /// `ConductorLimiter::from_env` / `AgentManager::with_default_cap` が
    /// 反映する。親プロセス env が既に設定されている場合は env を優先（後方互換）。
    #[serde(default)]
    pub(crate) concurrency: ConcurrencyConfig,
}

/// Phase 128 — `[concurrency]` セクションの schema。
/// 全フィールド optional。未指定時は library 既定（global=8 / dispatch=2 /
/// per_conductor=4 / timeout=600s）が使われる。
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ConcurrencyConfig {
    #[serde(default)]
    pub(crate) global_max: Option<usize>,
    #[serde(default)]
    pub(crate) ai_conductor_dispatch_max: Option<usize>,
    #[serde(default)]
    pub(crate) per_conductor_max: Option<usize>,
    #[serde(default)]
    pub(crate) acquire_timeout_secs: Option<u64>,
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

    /// Phase 115 — engine type に応じて `--provider` 引数を調整する。
    ///
    /// - `claude_cli_shim` engine: claude 単一 backend なので、`agent_cli.backend`
    ///   が設定されていれば値に関係なく `"claude"` を強制、未設定なら None
    ///   (`--provider` を渡さない / shim の default を使う)。これにより
    ///   `[agent_cli] backend = "ollama"` のような設定が claude-cli-shim の
    ///   registry に `provider: "ollama"` として誤記録される問題を防ぐ。
    /// - `agent_cli` engine (default): 受け取った `provider_arg` をそのまま返す
    ///   (後方互換)。
    pub(crate) fn filter_provider(&self, provider_arg: Option<String>) -> Option<String> {
        match self.type_.as_str() {
            "claude_cli_shim" => provider_arg.map(|_| "claude".to_string()),
            _ => provider_arg,
        }
    }

    /// Phase 115 — engine type に応じて `--model` 引数を調整する。
    ///
    /// - `claude_cli_shim` engine: agent_cli.model (`glm-5.1:cloud` など) は
    ///   claude が認識しないため None を返し、claude の login default model
    ///   を使わせる。明示的に claude モデルを使いたい場合は `[agent_cli] model`
    ///   で `claude-opus-4-7` 等を指定すれば claude が認識する値はそのまま
    ///   通る — ここで値検証はせず、claude モデル名以外は無視するためにのみ
    ///   None 返却を選ぶ簡素な実装とする (将来 model 値を見て分岐する余地は
    ///   残す)。
    /// - `agent_cli` engine (default): 受け取った `model_arg` をそのまま返す
    ///   (後方互換)。
    pub(crate) fn filter_model<'a>(&self, model_arg: Option<&'a str>) -> Option<&'a str> {
        match self.type_.as_str() {
            "claude_cli_shim" => match model_arg {
                Some(m) if m.starts_with("claude-") => Some(m),
                _ => None,
            },
            _ => model_arg,
        }
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

/// Phase 121 — engine-aware peer-row predicate.
///
/// `<engine> list` の stdout 1 行が「実 peer 行」かどうかを ID 列の prefix で
/// 判定する純関数。
/// - agent-cli engine: `agent-<ULID>` 形式
/// - claude_cli_shim engine: `shim-<UUID>` 形式
///
/// header 行（`ID NAME ...`）/ 区切り行 / 空行は false を返す。
pub(crate) fn is_engine_peer_id(id: &str) -> bool {
    id.starts_with("agent-") || id.starts_with("shim-")
}

/// Phase 121 — engine-aware log directory resolver.
///
/// `agent_id` の prefix から engine namespace を推定し、JSONL ログ dir を返す:
/// - `shim-...` → `~/.local/share/claude-cli-shim/logs/<id>/`
/// - その他 (`agent-...`) → `~/.local/share/agent-cli/logs/<id>/`（既定）
///
/// `dirs::home_dir()` 失敗時は `None`。dir の存在は確認しない（呼び元で対処）。
pub(crate) fn agent_log_dir(agent_id: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let dir = if agent_id.starts_with("shim-") {
        home.join(".local/share/claude-cli-shim/logs").join(agent_id)
    } else {
        home.join(".local/share/agent-cli/logs").join(agent_id)
    };
    Some(dir)
}

/// Phase 123 — engine-aware registry directory resolver.
///
/// 各 engine が peer メタデータを書き出す registry dir を返す。
/// - `claude_cli_shim`: `~/.local/share/claude-cli-shim/registry/`
/// - `agent_cli` (default): `$XDG_RUNTIME_DIR/agent-cli/`、未設定時は
///   `/tmp/agent-cli/`（`agent-cli/src/config.rs::registry_dir()` と同一規則）
///
/// `EngineConfig::registry_path` で明示 override されていればそれを優先。
/// `dirs::home_dir()` 失敗時 (`claude_cli_shim` 既定経路) は `None`。
pub(crate) fn engine_registry_dir(cfg: &HestiaConfig) -> Option<PathBuf> {
    if let Some(p) = cfg.engine.registry_path.as_ref() {
        if !p.as_os_str().is_empty() {
            return Some(p.clone());
        }
    }
    match cfg.engine.type_.as_str() {
        "claude_cli_shim" => {
            let home = dirs::home_dir()?;
            Some(home.join(".local/share/claude-cli-shim/registry"))
        }
        _ => {
            if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
                if !dir.is_empty() {
                    return Some(PathBuf::from(dir).join("agent-cli"));
                }
            }
            Some(PathBuf::from("/tmp/agent-cli"))
        }
    }
}

/// Phase 123 — engine-aware kill pattern enumerator.
///
/// `pgrep -f <pattern>` で引っかけるべき自エンジン peer プロセスのパターン列を返す。
/// `cfg.engine.binary_basename()` から動的生成するため、agent_cli / claude_cli_shim
/// 双方の peer プロセスが `hestia kill` で確実に SIGKILL されるようになる
/// (Phase 121 で `KILL_PATTERNS` の hardcode が claude_cli_shim 未対応だった漏れを fix)。
pub(crate) fn engine_kill_patterns(cfg: &HestiaConfig) -> Vec<String> {
    let bin = cfg.engine.binary_basename();
    vec![
        format!("{bin} run"),
        "hestia mirror".to_string(),
        "hestia monitor-daemon".to_string(),
    ]
}

/// Phase 123 — pid 生死判定。
///
/// `kill(pid, 0)` を libc で発行し、戻り値が 0 なら alive とみなす純粋判定。
/// `EPERM` (権限なし、戻り値 -1) は本実装では dead 扱いにせず alive とみなしたい
/// ところだが、registry の pid は通常自 uid のプロセスなので発生しない想定。
///
/// 防衛的に弾く値:
/// - `pid == 0` — libc 上 "全プロセスへ送信" の意味になるため。
/// - `pid > i32::MAX as u32` — `pid_t` は符号付き整数なので、cast すると
///   負値となり `kill(-N, 0)` が「プロセスグループ送信」として扱われ、
///   alive と誤判定されうる (`u32::MAX as i32 == -1` で全プロセス対象に)。
pub(crate) fn is_pid_alive(pid: u32) -> bool {
    if pid == 0 || pid > i32::MAX as u32 {
        return false;
    }
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Phase 123 — registry エントリの I/O フリー dead 判定 (純関数)。
///
/// `(path, pid)` のスライスから、`is_alive(pid) == false` の path を抽出する。
/// `is_alive` をクロージャ化することで filesystem / libc に依存せず unit-test
/// 可能にする (`prune_dead_peers` の中核ロジック)。
pub(crate) fn classify_registry_entries(
    entries: &[(PathBuf, u32)],
    is_alive: impl Fn(u32) -> bool,
) -> Vec<PathBuf> {
    entries
        .iter()
        .filter(|(_, pid)| !is_alive(*pid))
        .map(|(p, _)| p.clone())
        .collect()
}

/// Phase 123 — registry dir 配下の `*.json` を走査し、各エントリの `pid` field を
/// 抽出して `(path, pid)` のリストを返す純関数寄りヘルパ。
///
/// JSON parse / pid 取得失敗のエントリは silent skip (warning も出さない —
/// 呼び元で件数からズレを推定する)。`read_dir` 失敗時は空配列。
fn read_registry_pids(dir: &Path) -> Vec<(PathBuf, u32)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let Some(pid) = v.get("pid").and_then(|p| p.as_u64()) else {
            continue;
        };
        if pid > u32::MAX as u64 {
            continue;
        }
        out.push((path, pid as u32));
    }
    out
}

/// Phase 123 — `cfg.engine` に応じて registry から dead peer を除去する。
///
/// 戻り値は削除した entry 件数。失敗 (registry dir 不在 / 全 entry alive) は 0 を返す。
/// agent_cli engine では `*.json` に加え対応する `*.sock` も削除する
/// (`agent-cli/src/ipc/registry.rs::cleanup` と同じ後始末)。
pub(crate) fn prune_dead_peers(cfg: &HestiaConfig) -> usize {
    let Some(dir) = engine_registry_dir(cfg) else {
        return 0;
    };
    let pids = read_registry_pids(&dir);
    let dead = classify_registry_entries(&pids, is_pid_alive);
    let mut removed = 0;
    for path in dead {
        // <agent-id>.json を削除
        if std::fs::remove_file(&path).is_ok() {
            removed += 1;
            // agent_cli engine では同 stem の .sock も後始末する。
            if cfg.engine.type_ != "claude_cli_shim" {
                let sock = path.with_extension("sock");
                let _ = std::fs::remove_file(&sock);
            }
        } else {
            eprintln!(
                "[warn] failed to remove stale registry entry: {}",
                path.display()
            );
        }
    }
    removed
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

/// Phase 128 — `std::process::Command` と `tokio::process::Command` の双方を
/// 統一的に env 設定するためのアダプタ trait。
pub(crate) trait CmdEnvSetter {
    fn set_env(&mut self, key: &str, val: &str);
}

impl CmdEnvSetter for std::process::Command {
    fn set_env(&mut self, key: &str, val: &str) {
        self.env(key, val);
    }
}

impl CmdEnvSetter for tokio::process::Command {
    fn set_env(&mut self, key: &str, val: &str) {
        self.env(key, val);
    }
}

/// Phase 128 — `.hestia/config.toml` の `[concurrency]` を子プロセスの env に
/// export する。親プロセス env が既に同 key を持っていればそれを優先（後方互換）。
///
/// 優先順位: 親プロセス env > config.toml `[concurrency]` > library 既定。
pub(crate) fn apply_concurrency_env<C: CmdEnvSetter>(
    cmd: &mut C,
    cfg: &ConcurrencyConfig,
) {
    fn set_if_unset<C: CmdEnvSetter>(cmd: &mut C, key: &str, val: String) {
        if std::env::var_os(key).is_none() {
            cmd.set_env(key, &val);
        }
    }
    if let Some(v) = cfg.global_max {
        set_if_unset(cmd, "HESTIA_GLOBAL_MAX_AGENTS", v.to_string());
    }
    if let Some(v) = cfg.ai_conductor_dispatch_max {
        set_if_unset(cmd, "HESTIA_AI_DISPATCH_MAX", v.to_string());
    }
    if let Some(v) = cfg.per_conductor_max {
        set_if_unset(cmd, "HESTIA_PER_CONDUCTOR_MAX", v.to_string());
    }
    if let Some(v) = cfg.acquire_timeout_secs {
        set_if_unset(cmd, "HESTIA_ACQUIRE_TIMEOUT_SECS", v.to_string());
    }
}

/// Phase 127 — `--version` で表示する version 文字列。
///
/// `build.rs` が `git describe --tags --dirty` の結果を `HESTIA_BUILD_VERSION` に
/// 注入していればそれを使い (例: `0.1.5-3-gabc1234[-dirty]`)、無ければ
/// `CARGO_PKG_VERSION` (= `[workspace.package] version`) にフォールバックする。
/// これによりバイナリ表示が GitHub TAG と自動同期される。
pub const HESTIA_VERSION: &str = match option_env!("HESTIA_BUILD_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

/// Hestia -- unified runner for domain conductors and CLIs
#[derive(Parser)]
#[command(name = "hestia", version = HESTIA_VERSION, about)]
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
    /// Upgrade hestia itself by rebuilding from source and reinstalling
    /// to ~/.local/bin/hestia.
    ///
    /// Source resolution order:
    ///   1. --source <PATH>
    ///   2. $HESTIA_SOURCE_DIR
    ///   3. cwd if it has .hestia/tools/Cargo.toml
    ///   4. ~/hestia
    ///
    /// Then runs `git pull --ff-only` (unless --no-pull), `cargo build
    /// --release --bin hestia`, and copies the new binary in place.
    Upgrade {
        /// Hestia source repo path (overrides $HESTIA_SOURCE_DIR / cwd / ~/hestia).
        #[arg(long)]
        source: Option<PathBuf>,
        /// Skip `git pull --ff-only`.
        #[arg(long, default_value_t = false)]
        no_pull: bool,
        /// Print the steps without executing them.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Stream cargo / git output verbatim (default: summarized).
        #[arg(long, default_value_t = false)]
        verbose: bool,
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

# Phase 126 — サブエージェント並列度の上限。値を増やすほど LLM / PC 負荷が上がる。
# 各値は対応する環境変数で個別 override 可能。
[concurrency]
global_max = 8                       # ai-conductor が把握する全エージェント合計 (HESTIA_GLOBAL_MAX_AGENTS)
ai_conductor_dispatch_max = 2        # ai-conductor が同時 dispatch する domain conductor 数 (HESTIA_AI_DISPATCH_MAX)
per_conductor_max = 4                # 各 conductor が同時 spawn できるサブエージェント数 (HESTIA_PER_CONDUCTOR_MAX)
acquire_timeout_secs = 600           # slot 待機タイムアウト秒（デッドロック検知）(HESTIA_ACQUIRE_TIMEOUT_SECS)
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

/// Phase 109 — `<engine> list` の stdout から既登録 peer 名集合を抽出する純関数。
/// NAME 列（2 列目）を完全一致で集める。
/// Phase 121: ID prefix 判定を `is_engine_peer_id` で engine 抽象化
/// (agent-cli の `agent-`、claude_cli_shim の `shim-` 双方を許容)。
pub(crate) fn registered_peer_names(stdout: &str) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    for line in stdout.lines().skip(1) {
        let mut it = line.split_whitespace();
        let Some(id) = it.next() else { continue };
        let Some(name) = it.next() else { continue };
        if is_engine_peer_id(id) {
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
    // Phase 115 — claude_cli_shim engine 時は --provider を claude に強制し、
    // 非 claude モデル名 (例: glm-5.1:cloud) は claude が認識しないため抑制する。
    let provider = config
        .engine
        .filter_provider(config.agent_cli.provider_arg());
    let model = config
        .engine
        .filter_model(config.agent_cli.model.as_deref());
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
    // Phase 128 — config.toml の [concurrency] を子プロセス env に export
    apply_concurrency_env(&mut cmd, &config.concurrency);
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
        // Phase 115 — claude_cli_shim engine 時は --provider を claude に強制し、
        // 非 claude モデル名は抑制する。
        let provider = config
            .engine
            .filter_provider(config.agent_cli.provider_arg());
        let model = config
            .engine
            .filter_model(config.agent_cli.model.as_deref());
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
        // Phase 128 — config.toml の [concurrency] を子プロセス env に export
        apply_concurrency_env(&mut cmd, &config.concurrency);
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

/// (Phase 123 廃止) 旧 hardcode 配列 — `engine_kill_patterns(&cfg)` で
/// engine 抽象化されたため未使用。テスト保護のため定数自体は残す。
#[allow(dead_code)]
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
/// 1. Run `pgrep -f <pattern>` for each entry returned by
///    [`engine_kill_patterns`] (Phase 123: engine 抽象化、agent-cli /
///    claude-cli-shim 双方を必ずカバー) and collect stdout. A pgrep failure
///    is logged as a warning but doesn't abort the overall command.
/// 2. Compute the kill list with [`select_kill_targets`]: own PID excluded,
///    duplicates removed across patterns.
/// 3. Send SIGKILL to each target via `libc::kill` and tally success / failure.
///    Per-PID failures (already exited, permission denied) are logged and
///    skipped — they don't fail the whole command.
/// 4. (Phase 123) 300ms grace 後に [`prune_dead_peers`] で registry から
///    dead エントリを掃除する。`hestia kill` で peer を SIGKILL したあとも
///    `<engine> list` (= `hestia monitor` の表示根拠) に行が残り続ける
///    不具合を解消するため。
async fn kill_all_processes(cfg: &HestiaConfig) -> Result<()> {
    let patterns = engine_kill_patterns(cfg);
    let mut outputs: Vec<(String, String)> = Vec::with_capacity(patterns.len());
    for pattern in &patterns {
        let result = Command::new("pgrep")
            .arg("-f")
            .arg(pattern)
            .output()
            .await;
        match result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
                outputs.push((pattern.clone(), stdout));
            }
            Err(e) => {
                eprintln!("[warn] pgrep failed for pattern '{pattern}': {e}");
                outputs.push((pattern.clone(), String::new()));
            }
        }
    }

    let outputs_ref: Vec<(&str, &str)> =
        outputs.iter().map(|(p, s)| (p.as_str(), s.as_str())).collect();
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
        // SIGKILL は非同期に届くため、registry 掃除前に短い grace を入れる
        // (NFR-5: race condition 緩和)。
        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    // Phase 123: 死 peer の registry エントリを削除し、`hestia monitor` 表示
    // から消えるようにする。targets が 0 件でも前回 run の残骸を掃除できる
    // ように常に呼ぶ。
    let pruned = prune_dead_peers(cfg);
    if pruned > 0 {
        println!("Pruned {pruned} dead peer(s) from registry.");
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
    // Phase 121: agent_id prefix から engine 別 log dir を解決
    // (agent-cli は `~/.local/share/agent-cli/logs/<id>/`、
    //  claude_cli_shim は `~/.local/share/claude-cli-shim/logs/<id>/session.jsonl`)。
    let Some(log_dir) = agent_log_dir(agent_id) else {
        return AgentStatus::Unknown;
    };
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
        // Phase 121: prefix 判定を engine 抽象化 (agent-cli + claude_cli_shim 両対応)
        if is_engine_peer_id(id) && !out.contains_key(id) {
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
            "no running agent named '{domain}' found via '{engine_bin} list'. Did you run 'hestia start'?"
        )
    })?;

    // Phase 121: agent_id prefix から engine 別 log dir を解決。
    let log_dir = agent_log_dir(&agent_id)
        .ok_or_else(|| anyhow::anyhow!("could not resolve $HOME"))?;
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
                        // Phase 121: agent-cli は `name`、claude-cli-shim は `tool` field を使う
                        let name = ev
                            .get("name")
                            .or_else(|| ev.get("tool"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        let args_text = ev.get("args").map(|v| v.to_string()).unwrap_or_default();
                        let args_short: String = args_text.chars().take(160).collect();
                        let _ = writeln!(out, "[mirror][tool_call] {} args={}", name, args_short);
                    }
                    "tool_result" => {
                        let name = ev
                            .get("name")
                            .or_else(|| ev.get("tool"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        let ok = ev.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                        let _ = writeln!(out, "[mirror][tool_result] {} ok={}", name, ok);
                    }
                    "peer_prompt" => {
                        let from = ev.get("from").and_then(|v| v.as_str()).unwrap_or("?");
                        let _ = writeln!(out, "[mirror][peer_prompt] from={}", from);
                    }
                    // Phase 121: claude-cli-shim は peer_prompt の代わりに `user` kind を出す
                    "user" => {
                        let snippet: String = ev
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .chars()
                            .take(160)
                            .collect();
                        let _ = writeln!(out, "[mirror][user] {}", snippet.trim());
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

// ─── Phase 124: hestia upgrade ────────────────────────────────────────────

/// Phase 124 — Source repo 解決優先順 (純関数)。
///
/// I/O フリーにするため `path_has_workspace` クロージャで existence check を
/// 注入する。実環境では `|p| p.join(".hestia/tools/Cargo.toml").is_file()`。
///
/// 解決順:
///   1. `--source <PATH>` (validation: workspace check も走る)
///   2. `$HESTIA_SOURCE_DIR`
///   3. `cwd`
///   4. `~/hestia`
///
/// すべて該当なしなら hint 付きエラー文字列を返す。
pub(crate) fn resolve_source_path(
    arg_source: Option<&Path>,
    env_source: Option<&str>,
    cwd: &Path,
    home: Option<&Path>,
    path_has_workspace: impl Fn(&Path) -> bool,
) -> std::result::Result<PathBuf, String> {
    if let Some(p) = arg_source {
        return if path_has_workspace(p) {
            Ok(p.to_path_buf())
        } else {
            Err(format!(
                "--source path is not a hestia repo: {}",
                p.display()
            ))
        };
    }
    if let Some(s) = env_source {
        if !s.is_empty() {
            let p = Path::new(s);
            if path_has_workspace(p) {
                return Ok(p.to_path_buf());
            }
        }
    }
    if path_has_workspace(cwd) {
        return Ok(cwd.to_path_buf());
    }
    if let Some(h) = home {
        let candidate = h.join("hestia");
        if path_has_workspace(&candidate) {
            return Ok(candidate);
        }
    }
    Err("hestia source repo not found.\n  Tried: --source / $HESTIA_SOURCE_DIR / cwd / ~/hestia\n  Hint: clone the repo to ~/hestia or pass --source <path>.".to_string())
}

/// Phase 124 — install 完了サマリの整形 (純関数)。
pub(crate) fn format_install_summary(
    source: &Path,
    built: &Path,
    install: &Path,
    version: &str,
) -> String {
    format!(
        "hestia upgraded successfully.\n  Source:    {}\n  Built:     {}\n  Installed: {}\n  Version:   {}\n",
        source.display(),
        built.display(),
        install.display(),
        version.trim(),
    )
}

/// Phase 124 — `git -C <source> pull --ff-only` を実行する。
/// 失敗時は warning ログを出して継続 (NFR: ローカル変更があってもユーザを困らせない)。
async fn run_git_pull(source: &Path, verbose: bool) -> Result<()> {
    let stdout_cfg = if verbose { Stdio::inherit() } else { Stdio::piped() };
    let stderr_cfg = if verbose { Stdio::inherit() } else { Stdio::piped() };
    println!("$ git -C {} pull --ff-only", source.display());
    let output = Command::new("git")
        .arg("-C")
        .arg(source)
        .arg("pull")
        .arg("--ff-only")
        .stdout(stdout_cfg)
        .stderr(stderr_cfg)
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to spawn git: {e}"))?;
    if !output.status.success() {
        eprintln!("[warn] git pull failed (continuing with current checkout)");
        if !verbose {
            let stderr = String::from_utf8_lossy(&output.stderr);
            for line in stderr.lines().take(5) {
                eprintln!("[warn]   {line}");
            }
        }
    }
    Ok(())
}

/// Phase 124 — `cargo build --release --bin hestia` を `<source>/.hestia/tools` で実行。
/// 失敗時は末尾 stderr を表示して bail。
async fn run_cargo_build(source: &Path, verbose: bool) -> Result<()> {
    let workspace = source.join(".hestia/tools");
    let stdout_cfg = if verbose { Stdio::inherit() } else { Stdio::piped() };
    let stderr_cfg = if verbose { Stdio::inherit() } else { Stdio::piped() };
    println!("Rebuilding hestia from {} ...", source.display());
    let start = std::time::Instant::now();
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--bin")
        .arg("hestia")
        .current_dir(&workspace)
        .stdout(stdout_cfg)
        .stderr(stderr_cfg)
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to spawn cargo: {e}"))?;
    if !output.status.success() {
        if !verbose {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let lines: Vec<&str> = stderr.lines().collect();
            let tail = if lines.len() > 20 { &lines[lines.len() - 20..] } else { &lines[..] };
            for line in tail {
                eprintln!("{line}");
            }
        }
        bail!("cargo build failed");
    }
    let elapsed = start.elapsed().as_secs_f32();
    println!("Built in {elapsed:.2} s");
    Ok(())
}

/// Phase 124 — built バイナリを install path に置換する。
///
/// Phase 123 検証で「Text file busy」(現実行中バイナリの上書きエラー) が発生
/// したため、まず `remove_file` → `copy` → `set_permissions` の順で確実に置換する。
fn install_binary(built: &Path, install: &Path) -> Result<()> {
    if !built.is_file() {
        bail!("built binary not found at {}", built.display());
    }
    if let Some(parent) = install.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let _ = std::fs::remove_file(install); // 不在 / 失敗は ok 扱い
    std::fs::copy(built, install).map_err(|e| {
        anyhow::anyhow!(
            "failed to copy {} -> {}: {e}",
            built.display(),
            install.display()
        )
    })?;
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(install) {
        let mut perm = meta.permissions();
        perm.set_mode(0o755);
        let _ = std::fs::set_permissions(install, perm);
    }
    Ok(())
}

/// Phase 124 — install 後の binary から `--version` を取得する。
async fn fetch_installed_version(install: &Path) -> Result<String> {
    let out = Command::new(install)
        .arg("--version")
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to run installed hestia --version: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Phase 124 — `--dry-run` で各ステップを表示するのみ (実コマンドは投入しない)。
fn print_dry_run(source: &Path, built: &Path, install: &Path, no_pull: bool) {
    if !no_pull {
        println!("$ git -C {} pull --ff-only", source.display());
    } else {
        println!("(skip git pull)");
    }
    let workspace = source.join(".hestia/tools");
    println!(
        "$ (cd {} && cargo build --release --bin hestia)",
        workspace.display()
    );
    println!("$ rm -f {}", install.display());
    println!("$ cp {} {}", built.display(), install.display());
    println!("(dry-run: no commands were executed)");
}

/// Phase 124 — `hestia upgrade` の main handler。
async fn run_upgrade(
    source: Option<PathBuf>,
    no_pull: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let env_source = std::env::var("HESTIA_SOURCE_DIR").ok();
    let home = dirs::home_dir();
    let resolved = resolve_source_path(
        source.as_deref(),
        env_source.as_deref(),
        &cwd,
        home.as_deref(),
        |p| p.join(".hestia/tools/Cargo.toml").is_file(),
    )
    .map_err(anyhow::Error::msg)?;
    let built = resolved.join(".hestia/tools/target/release/hestia");
    let install = home
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("$HOME unresolved"))?
        .join(".local/bin/hestia");

    if dry_run {
        print_dry_run(&resolved, &built, &install, no_pull);
        return Ok(());
    }

    if !no_pull {
        run_git_pull(&resolved, verbose).await?;
    }
    run_cargo_build(&resolved, verbose).await?;
    install_binary(&built, &install)?;
    let version = fetch_installed_version(&install).await?;
    print!(
        "{}",
        format_install_summary(&resolved, &built, &install, &version)
    );
    Ok(())
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

    // Phase 115 — 親プロセス自身の env にも HESTIA_ENGINE_BINARY を export し、
    // conductor-sdk の wait_for_registry / agent_cli_peer_alive / agent_cli_send 等が
    // 正しい engine を呼ぶようにする (subprocess_env() は子プロセスにのみ env を渡すため、
    // 親プロセス内から conductor-sdk を呼んだ場合の fallback "agent-cli" を防ぐ目的)。
    // hestia ai run 経由で spawn される hestia-ai-cli にも env が伝搬する。
    let cfg = load_hestia_config();
    std::env::set_var("HESTIA_ENGINE_BINARY", cfg.engine.binary_name());

    match cli.command {
        Commands::Init => init_hestia_dir()?,
        Commands::Start { domain } => {
            // Phase 123: 起動前に死 peer registry を掃除し、`hestia monitor` の
            // 表示が前回 run の残骸を引きずらないようにする。
            let pruned = prune_dead_peers(&cfg);
            if pruned > 0 {
                println!("Pruned {pruned} dead peer(s) before start.");
            }
            match domain {
                Some(d) => start_conductor(&d).await?,
                None => start_all_conductors().await?,
            }
        }
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
        Commands::Kill => kill_all_processes(&cfg).await?,
        Commands::Monitor { interval, once, all } => {
            monitor::run_monitor(interval, once, all).await?
        }
        Commands::MonitorDaemon => monitor::run_monitor_daemon().await?,
        Commands::Upgrade { source, no_pull, dry_run, verbose } => {
            run_upgrade(source, no_pull, dry_run, verbose).await?
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        agent_log_dir, apply_concurrency_env, classify_registry_entries, derive_status_from_log,
        engine_kill_patterns, format_install_summary, is_engine_peer_id, is_pid_alive,
        parse_status_events, registered_peer_names, resolve_source_path, select_kill_targets,
        transform_status_listing, AgentStatus, ConcurrencyConfig, EngineConfig, HestiaConfig,
        StatusEvent,
    };
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
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

    // ─── Phase 121: engine 抽象化ヘルパ ────────────────────────────────

    #[test]
    fn is_engine_peer_id_accepts_agent_cli_prefix() {
        assert!(is_engine_peer_id("agent-AAA"));
        assert!(is_engine_peer_id(
            "agent-01KQX72WJY3Z59RN77YXB9Z02P"
        ));
    }

    #[test]
    fn is_engine_peer_id_accepts_claude_cli_shim_prefix() {
        assert!(is_engine_peer_id("shim-aaaa-1111"));
        assert!(is_engine_peer_id(
            "shim-e72f954c-18c5-4a83-9b5e-927d721589c2"
        ));
    }

    #[test]
    fn is_engine_peer_id_rejects_header_and_garbage() {
        assert!(!is_engine_peer_id("ID"));
        assert!(!is_engine_peer_id(""));
        assert!(!is_engine_peer_id("0001"));
        assert!(!is_engine_peer_id("---"));
        assert!(!is_engine_peer_id("ai"));
    }

    #[test]
    fn agent_log_dir_routes_by_prefix() {
        let agent_dir = agent_log_dir("agent-AAA").expect("$HOME present");
        assert!(agent_dir
            .to_string_lossy()
            .contains(".local/share/agent-cli/logs/agent-AAA"));

        let shim_dir = agent_log_dir("shim-bbbb-2222").expect("$HOME present");
        assert!(shim_dir
            .to_string_lossy()
            .contains(".local/share/claude-cli-shim/logs/shim-bbbb-2222"));
    }

    #[test]
    fn registered_peer_names_picks_claude_cli_shim_rows_phase121() {
        let input = "\
ID                                         NAME         PROVIDER  MODEL            ROLE
shim-aaaa-1111                             ai           claude    claude-opus-4-7  meta
shim-bbbb-2222                             ai-designer  claude    claude-opus-4-7  designer
";
        let got = registered_peer_names(input);
        assert_eq!(got.len(), 2);
        assert!(got.contains("ai"));
        assert!(got.contains("ai-designer"));
    }

    // ─── Phase 123: hestia kill 残留 / hestia start cleanup ─────────────

    fn cfg_with_engine(t: &str) -> HestiaConfig {
        let mut cfg = HestiaConfig::default();
        cfg.engine = EngineConfig {
            type_: t.to_string(),
            binary: None,
            registry_path: None,
            log_path: None,
        };
        cfg
    }

    #[test]
    fn is_pid_alive_returns_true_for_self_pid() {
        let me = std::process::id();
        assert!(is_pid_alive(me));
    }

    #[test]
    fn is_pid_alive_returns_false_for_zero() {
        // pid=0 は libc 上「全プロセスへ送信」になるため defensive に false。
        assert!(!is_pid_alive(0));
    }

    #[test]
    fn is_pid_alive_returns_false_for_unlikely_max_pid() {
        // u32::MAX は実存しない pid 想定。万が一 alive でもテスト環境依存の
        // 偶然なので skip 扱いで問題ない (CI で再現性を確保できる程度には
        // u32::MAX が割り当てられないことを前提とする一般的な assumption)。
        assert!(!is_pid_alive(u32::MAX));
    }

    #[test]
    fn engine_kill_patterns_for_agent_cli() {
        let cfg = cfg_with_engine("agent_cli");
        let p = engine_kill_patterns(&cfg);
        assert_eq!(p.len(), 3);
        assert_eq!(p[0], "agent-cli run");
        assert_eq!(p[1], "hestia mirror");
        assert_eq!(p[2], "hestia monitor-daemon");
    }

    #[test]
    fn engine_kill_patterns_for_claude_cli_shim() {
        let cfg = cfg_with_engine("claude_cli_shim");
        let p = engine_kill_patterns(&cfg);
        assert_eq!(p.len(), 3);
        assert_eq!(p[0], "claude-cli-shim run");
        assert_eq!(p[1], "hestia mirror");
        assert_eq!(p[2], "hestia monitor-daemon");
    }

    #[test]
    fn classify_registry_entries_filters_dead_only() {
        let entries = vec![
            (PathBuf::from("/tmp/a.json"), 100u32), // even → alive
            (PathBuf::from("/tmp/b.json"), 200u32), // even → alive
            (PathBuf::from("/tmp/c.json"), 301u32), // odd  → dead
            (PathBuf::from("/tmp/d.json"), 400u32), // even → alive
        ];
        // 偶数 pid を alive とみなすクロージャ → 301 のみ dead 扱い。
        let dead = classify_registry_entries(&entries, |pid| pid % 2 == 0);
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0], PathBuf::from("/tmp/c.json"));
    }

    #[test]
    fn classify_registry_entries_empty_input_yields_empty() {
        let entries: Vec<(PathBuf, u32)> = Vec::new();
        let dead = classify_registry_entries(&entries, |_| true);
        assert!(dead.is_empty());
    }

    #[test]
    fn classify_registry_entries_all_alive_yields_empty() {
        let entries = vec![
            (PathBuf::from("/tmp/a.json"), 1u32),
            (PathBuf::from("/tmp/b.json"), 2u32),
        ];
        let dead = classify_registry_entries(&entries, |_| true);
        assert!(dead.is_empty());
    }

    #[test]
    fn classify_registry_entries_all_dead_returns_all_paths() {
        let entries = vec![
            (PathBuf::from("/tmp/a.json"), 10u32),
            (PathBuf::from("/tmp/b.json"), 20u32),
        ];
        let dead = classify_registry_entries(&entries, |_| false);
        assert_eq!(dead.len(), 2);
        assert!(dead.contains(&PathBuf::from("/tmp/a.json")));
        assert!(dead.contains(&PathBuf::from("/tmp/b.json")));
    }

    // ─── Phase 124: hestia upgrade ──────────────────────────────────────

    /// 「該当 path を hestia repo とみなす」mock 用クロージャを生成する。
    fn workspace_at(allowed: &[&str]) -> impl Fn(&Path) -> bool {
        let owned: Vec<PathBuf> = allowed.iter().map(PathBuf::from).collect();
        move |p: &Path| owned.iter().any(|a| a.as_path() == p)
    }

    #[test]
    fn resolve_source_path_prefers_arg() {
        let arg = PathBuf::from("/some/repo");
        let cwd = PathBuf::from("/tmp/elsewhere");
        let home = PathBuf::from("/home/user");
        let got = resolve_source_path(
            Some(&arg),
            Some("/env/path"),
            &cwd,
            Some(&home),
            workspace_at(&["/some/repo", "/env/path", "/home/user/hestia"]),
        );
        assert_eq!(got.unwrap(), arg);
    }

    #[test]
    fn resolve_source_path_falls_back_to_env() {
        let cwd = PathBuf::from("/tmp/elsewhere");
        let home = PathBuf::from("/home/user");
        let got = resolve_source_path(
            None,
            Some("/env/path"),
            &cwd,
            Some(&home),
            workspace_at(&["/env/path", "/home/user/hestia"]),
        );
        assert_eq!(got.unwrap(), PathBuf::from("/env/path"));
    }

    #[test]
    fn resolve_source_path_uses_cwd_if_workspace() {
        let cwd = PathBuf::from("/work/hestia-checkout");
        let home = PathBuf::from("/home/user");
        let got = resolve_source_path(
            None,
            None,
            &cwd,
            Some(&home),
            workspace_at(&["/work/hestia-checkout", "/home/user/hestia"]),
        );
        assert_eq!(got.unwrap(), cwd);
    }

    #[test]
    fn resolve_source_path_uses_home_hestia_default() {
        let cwd = PathBuf::from("/tmp/elsewhere");
        let home = PathBuf::from("/home/user");
        let got = resolve_source_path(
            None,
            None,
            &cwd,
            Some(&home),
            workspace_at(&["/home/user/hestia"]),
        );
        assert_eq!(got.unwrap(), PathBuf::from("/home/user/hestia"));
    }

    #[test]
    fn resolve_source_path_returns_error_when_none() {
        let cwd = PathBuf::from("/tmp/elsewhere");
        let home = PathBuf::from("/home/user");
        let err = resolve_source_path(None, None, &cwd, Some(&home), workspace_at(&[]))
            .unwrap_err();
        assert!(err.contains("hestia source repo not found"));
        assert!(err.contains("--source"));
        assert!(err.contains("$HESTIA_SOURCE_DIR"));
    }

    #[test]
    fn resolve_source_path_arg_validates_existence() {
        let arg = PathBuf::from("/tmp/nonexistent");
        let cwd = PathBuf::from("/tmp");
        let err = resolve_source_path(
            Some(&arg),
            None,
            &cwd,
            None,
            workspace_at(&[]),
        )
        .unwrap_err();
        assert!(err.contains("--source path is not a hestia repo"));
        assert!(err.contains("/tmp/nonexistent"));
    }

    #[test]
    fn format_install_summary_contains_all_paths_and_version() {
        let summary = format_install_summary(
            Path::new("/home/u/hestia"),
            Path::new("/home/u/hestia/.hestia/tools/target/release/hestia"),
            Path::new("/home/u/.local/bin/hestia"),
            "hestia 0.1.0\n",
        );
        assert!(summary.contains("hestia upgraded successfully."));
        assert!(summary.contains("Source:    /home/u/hestia"));
        assert!(summary.contains(
            "Built:     /home/u/hestia/.hestia/tools/target/release/hestia"
        ));
        assert!(summary.contains("Installed: /home/u/.local/bin/hestia"));
        assert!(summary.contains("Version:   hestia 0.1.0"));
        // 末尾改行なし version でも改行で 5 行構成 (見出し + 4 詳細)。
        assert_eq!(summary.lines().count(), 5);
    }

    // ─── Phase 128: [concurrency] config.toml 連携 ────────────────────────

    #[test]
    fn concurrency_config_parses_from_toml() {
        let toml = r#"
[agent_cli]
backend = "ollama"

[concurrency]
global_max = 1
per_conductor_max = 2
acquire_timeout_secs = 60
"#;
        let cfg: HestiaConfig = toml::from_str(toml).expect("parse ok");
        assert_eq!(cfg.concurrency.global_max, Some(1));
        assert_eq!(cfg.concurrency.per_conductor_max, Some(2));
        assert_eq!(cfg.concurrency.ai_conductor_dispatch_max, None);
        assert_eq!(cfg.concurrency.acquire_timeout_secs, Some(60));
    }

    #[test]
    fn concurrency_config_absent_yields_all_none() {
        let toml = r#"[agent_cli]
backend = "ollama"
"#;
        let cfg: HestiaConfig = toml::from_str(toml).expect("parse ok");
        assert!(cfg.concurrency.global_max.is_none());
        assert!(cfg.concurrency.ai_conductor_dispatch_max.is_none());
        assert!(cfg.concurrency.per_conductor_max.is_none());
        assert!(cfg.concurrency.acquire_timeout_secs.is_none());
    }

    #[test]
    fn apply_concurrency_env_sets_envs_on_command() {
        // 親 env を一旦すべて削除（テスト分離。serial 実行が前提）。
        std::env::remove_var("HESTIA_GLOBAL_MAX_AGENTS");
        std::env::remove_var("HESTIA_AI_DISPATCH_MAX");
        std::env::remove_var("HESTIA_PER_CONDUCTOR_MAX");
        std::env::remove_var("HESTIA_ACQUIRE_TIMEOUT_SECS");

        let cfg = ConcurrencyConfig {
            global_max: Some(3),
            ai_conductor_dispatch_max: Some(2),
            per_conductor_max: Some(1),
            acquire_timeout_secs: Some(30),
        };
        let mut cmd = std::process::Command::new("true");
        apply_concurrency_env(&mut cmd, &cfg);

        let envs: HashMap<_, _> = cmd
            .get_envs()
            .filter_map(|(k, v)| {
                v.map(|val| (k.to_string_lossy().into_owned(), val.to_string_lossy().into_owned()))
            })
            .collect();
        assert_eq!(envs.get("HESTIA_GLOBAL_MAX_AGENTS").map(String::as_str), Some("3"));
        assert_eq!(envs.get("HESTIA_AI_DISPATCH_MAX").map(String::as_str), Some("2"));
        assert_eq!(envs.get("HESTIA_PER_CONDUCTOR_MAX").map(String::as_str), Some("1"));
        assert_eq!(envs.get("HESTIA_ACQUIRE_TIMEOUT_SECS").map(String::as_str), Some("30"));
    }

    #[test]
    fn apply_concurrency_env_does_not_override_parent_env() {
        std::env::set_var("HESTIA_PER_CONDUCTOR_MAX", "8");
        std::env::remove_var("HESTIA_GLOBAL_MAX_AGENTS");

        let cfg = ConcurrencyConfig {
            global_max: Some(3),
            ai_conductor_dispatch_max: None,
            per_conductor_max: Some(1),  // 親 env=8 が優先される (set されない)
            acquire_timeout_secs: None,
        };
        let mut cmd = std::process::Command::new("true");
        apply_concurrency_env(&mut cmd, &cfg);

        let envs: HashMap<_, _> = cmd
            .get_envs()
            .filter_map(|(k, v)| {
                v.map(|val| (k.to_string_lossy().into_owned(), val.to_string_lossy().into_owned()))
            })
            .collect();
        // 親 env がある HESTIA_PER_CONDUCTOR_MAX は Command に追加されない (継承される)
        assert!(envs.get("HESTIA_PER_CONDUCTOR_MAX").is_none());
        // 親 env が無い HESTIA_GLOBAL_MAX_AGENTS は config.toml 値が適用される
        assert_eq!(envs.get("HESTIA_GLOBAL_MAX_AGENTS").map(String::as_str), Some("3"));

        std::env::remove_var("HESTIA_PER_CONDUCTOR_MAX");
    }

    #[test]
    fn apply_concurrency_env_skips_none_fields() {
        std::env::remove_var("HESTIA_GLOBAL_MAX_AGENTS");
        std::env::remove_var("HESTIA_AI_DISPATCH_MAX");
        std::env::remove_var("HESTIA_PER_CONDUCTOR_MAX");
        std::env::remove_var("HESTIA_ACQUIRE_TIMEOUT_SECS");

        let cfg = ConcurrencyConfig::default(); // 全 None
        let mut cmd = std::process::Command::new("true");
        apply_concurrency_env(&mut cmd, &cfg);

        let envs: HashMap<_, _> = cmd
            .get_envs()
            .filter_map(|(k, v)| {
                v.map(|val| (k.to_string_lossy().into_owned(), val.to_string_lossy().into_owned()))
            })
            .collect();
        assert!(envs.get("HESTIA_GLOBAL_MAX_AGENTS").is_none());
        assert!(envs.get("HESTIA_AI_DISPATCH_MAX").is_none());
        assert!(envs.get("HESTIA_PER_CONDUCTOR_MAX").is_none());
        assert!(envs.get("HESTIA_ACQUIRE_TIMEOUT_SECS").is_none());
    }
}