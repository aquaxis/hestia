//! Phase 108 — hestia システム稼働監視 + 稼働状況モニター。
//!
//! - `run_monitor_daemon()`: `hestia start ai` から子プロセスとして spawn される
//!   常駐ループ。30 秒周期で配下サブエージェント (ai-designer / ai-reviewer)
//!   と起動中 domain conductor の稼働状況を取得し、全停止 + タスク残存を検知
//!   した時のみ `agent-cli send` で再開指示を発行する。
//! - `run_monitor()`: 人間ユーザー向けの `hestia monitor` サブコマンド本体。
//!   既存 `show_status` 出力を定期更新表示する。
//!
//! 設計詳細は `.aiprj/AI_PRJ_DESIGN.md` §3 / §4 を参照。

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::process::Command;

use super::{collect_agent_statuses, show_status, transform_status_listing, AgentStatus};

/// 監視対象種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MonitorKind {
    /// ai-designer / ai-reviewer などの常駐サブエージェント。
    Subagent,
    /// rtl / fpga / asic / pcb / hal / apps / debug / rag のうち起動中のもの。
    DomainConductor,
}

/// 監視対象 1 件の解決結果。
#[derive(Debug, Clone)]
pub(crate) struct MonitorTarget {
    pub agent_id: String,
    pub peer: String,
    /// 種別（読込専用、ログ整形と将来の拡張で参照する）。
    #[allow(dead_code)]
    pub kind: MonitorKind,
    /// Phase 109 — 動的サブエージェント (`<domain>-coder-*` 等) の親 domain conductor 名。
    /// `Subagent` でも `parent_conductor = Some("ai")` (常駐 sub-agent) または
    /// `Some("rtl")` (動的 sub-agent) のように区別される。
    /// `DomainConductor` の場合は `None`。
    pub parent_conductor: Option<String>,
}

/// `task_status.md` の 1 行分。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskStatusEntry {
    pub task_id: String,
    pub state: String,
}

impl TaskStatusEntry {
    pub fn is_pending(&self) -> bool {
        matches!(self.state.as_str(), "未着手" | "進行中" | "ブロック")
    }
}

/// 監視対象の peer 名。`ai` 自身は除外（自己監視は行わない）。
const SUBAGENT_PEERS: &[&str] = &["ai-designer", "ai-reviewer"];
const DOMAIN_CONDUCTOR_PEERS: &[&str] =
    &["rtl", "fpga", "asic", "pcb", "hal", "apps", "debug", "rag"];

/// 監視周期（秒）。`HESTIA_MONITOR_INTERVAL_SECS` で上書き、5..=600 にクランプ。
pub(crate) fn monitor_interval_secs() -> u64 {
    std::env::var("HESTIA_MONITOR_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(clamp_monitor_interval)
        .unwrap_or(30)
}

/// 再開指示後の cooldown（秒）。`HESTIA_MONITOR_COOLDOWN_SECS` で上書き、0..=600 にクランプ。
pub(crate) fn monitor_cooldown_secs() -> u64 {
    std::env::var("HESTIA_MONITOR_COOLDOWN_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(clamp_monitor_cooldown)
        .unwrap_or(60)
}

/// 監視ループを完全に無効化する場合は `HESTIA_MONITOR_DISABLED=1` を設定する。
pub(crate) fn monitor_disabled() -> bool {
    matches!(
        std::env::var("HESTIA_MONITOR_DISABLED").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

pub(crate) fn clamp_monitor_interval(s: u64) -> u64 {
    s.clamp(5, 600)
}

pub(crate) fn clamp_monitor_cooldown(s: u64) -> u64 {
    s.clamp(0, 600)
}

/// `hestia monitor` の更新間隔（秒）をクランプ。1..=60。
pub(crate) fn clamp_view_interval(s: u64) -> u64 {
    s.clamp(1, 60)
}

/// プロジェクトルート直下の `.hestia/workspaces/` を返す。
pub(crate) fn workspaces_root() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".hestia/workspaces")
}

/// Phase 109 — peer 名から監視種別と親 conductor を分類する純関数。
///
/// 戻り値: `Some((kind, parent_conductor))` または `None`（監視対象外）。
/// - `ai-designer` / `ai-reviewer` → `(Subagent, Some("ai"))`
/// - `rtl` / `fpga` / ... → `(DomainConductor, None)`
/// - `rtl-coder-uart` / `asic-signoff` 等の `<domain>-*` 形式 → `(Subagent, Some(<domain>))`
/// - `ai`（自身）/ unknown → `None`
pub(crate) fn classify_peer(name: &str) -> Option<(MonitorKind, Option<String>)> {
    if name == "ai" {
        return None;
    }
    if SUBAGENT_PEERS.contains(&name) {
        return Some((MonitorKind::Subagent, Some("ai".to_string())));
    }
    if DOMAIN_CONDUCTOR_PEERS.contains(&name) {
        return Some((MonitorKind::DomainConductor, None));
    }
    for &d in DOMAIN_CONDUCTOR_PEERS {
        let prefix = format!("{d}-");
        if name.starts_with(&prefix) {
            return Some((MonitorKind::Subagent, Some(d.to_string())));
        }
    }
    None
}

/// `agent-cli list` の stdout から監視対象を抽出する純関数。
///
/// `classify_peer()` で監視種別を判定し、対象外なら除外する。Phase 109 で動的
/// サブエージェント（`<domain>-coder-*` 等）にも対応。
pub(crate) fn resolve_monitor_targets(stdout: &str) -> Vec<MonitorTarget> {
    let mut out = Vec::new();
    for line in stdout.lines().skip(1) {
        // 1 列目 = ID、2 列目 = NAME。空白区切りで先頭 2 トークンを取る。
        let mut it = line.split_whitespace();
        let Some(id) = it.next() else { continue };
        let Some(name) = it.next() else { continue };
        if !id.starts_with("agent-") {
            continue;
        }
        let Some((kind, parent_conductor)) = classify_peer(name) else {
            continue;
        };
        out.push(MonitorTarget {
            agent_id: id.to_string(),
            peer: name.to_string(),
            kind,
            parent_conductor,
        });
    }
    out
}

/// 全停止判定（純関数）。targets が空 = 監視対象不在 = `false`（停止検知扱いとしない）。
pub(crate) fn is_all_stopped(
    targets: &[MonitorTarget],
    statuses: &HashMap<String, AgentStatus>,
) -> bool {
    if targets.is_empty() {
        return false;
    }
    targets.iter().all(|t| match statuses.get(&t.agent_id) {
        // プロセス不在 (None) = 停止扱い。
        None => true,
        Some(AgentStatus::Idle) => true,
        Some(AgentStatus::Error) => true,
        Some(AgentStatus::Unknown) => true,
        // BUSY / WAITING は処理中、STARTING は起動直後（誤再開防止）。
        Some(AgentStatus::Busy) => false,
        Some(AgentStatus::Waiting) => false,
        Some(AgentStatus::Starting) => false,
    })
}

/// `task_status.md` の Markdown 表を解析して残存判定用エントリを返す純関数。
///
/// 期待フォーマット（`AI_PRJ_DESIGN.md` §4.2）:
/// ```text
/// | タスク ID | 状態 | 更新者 | 更新日時 | 備考 |
/// |----------|-----|-------|---------|------|
/// | T-001    | 完了 | foo  | ...    | -    |
/// ```
/// ヘッダ行と区切り行はスキップし、状態列が「未着手 / 進行中 / 完了 / ブロック」
/// のいずれかである行のみエントリとして採用する。
pub(crate) fn parse_task_status(content: &str) -> Vec<TaskStatusEntry> {
    let mut out = Vec::new();
    for raw in content.lines() {
        let line = raw.trim();
        if !line.starts_with('|') {
            continue;
        }
        // 区切り行 (|----|----|) を弾く。
        if line.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ')) {
            continue;
        }
        let cells: Vec<&str> = line
            .trim_matches('|')
            .split('|')
            .map(|s| s.trim())
            .collect();
        if cells.len() < 2 {
            continue;
        }
        let task_id = cells[0].to_string();
        let state = cells[1].to_string();
        // ヘッダ行（タスク ID / 状態 という日本語文字列が並ぶ）を弾く。
        if state == "状態" || task_id == "タスク ID" || task_id == "タスクID" {
            continue;
        }
        // 状態列の値が想定セット内に無ければスキップ（任意の備考行を除外）。
        if !matches!(state.as_str(), "未着手" | "進行中" | "完了" | "ブロック") {
            continue;
        }
        out.push(TaskStatusEntry { task_id, state });
    }
    out
}

/// 各 peer の `<workspace>/<peer>/task_status.md` を fs_read し、未消化タスクが
/// 1 つでもあれば `true` を返す。ファイル不在は残存なし扱い（誤抑制優先、NFR-4）。
pub(crate) fn has_pending_tasks(workspace_root: &Path, peers: &[String]) -> bool {
    peers.iter().any(|peer| {
        let path = workspace_root.join(peer).join("task_status.md");
        match std::fs::read_to_string(&path) {
            Ok(content) => parse_task_status(&content).iter().any(|e| e.is_pending()),
            Err(_) => false,
        }
    })
}

/// Phase 109 — `<workspace>/<peer>/task_status.md` 上の **全** タスクが「完了」であれば
/// `true` を返す純関数。エントリ 0 件 / ファイル不在は `false`（誤終了を防ぐ）。
pub(crate) fn peer_tasks_all_complete(workspace_root: &Path, peer: &str) -> bool {
    let path = workspace_root.join(peer).join("task_status.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return false;
    };
    let entries = parse_task_status(&content);
    !entries.is_empty() && entries.iter().all(|e| e.state == "完了")
}

/// Phase 109 — domain conductor のうち以下を満たすものの peer 名を返す純関数:
/// 1. `task_status.md` の全行が「完了」
/// 2. 当該 conductor 配下のサブエージェント (`<domain>-*` peer) が `targets` に
///    1 件も含まれない（= `agent-cli list` 上で既に消えている）
///
/// 順序保証: 配下 sub-agent が残存している間は conductor を返さない。
pub(crate) fn conductors_ready_to_terminate(
    targets: &[MonitorTarget],
    workspace_root: &Path,
) -> Vec<String> {
    let mut out = Vec::new();
    for c in targets.iter() {
        if !matches!(c.kind, MonitorKind::DomainConductor) {
            continue;
        }
        if !peer_tasks_all_complete(workspace_root, &c.peer) {
            continue;
        }
        let has_subagent = targets.iter().any(|t| {
            matches!(t.kind, MonitorKind::Subagent)
                && t.parent_conductor.as_deref() == Some(c.peer.as_str())
        });
        if has_subagent {
            continue;
        }
        out.push(c.peer.clone());
    }
    out
}

/// Phase 109 — `agent-cli list` 上の特定 status (IDLE / Error / Unknown) を判定する純関数。
/// 監視ループから「終了対象として安全な status か？」を問う用途。
pub(crate) fn is_terminable_status(status: AgentStatus) -> bool {
    matches!(
        status,
        AgentStatus::Idle | AgentStatus::Error | AgentStatus::Unknown
    )
}

/// Phase 109 — `peer` を graceful に終了させる。
///
/// 1. `pgrep -f "agent-cli run.*--name <peer>"` で関連 PID を抽出
/// 2. 各 PID に SIGTERM を送る
/// 3. `HESTIA_MONITOR_TERMINATE_GRACE_SECS`（既定 10 秒、0..=60 にクランプ）猶予後、
///    まだ生存していれば SIGKILL escalate
///
/// 重複 peer 状態（Phase 109 修正以前の状況）でも全 PID を網羅して停止する。
async fn terminate_peer(peer: &str) -> anyhow::Result<()> {
    let pids = pgrep_agent_cli_pids(peer).await;
    if pids.is_empty() {
        return Ok(());
    }
    eprintln!(
        "[monitor] terminating peer '{peer}' (PIDs: {})",
        pids.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    for &pid in &pids {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }

    let grace = Duration::from_secs(terminate_grace_secs());
    tokio::time::sleep(grace).await;

    // 残存している PID に対して SIGKILL escalate
    let still_alive = pgrep_agent_cli_pids(peer).await;
    for &pid in &still_alive {
        if pids.contains(&pid) {
            eprintln!("[monitor] SIGKILL escalation for '{peer}' PID {pid}");
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
        }
    }
    Ok(())
}

async fn pgrep_agent_cli_pids(peer: &str) -> Vec<u32> {
    let pattern = format!("agent-cli run.*--name {peer}");
    let Ok(out) = Command::new("pgrep")
        .arg("-f")
        .arg(&pattern)
        .output()
        .await
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.trim().parse::<u32>().ok())
        .collect()
}

/// Phase 109 — 終了猶予秒数（既定 10、`HESTIA_MONITOR_TERMINATE_GRACE_SECS` で上書き、0..=60 にクランプ）。
pub(crate) fn terminate_grace_secs() -> u64 {
    std::env::var("HESTIA_MONITOR_TERMINATE_GRACE_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(clamp_terminate_grace)
        .unwrap_or(10)
}

pub(crate) fn clamp_terminate_grace(s: u64) -> u64 {
    s.clamp(0, 60)
}

/// 状況サマリ（`hestia monitor` のヘッダ表示用）。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct StatusSummary {
    pub running: usize,
    pub busy: usize,
    pub idle: usize,
    pub waiting: usize,
    pub error: usize,
    pub starting: usize,
    pub unknown: usize,
}

pub(crate) fn summarize_statuses(statuses: &HashMap<String, AgentStatus>) -> StatusSummary {
    let mut s = StatusSummary::default();
    s.running = statuses.len();
    for v in statuses.values() {
        match v {
            AgentStatus::Busy => s.busy += 1,
            AgentStatus::Idle => s.idle += 1,
            AgentStatus::Waiting => s.waiting += 1,
            AgentStatus::Error => s.error += 1,
            AgentStatus::Starting => s.starting += 1,
            AgentStatus::Unknown => s.unknown += 1,
        }
    }
    s
}

pub(crate) fn build_monitor_header(
    summary: &StatusSummary,
    interval: u64,
    timestamp: &str,
) -> String {
    format!(
        "[Hestia Monitor]  refreshed: {ts}  (every {iv}s, Ctrl+C to exit)\n\
         Conductors: {run} running   BUSY: {b}   IDLE: {i}   WAITING: {w}   ERROR: {e}   STARTING: {st}   UNKNOWN: {u}\n\n",
        ts = timestamp,
        iv = interval,
        run = summary.running,
        b = summary.busy,
        i = summary.idle,
        w = summary.waiting,
        e = summary.error,
        st = summary.starting,
        u = summary.unknown,
    )
}

async fn run_agent_cli_list() -> Result<String> {
    let out = Command::new("agent-cli")
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to run agent-cli list: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn build_resume_message(peer: &str) -> String {
    format!(
        "[hestia monitor] あなた（{p}）の <workspace>/{p}/task_status.md と \
         <workspace>/{p}/tasks.md を fs_read で読み取り、未消化タスク\
         （未着手 / 進行中 / ブロック）から作業を再開してください。\
         作業再開手順は persona の「作業再開」セクションに従ってください。",
        p = peer
    )
}

async fn send_resume_instruction(peer: &str) -> Result<()> {
    let msg = build_resume_message(peer);
    let status = Command::new("agent-cli")
        .arg("send")
        .arg(peer)
        .arg(&msg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("agent-cli send {peer} failed: {e}"))?;
    if !status.success() {
        bail!("agent-cli send {peer} exited with {status}");
    }
    Ok(())
}

fn install_signal_handler() -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let s = stop.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            s.store(true, Ordering::SeqCst);
        }
    });
    stop
}

fn now_iso8601() -> String {
    use std::time::UNIX_EPOCH;
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 簡易 ISO 8601（chrono を Hestia 全体で workspace dep として持っているが、
    // hestia crate には未追加。format! ベースで秒精度の UTC 表示とする）。
    let days = (secs / 86_400) as i64;
    let h = ((secs % 86_400) / 3_600) as u32;
    let m = ((secs % 3_600) / 60) as u32;
    let s = (secs % 60) as u32;
    let (y, mo, d) = ymd_from_days_since_epoch(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// 1970-01-01 からの日数を YYYY-MM-DD に変換するシンプルな実装。
/// 4/100/400 ルールの閏年だけ考慮。秒精度の表示用なので timezone は UTC 固定。
fn ymd_from_days_since_epoch(mut days: i64) -> (i32, u32, u32) {
    let mut year: i32 = 1970;
    loop {
        let yd = if is_leap(year) { 366 } else { 365 };
        if days < yd {
            break;
        }
        days -= yd;
        year += 1;
    }
    let mlen = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month: u32 = 1;
    for &len in &mlen {
        if days < len as i64 {
            break;
        }
        days -= len as i64;
        month += 1;
    }
    (year, month, (days + 1) as u32)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

/// `hestia monitor-daemon` 本体（`Commands::MonitorDaemon` から起動）。
pub(crate) async fn run_monitor_daemon() -> Result<()> {
    if monitor_disabled() {
        eprintln!("[monitor] HESTIA_MONITOR_DISABLED=1 — daemon exiting without monitoring");
        return Ok(());
    }

    let interval = Duration::from_secs(monitor_interval_secs());
    let cooldown = Duration::from_secs(monitor_cooldown_secs());
    let stop = install_signal_handler();
    let mut last_resume_at: Option<Instant> = None;

    eprintln!(
        "[monitor] daemon started (interval={}s, cooldown={}s)",
        interval.as_secs(),
        cooldown.as_secs()
    );

    while !stop.load(Ordering::SeqCst) {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = wait_for_stop(stop.clone()) => break,
        }
        if stop.load(Ordering::SeqCst) {
            break;
        }

        let stdout = match run_agent_cli_list().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[monitor] agent-cli list failed: {e} — retrying next tick");
                continue;
            }
        };

        let statuses = collect_agent_statuses(&stdout, SystemTime::now());
        let targets = resolve_monitor_targets(&stdout);
        let workspace_root = workspaces_root();

        // ── Phase 109 ① 完了したサブエージェントを graceful 終了 ──
        for t in &targets {
            if !matches!(t.kind, MonitorKind::Subagent) {
                continue;
            }
            let Some(status) = statuses.get(&t.agent_id).copied() else {
                continue;
            };
            if !is_terminable_status(status) {
                continue;
            }
            if !peer_tasks_all_complete(&workspace_root, &t.peer) {
                continue;
            }
            if let Err(e) = terminate_peer(&t.peer).await {
                eprintln!("[monitor] terminate '{}' failed: {e}", t.peer);
            }
        }

        // ── Phase 109 ② 配下 sub-agent が居ない conductor のうち完了したものを終了 ──
        // ※ ① で消えた sub-agent は次周期の `agent-cli list` で消失する。同周期で
        //   同時消去しないため `targets`（= 旧スナップショット）を引数とする
        //   `conductors_ready_to_terminate` は安全側に倒れる（subagent 残存 →
        //   conductor 終了させない）。
        for peer in conductors_ready_to_terminate(&targets, &workspace_root) {
            if let Err(e) = terminate_peer(&peer).await {
                eprintln!("[monitor] terminate conductor '{peer}' failed: {e}");
            }
        }

        // ── Phase 108 ③ 全停止 + タスク残存 → 再開指示 ──
        if !is_all_stopped(&targets, &statuses) {
            continue;
        }

        if let Some(t) = last_resume_at {
            if t.elapsed() < cooldown {
                continue;
            }
        }

        let peers: Vec<String> = targets.iter().map(|t| t.peer.clone()).collect();

        if !has_pending_tasks(&workspace_root, &peers) {
            eprintln!("[monitor] all tasks complete; exiting monitor loop");
            break;
        }

        eprintln!(
            "[monitor] all {} agents stopped with pending tasks — sending resume instructions",
            targets.len()
        );
        for peer in &peers {
            match send_resume_instruction(peer).await {
                Ok(()) => eprintln!("[monitor] resume sent to {peer}"),
                Err(e) => eprintln!("[monitor] resume to {peer} failed: {e}"),
            }
        }
        last_resume_at = Some(Instant::now());
    }

    eprintln!("[monitor] daemon stopping");
    Ok(())
}

async fn wait_for_stop(stop: Arc<AtomicBool>) {
    loop {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// `hestia monitor` 本体（人間ユーザー向け、定期更新表示）。
pub(crate) async fn run_monitor(interval: u64, once: bool, all: bool) -> Result<()> {
    if once {
        return show_status(all).await;
    }

    let interval = clamp_view_interval(interval);
    let stop = install_signal_handler();
    let is_tty = atty_stdout();

    while !stop.load(Ordering::SeqCst) {
        match build_monitor_frame(all, interval).await {
            Ok(frame) => {
                if is_tty {
                    // ANSI: cursor home + clear screen + clear scrollback。
                    print!("\x1b[H\x1b[2J\x1b[3J{frame}");
                } else {
                    print!("{frame}\n");
                }
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
            Err(e) => {
                eprintln!("[monitor] frame build failed: {e}");
            }
        }
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(interval)) => {}
            _ = wait_for_stop(stop.clone()) => break,
        }
    }

    if is_tty {
        // 終了時にカーソルを左下に移動（次プロンプトと衝突しないように）。
        println!();
    }
    Ok(())
}

async fn build_monitor_frame(all: bool, interval: u64) -> Result<String> {
    let stdout = run_agent_cli_list().await?;
    let now = SystemTime::now();
    let statuses = collect_agent_statuses(&stdout, now);
    let summary = summarize_statuses(&statuses);
    let header = build_monitor_header(&summary, interval, &now_iso8601());
    let body = transform_status_listing(&stdout, all, &statuses);
    Ok(format!("{header}{body}"))
}

fn atty_stdout() -> bool {
    // libc::isatty(1) で stdout が端末か判定。失敗時は false 扱い（パイプ出力扱い）。
    unsafe { libc::isatty(1) == 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: &str, peer: &str, kind: MonitorKind) -> MonitorTarget {
        // テスト用ヘルパ: parent_conductor は kind から自動推定する
        // （Subagent → "ai" を仮置き、DomainConductor → None）。
        // 動的 sub-agent の親 conductor を別途指定したい場合は target_with_parent を使う。
        let parent = match kind {
            MonitorKind::Subagent => Some("ai".to_string()),
            MonitorKind::DomainConductor => None,
        };
        MonitorTarget {
            agent_id: id.to_string(),
            peer: peer.to_string(),
            kind,
            parent_conductor: parent,
        }
    }

    fn target_with_parent(
        id: &str,
        peer: &str,
        kind: MonitorKind,
        parent: Option<&str>,
    ) -> MonitorTarget {
        MonitorTarget {
            agent_id: id.to_string(),
            peer: peer.to_string(),
            kind,
            parent_conductor: parent.map(|s| s.to_string()),
        }
    }

    fn statuses(pairs: &[(&str, AgentStatus)]) -> HashMap<String, AgentStatus> {
        pairs.iter().map(|(k, v)| ((*k).to_string(), *v)).collect()
    }

    // ─── clamp helpers ────────────────────────────────────────────────────
    #[test]
    fn clamp_monitor_interval_bounds() {
        assert_eq!(clamp_monitor_interval(0), 5);
        assert_eq!(clamp_monitor_interval(4), 5);
        assert_eq!(clamp_monitor_interval(5), 5);
        assert_eq!(clamp_monitor_interval(30), 30);
        assert_eq!(clamp_monitor_interval(600), 600);
        assert_eq!(clamp_monitor_interval(601), 600);
        assert_eq!(clamp_monitor_interval(1_000_000), 600);
    }

    #[test]
    fn clamp_monitor_cooldown_bounds() {
        assert_eq!(clamp_monitor_cooldown(0), 0);
        assert_eq!(clamp_monitor_cooldown(60), 60);
        assert_eq!(clamp_monitor_cooldown(600), 600);
        assert_eq!(clamp_monitor_cooldown(601), 600);
    }

    #[test]
    fn clamp_view_interval_bounds() {
        assert_eq!(clamp_view_interval(0), 1);
        assert_eq!(clamp_view_interval(1), 1);
        assert_eq!(clamp_view_interval(2), 2);
        assert_eq!(clamp_view_interval(60), 60);
        assert_eq!(clamp_view_interval(61), 60);
        assert_eq!(clamp_view_interval(10_000), 60);
    }

    // ─── resolve_monitor_targets ─────────────────────────────────────────
    const HEADER_LIST: &str =
        "ID                                NAME          PROVIDER  MODEL          ROLE\n";

    #[test]
    fn resolve_picks_subagents_and_domain_only() {
        let body = "agent-AAA  ai          ollama  glm  meta\n\
                    agent-BBB  ai-designer ollama  glm  designer\n\
                    agent-CCC  ai-reviewer ollama  glm  reviewer\n\
                    agent-DDD  rtl         ollama  glm  conductor\n\
                    agent-EEE  bogus       ollama  glm  ???\n";
        let input = format!("{HEADER_LIST}{body}");
        let got = resolve_monitor_targets(&input);
        let names: Vec<&str> = got.iter().map(|t| t.peer.as_str()).collect();
        assert_eq!(names, vec!["ai-designer", "ai-reviewer", "rtl"]);
        assert_eq!(got[0].kind, MonitorKind::Subagent);
        assert_eq!(got[2].kind, MonitorKind::DomainConductor);
    }

    #[test]
    fn resolve_ignores_non_agent_rows() {
        let body = "0001       ai          ollama  glm  meta\n\
                    agent-BBB  ai-designer ollama  glm  designer\n";
        let input = format!("{HEADER_LIST}{body}");
        let got = resolve_monitor_targets(&input);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].peer, "ai-designer");
    }

    #[test]
    fn resolve_handles_empty_and_header_only() {
        assert!(resolve_monitor_targets("").is_empty());
        assert!(resolve_monitor_targets(HEADER_LIST).is_empty());
    }

    // ─── is_all_stopped ──────────────────────────────────────────────────
    #[test]
    fn all_stopped_empty_targets_returns_false() {
        let map: HashMap<String, AgentStatus> = HashMap::new();
        assert!(!is_all_stopped(&[], &map));
    }

    #[test]
    fn all_stopped_when_every_target_idle() {
        let t = vec![
            target("agent-A", "ai-designer", MonitorKind::Subagent),
            target("agent-B", "ai-reviewer", MonitorKind::Subagent),
        ];
        let s = statuses(&[
            ("agent-A", AgentStatus::Idle),
            ("agent-B", AgentStatus::Idle),
        ]);
        assert!(is_all_stopped(&t, &s));
    }

    #[test]
    fn all_stopped_when_one_busy_returns_false() {
        let t = vec![
            target("agent-A", "ai-designer", MonitorKind::Subagent),
            target("agent-B", "ai-reviewer", MonitorKind::Subagent),
        ];
        let s = statuses(&[
            ("agent-A", AgentStatus::Idle),
            ("agent-B", AgentStatus::Busy),
        ]);
        assert!(!is_all_stopped(&t, &s));
    }

    #[test]
    fn all_stopped_when_starting_returns_false() {
        // STARTING は稼働中扱い（誤再開防止）。
        let t = vec![target("agent-A", "ai-designer", MonitorKind::Subagent)];
        let s = statuses(&[("agent-A", AgentStatus::Starting)]);
        assert!(!is_all_stopped(&t, &s));
    }

    #[test]
    fn all_stopped_when_waiting_returns_false() {
        let t = vec![target("agent-A", "ai-designer", MonitorKind::Subagent)];
        let s = statuses(&[("agent-A", AgentStatus::Waiting)]);
        assert!(!is_all_stopped(&t, &s));
    }

    #[test]
    fn all_stopped_when_error_or_unknown_or_missing() {
        let t = vec![
            target("agent-A", "ai-designer", MonitorKind::Subagent),
            target("agent-B", "ai-reviewer", MonitorKind::Subagent),
            target("agent-C", "rtl", MonitorKind::DomainConductor),
        ];
        let s = statuses(&[
            ("agent-A", AgentStatus::Error),
            ("agent-B", AgentStatus::Unknown),
            // agent-C 不在
        ]);
        assert!(is_all_stopped(&t, &s));
    }

    #[test]
    fn all_stopped_mixed_idle_and_busy_false() {
        let t = vec![
            target("agent-A", "ai-designer", MonitorKind::Subagent),
            target("agent-B", "rtl", MonitorKind::DomainConductor),
            target("agent-C", "fpga", MonitorKind::DomainConductor),
        ];
        let s = statuses(&[
            ("agent-A", AgentStatus::Idle),
            ("agent-B", AgentStatus::Busy),
            ("agent-C", AgentStatus::Error),
        ]);
        assert!(!is_all_stopped(&t, &s));
    }

    // ─── parse_task_status ───────────────────────────────────────────────
    #[test]
    fn parse_task_status_header_only_returns_empty() {
        let md = "# header\n\n| タスク ID | 状態 | 更新者 | 更新日時 | 備考 |\n|----|----|----|----|----|\n";
        assert!(parse_task_status(md).is_empty());
    }

    #[test]
    fn parse_task_status_picks_pending_rows() {
        let md = "\
| タスク ID | 状態 | 更新者 | 更新日時 | 備考 |
|----|----|----|----|----|
| T-001 | 完了 | foo | t1 | - |
| T-002 | 進行中 | foo | t2 | - |
| T-003 | 未着手 | foo | t3 | - |
| T-004 | ブロック | foo | t4 | - |
";
        let entries = parse_task_status(md);
        assert_eq!(entries.len(), 4);
        assert!(entries[0].is_pending() == false);
        assert!(entries[1].is_pending());
        assert!(entries[2].is_pending());
        assert!(entries[3].is_pending());
    }

    #[test]
    fn parse_task_status_all_complete_no_pending() {
        let md = "\
| タスク ID | 状態 | 更新者 | 更新日時 | 備考 |
|----|----|----|----|----|
| T-001 | 完了 | foo | t1 | - |
| T-002 | 完了 | foo | t2 | - |
";
        let entries = parse_task_status(md);
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| !e.is_pending()));
    }

    #[test]
    fn parse_task_status_ignores_garbage() {
        let md = "なにかのテキスト\n適当な行\n";
        assert!(parse_task_status(md).is_empty());
    }

    #[test]
    fn parse_task_status_ignores_unknown_state_values() {
        let md = "\
| タスク ID | 状態 | 更新者 |
|----|----|----|
| T-001 | XXX | foo |
| T-002 | 進行中 | foo |
";
        let entries = parse_task_status(md);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].task_id, "T-002");
    }

    // ─── has_pending_tasks ───────────────────────────────────────────────
    #[test]
    fn has_pending_tasks_returns_true_when_any_peer_has_pending() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let p1 = root.join("ai-designer");
        let p2 = root.join("rtl");
        std::fs::create_dir_all(&p1).unwrap();
        std::fs::create_dir_all(&p2).unwrap();
        std::fs::write(
            p1.join("task_status.md"),
            "| タスク ID | 状態 |\n|----|----|\n| T-001 | 完了 |\n",
        )
        .unwrap();
        std::fs::write(
            p2.join("task_status.md"),
            "| タスク ID | 状態 |\n|----|----|\n| T-002 | 進行中 |\n",
        )
        .unwrap();
        let peers = vec!["ai-designer".to_string(), "rtl".to_string()];
        assert!(has_pending_tasks(root, &peers));
    }

    #[test]
    fn has_pending_tasks_false_when_all_complete() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let p = root.join("ai-designer");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(
            p.join("task_status.md"),
            "| タスク ID | 状態 |\n|----|----|\n| T-001 | 完了 |\n",
        )
        .unwrap();
        let peers = vec!["ai-designer".to_string()];
        assert!(!has_pending_tasks(root, &peers));
    }

    #[test]
    fn has_pending_tasks_missing_file_treated_as_no_pending() {
        let dir = tempfile::tempdir().unwrap();
        let peers = vec!["ai-designer".to_string()];
        // ファイル不在 → 残存なし扱い（NFR-4 誤抑制優先）。
        assert!(!has_pending_tasks(dir.path(), &peers));
    }

    // ─── summarize_statuses & build_monitor_header ───────────────────────
    #[test]
    fn summarize_counts_all_kinds() {
        let s = statuses(&[
            ("agent-A", AgentStatus::Busy),
            ("agent-B", AgentStatus::Idle),
            ("agent-C", AgentStatus::Idle),
            ("agent-D", AgentStatus::Waiting),
            ("agent-E", AgentStatus::Error),
            ("agent-F", AgentStatus::Starting),
            ("agent-G", AgentStatus::Unknown),
        ]);
        let sum = summarize_statuses(&s);
        assert_eq!(sum.running, 7);
        assert_eq!(sum.busy, 1);
        assert_eq!(sum.idle, 2);
        assert_eq!(sum.waiting, 1);
        assert_eq!(sum.error, 1);
        assert_eq!(sum.starting, 1);
        assert_eq!(sum.unknown, 1);
    }

    #[test]
    fn summarize_empty_input() {
        let s: HashMap<String, AgentStatus> = HashMap::new();
        let sum = summarize_statuses(&s);
        assert_eq!(sum.running, 0);
        assert_eq!(sum.busy, 0);
    }

    #[test]
    fn header_contains_all_columns() {
        let summary = StatusSummary {
            running: 3,
            busy: 1,
            idle: 1,
            waiting: 0,
            error: 0,
            starting: 1,
            unknown: 0,
        };
        let h = build_monitor_header(&summary, 2, "2026-05-08T12:00:00Z");
        assert!(h.contains("Hestia Monitor"));
        assert!(h.contains("refreshed: 2026-05-08T12:00:00Z"));
        assert!(h.contains("every 2s"));
        assert!(h.contains("3 running"));
        assert!(h.contains("BUSY: 1"));
        assert!(h.contains("IDLE: 1"));
        assert!(h.contains("STARTING: 1"));
    }

    // ─── build_resume_message ────────────────────────────────────────────
    #[test]
    fn resume_message_includes_peer_and_paths() {
        let m = build_resume_message("rtl");
        assert!(m.contains("rtl"));
        assert!(m.contains("task_status.md"));
        assert!(m.contains("tasks.md"));
        assert!(m.contains("作業再開"));
        assert!(m.contains("未着手"));
        assert!(m.contains("進行中"));
        assert!(m.contains("ブロック"));
    }

    // ─── ymd_from_days_since_epoch ───────────────────────────────────────
    #[test]
    fn ymd_epoch_origin() {
        assert_eq!(ymd_from_days_since_epoch(0), (1970, 1, 1));
    }

    #[test]
    fn ymd_known_date() {
        // 2026-01-01 は epoch から 20454 日後。
        // 1970..=2025 の閏年は 1972..=2024 までの 4 で割り切れる年（100/400 例外なし）= 14 個。
        // 365 * 56 + 14 = 20440 + 14 = 20454。
        assert_eq!(ymd_from_days_since_epoch(20454), (2026, 1, 1));
    }

    // ─── classify_peer (Phase 109) ─────────────────────────────────────
    #[test]
    fn classify_peer_resident_subagents() {
        let (k, p) = classify_peer("ai-designer").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("ai".to_string()));
        let (k, p) = classify_peer("ai-reviewer").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("ai".to_string()));
    }

    #[test]
    fn classify_peer_domain_conductors() {
        let (k, p) = classify_peer("rtl").unwrap();
        assert_eq!(k, MonitorKind::DomainConductor);
        assert_eq!(p, None);
        let (k, p) = classify_peer("asic").unwrap();
        assert_eq!(k, MonitorKind::DomainConductor);
        assert_eq!(p, None);
    }

    #[test]
    fn classify_peer_dynamic_subagents() {
        let (k, p) = classify_peer("rtl-coder-uart").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("rtl".to_string()));
        let (k, p) = classify_peer("asic-signoff").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("asic".to_string()));
        let (k, p) = classify_peer("hal-designer").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("hal".to_string()));
    }

    #[test]
    fn classify_peer_excludes_ai_self() {
        assert!(classify_peer("ai").is_none());
    }

    #[test]
    fn classify_peer_unknown_returns_none() {
        assert!(classify_peer("bogus").is_none());
        assert!(classify_peer("nonsense-xyz").is_none());
    }

    // ─── peer_tasks_all_complete (Phase 109) ────────────────────────────
    #[test]
    fn peer_tasks_all_complete_returns_true_when_all_complete() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("ai-designer");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(
            p.join("task_status.md"),
            "| タスク ID | 状態 |\n|----|----|\n| T-001 | 完了 |\n| T-002 | 完了 |\n",
        )
        .unwrap();
        assert!(peer_tasks_all_complete(dir.path(), "ai-designer"));
    }

    #[test]
    fn peer_tasks_all_complete_returns_false_when_pending() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("rtl");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(
            p.join("task_status.md"),
            "| タスク ID | 状態 |\n|----|----|\n| T-001 | 完了 |\n| T-002 | 進行中 |\n",
        )
        .unwrap();
        assert!(!peer_tasks_all_complete(dir.path(), "rtl"));
    }

    #[test]
    fn peer_tasks_all_complete_returns_false_when_no_entries() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("ai-reviewer");
        std::fs::create_dir_all(&p).unwrap();
        // ヘッダだけでエントリ 0 件 → タスク未定義扱いで false（誤終了防止）。
        std::fs::write(
            p.join("task_status.md"),
            "# header\n\n| タスク ID | 状態 |\n|----|----|\n",
        )
        .unwrap();
        assert!(!peer_tasks_all_complete(dir.path(), "ai-reviewer"));
    }

    #[test]
    fn peer_tasks_all_complete_returns_false_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        // ファイル不在 → false（誤終了防止）。
        assert!(!peer_tasks_all_complete(dir.path(), "rtl"));
    }

    // ─── conductors_ready_to_terminate (Phase 109) ──────────────────────
    fn write_status(dir: &Path, peer: &str, body: &str) {
        let p = dir.join(peer);
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join("task_status.md"), body).unwrap();
    }

    #[test]
    fn conductors_ready_when_no_subagent_and_complete() {
        let dir = tempfile::tempdir().unwrap();
        write_status(
            dir.path(),
            "rtl",
            "| タスク ID | 状態 |\n|----|----|\n| T-001 | 完了 |\n",
        );
        let targets = vec![target_with_parent(
            "agent-A",
            "rtl",
            MonitorKind::DomainConductor,
            None,
        )];
        let got = conductors_ready_to_terminate(&targets, dir.path());
        assert_eq!(got, vec!["rtl".to_string()]);
    }

    #[test]
    fn conductors_blocked_when_subagent_still_present() {
        let dir = tempfile::tempdir().unwrap();
        write_status(
            dir.path(),
            "rtl",
            "| タスク ID | 状態 |\n|----|----|\n| T-001 | 完了 |\n",
        );
        let targets = vec![
            target_with_parent("agent-A", "rtl", MonitorKind::DomainConductor, None),
            target_with_parent(
                "agent-B",
                "rtl-coder-uart",
                MonitorKind::Subagent,
                Some("rtl"),
            ),
        ];
        // 配下 sub-agent が残存 → conductor 終了を留保
        assert!(conductors_ready_to_terminate(&targets, dir.path()).is_empty());
    }

    #[test]
    fn conductors_blocked_when_tasks_pending() {
        let dir = tempfile::tempdir().unwrap();
        write_status(
            dir.path(),
            "asic",
            "| タスク ID | 状態 |\n|----|----|\n| T-001 | 進行中 |\n",
        );
        let targets = vec![target_with_parent(
            "agent-A",
            "asic",
            MonitorKind::DomainConductor,
            None,
        )];
        assert!(conductors_ready_to_terminate(&targets, dir.path()).is_empty());
    }

    #[test]
    fn conductors_ready_empty_targets_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(conductors_ready_to_terminate(&[], dir.path()).is_empty());
    }

    // ─── is_terminable_status / clamp_terminate_grace (Phase 109) ──────
    #[test]
    fn is_terminable_only_for_idle_error_unknown() {
        assert!(is_terminable_status(AgentStatus::Idle));
        assert!(is_terminable_status(AgentStatus::Error));
        assert!(is_terminable_status(AgentStatus::Unknown));
        assert!(!is_terminable_status(AgentStatus::Busy));
        assert!(!is_terminable_status(AgentStatus::Waiting));
        assert!(!is_terminable_status(AgentStatus::Starting));
    }

    #[test]
    fn clamp_terminate_grace_bounds() {
        assert_eq!(clamp_terminate_grace(0), 0);
        assert_eq!(clamp_terminate_grace(10), 10);
        assert_eq!(clamp_terminate_grace(60), 60);
        assert_eq!(clamp_terminate_grace(61), 60);
        assert_eq!(clamp_terminate_grace(10_000), 60);
    }

    // ─── resolve_monitor_targets (Phase 109 拡張: 動的 sub-agent 採用) ──
    #[test]
    fn resolve_includes_dynamic_subagents() {
        let body = "agent-AAA  ai          ollama  glm  meta\n\
                    agent-BBB  ai-designer ollama  glm  designer\n\
                    agent-CCC  rtl         ollama  glm  conductor\n\
                    agent-DDD  rtl-coder-uart ollama glm worker\n\
                    agent-EEE  asic-signoff   ollama glm worker\n\
                    agent-FFF  bogus       ollama  glm  ???\n";
        let input = format!("{HEADER_LIST}{body}");
        let got = resolve_monitor_targets(&input);
        let names: Vec<&str> = got.iter().map(|t| t.peer.as_str()).collect();
        assert_eq!(
            names,
            vec!["ai-designer", "rtl", "rtl-coder-uart", "asic-signoff"]
        );
        // parent_conductor の検証
        assert_eq!(got[0].parent_conductor.as_deref(), Some("ai"));
        assert_eq!(got[1].parent_conductor, None);
        assert_eq!(got[2].parent_conductor.as_deref(), Some("rtl"));
        assert_eq!(got[3].parent_conductor.as_deref(), Some("asic"));
    }
}
