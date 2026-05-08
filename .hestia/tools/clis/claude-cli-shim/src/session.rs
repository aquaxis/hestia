//! 永続 session 実装。
//!
//! `Run` subcommand から呼び出され、`claude` プロセスを子として保持し、
//! FIFO 経由で受信した text を Claude の stdin に流し込み、Claude の
//! stream-json stdout を `transcoder` 経由で agent-cli 互換 JSONL に
//! 変換して log に追記する。

use crate::{config, ipc, log as shim_log, registry, transcoder};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

pub struct RunOpts {
    pub persona: PathBuf,
    pub name: String,
    pub auto_approve_tools: bool,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub registry_path: Option<PathBuf>,
    pub log_path: Option<PathBuf>,
}

pub async fn run(opts: RunOpts) -> Result<()> {
    let registry_dir = config::registry_dir(opts.registry_path.clone());
    let log_dir_root = config::log_dir(opts.log_path.clone());

    // persona ファイルを読込、フロントマター解析
    let persona_text = std::fs::read_to_string(&opts.persona)
        .with_context(|| format!("read persona {}", opts.persona.display()))?;
    let persona = parse_persona(&persona_text);

    // agent ID 生成
    let agent_id = format!("shim-{}", uuid::Uuid::new_v4());

    // log writer 起動
    let (mut log, log_path) = shim_log::LogWriter::open(&log_dir_root, &agent_id)?;
    log.write_simple("system", &format!("claude-cli-shim run start peer={}", opts.name))?;

    // FIFO 作成
    let fifo_path = config::fifo_path(&opts.name);
    ipc::ensure_fifo(&fifo_path)?;

    // registry 登録
    let entry = registry::PeerEntry {
        id: agent_id.clone(),
        name: opts.name.clone(),
        provider: opts.provider.clone().unwrap_or_else(|| "claude".to_string()),
        model: opts
            .model
            .clone()
            .unwrap_or_else(|| "claude-opus-4-7".to_string()),
        role: persona.role.clone(),
        skills: persona.skills.clone(),
        pid: std::process::id(),
        fifo_path: fifo_path.to_string_lossy().to_string(),
        log_path: log_path.to_string_lossy().to_string(),
        started_at: Utc::now().to_rfc3339(),
    };
    registry::write_entry(&registry_dir, &entry)?;

    // claude プロセス起動
    let mut claude = spawn_claude(&opts, &persona).await?;
    let stdin = claude
        .stdin
        .take()
        .ok_or_else(|| anyhow!("failed to capture claude stdin"))?;
    let stdout = claude
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to capture claude stdout"))?;
    let stderr = claude
        .stderr
        .take()
        .ok_or_else(|| anyhow!("failed to capture claude stderr"))?;

    // Phase 113: claude の stderr を log_dir/<agent_id>/claude_stderr.log に記録。
    // stream-json の起動失敗（モデル不正、API キー欠如等）を診断するための情報源。
    let stderr_log_path = log_path
        .parent()
        .map(|p| p.join("claude_stderr.log"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/claude_stderr.log"));
    let stderr_path_for_msg = stderr_log_path.clone();
    tokio::spawn(async move {
        let Ok(mut f) = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&stderr_log_path)
            .await
        else {
            return;
        };
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            use tokio::io::AsyncWriteExt as _;
            let _ = f.write_all(format!("{line}\n").as_bytes()).await;
            let _ = f.flush().await;
        }
    });

    // stdout reader task
    let mut log_for_reader = log; // move into task
    let _ = log_for_reader.write_simple(
        "system",
        &format!(
            "claude spawned, stderr -> {}",
            stderr_path_for_msg.display()
        ),
    );
    let reader_handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
                let _ = log_for_reader.write_simple("raw", &line);
                continue;
            };
            for ev in transcoder::transcode_event(&value) {
                let _ = log_for_reader.write_event(&ev);
            }
        }
    });

    // FIFO reader task: 受信 text を Claude stdin にフィード
    let stdin_arc = std::sync::Arc::new(tokio::sync::Mutex::new(stdin));
    let stdin_for_fifo = stdin_arc.clone();
    let fifo_path_clone = fifo_path.clone();
    let fifo_handle = tokio::spawn(async move {
        loop {
            let file = match tokio::fs::OpenOptions::new()
                .read(true)
                .open(&fifo_path_clone)
                .await
            {
                Ok(f) => f,
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    continue;
                }
            };
            let mut lines = BufReader::new(file).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let payload = serde_json::json!({
                    "type": "user",
                    "message": {
                        "role": "user",
                        "content": line,
                    }
                });
                let mut s = stdin_for_fifo.lock().await;
                if s.write_all(payload.to_string().as_bytes()).await.is_err() {
                    return;
                }
                let _ = s.write_all(b"\n").await;
                let _ = s.flush().await;
            }
        }
    });

    // claude プロセス終了 or 外部 SIGTERM 待機
    let status = claude.wait().await.context("claude process wait")?;
    let _ = reader_handle.await;
    fifo_handle.abort();

    // クリーンアップ: registry / fifo
    let _ = registry::remove_entry(&registry_dir, &opts.name);
    let _ = std::fs::remove_file(&fifo_path);

    if !status.success() {
        return Err(anyhow!("claude process exited with {status}"));
    }
    Ok(())
}

#[derive(Debug, Default)]
struct PersonaMeta {
    body: String,
    name: String,
    description: String,
    role: String,
    skills: Vec<String>,
}

fn parse_persona(text: &str) -> PersonaMeta {
    let mut meta = PersonaMeta::default();
    let trimmed = text.trim_start();
    if !trimmed.starts_with("---") {
        meta.body = text.to_string();
        return meta;
    }
    // ---\n ... \n---\n の YAML フロントマター
    let after_first = &trimmed[3..];
    let Some(end_idx) = after_first.find("\n---") else {
        meta.body = text.to_string();
        return meta;
    };
    let yaml_text = &after_first[..end_idx];
    let body_after = &after_first[end_idx + 4..]; // skip "\n---"
    meta.body = body_after.trim_start_matches('\n').to_string();

    // YAML フロントマター解析（簡易: 必要なキーのみ抽出）
    for line in yaml_text.lines() {
        let line = line.trim_end();
        if let Some(rest) = line.strip_prefix("name:") {
            meta.name = rest.trim().trim_matches('"').to_string();
        } else if let Some(rest) = line.strip_prefix("description:") {
            meta.description = rest.trim().trim_matches('"').to_string();
        } else if let Some(rest) = line.strip_prefix("role:") {
            meta.role = rest.trim().trim_matches('"').to_string();
        } else if line.trim() == "skills:" {
            // 次行以降の `- xxx` を集める処理は簡易実装で省略
            // skills は配列形式想定だが、ここでは空のまま（registry には空 vec）
        } else if let Some(rest) = line.strip_prefix("-") {
            let s = rest.trim().to_string();
            if !s.is_empty() && !meta.role.is_empty() {
                meta.skills.push(s);
            }
        }
    }
    if meta.role.is_empty() {
        // role が persona に無ければ name を fallback
        meta.role = meta.name.clone();
    }
    meta
}

async fn spawn_claude(opts: &RunOpts, persona: &PersonaMeta) -> Result<Child> {
    let mut cmd = Command::new("claude");
    // Phase 113 注: claude の `--input-format stream-json` / `--output-format
    // stream-json` は `--print` + `--verbose` の組み合わせが必須。stream-json
    // input は EOF まで複数 message を受け付けるため、shim は stdin を keep open
    // にしたまま FIFO 経由で input を流し込み、永続 session を実現する。
    cmd.arg("--print")
        .arg("--verbose")
        .arg("--input-format")
        .arg("stream-json")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--include-partial-messages");
    if opts.auto_approve_tools {
        cmd.arg("--dangerously-skip-permissions");
    }
    if let Some(model) = &opts.model {
        cmd.arg("--model").arg(model);
    }
    // persona 本文を `--append-system-prompt` で渡す
    if !persona.body.is_empty() {
        cmd.arg("--append-system-prompt").arg(&persona.body);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // Phase 113: stderr は log dir の `claude_stderr.log` に記録して診断容易化。
        .stderr(Stdio::piped());
    cmd.spawn().context("spawn claude")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_persona_no_frontmatter_returns_text_as_body() {
        let text = "Just a plain persona body.";
        let m = parse_persona(text);
        assert_eq!(m.body, "Just a plain persona body.");
        assert!(m.name.is_empty());
    }

    #[test]
    fn parse_persona_with_frontmatter_extracts_keys() {
        let text = "---\nname: ai-designer\ndescription: \"the designer\"\nrole: designer\n---\n\nPersona body here.";
        let m = parse_persona(text);
        assert_eq!(m.name, "ai-designer");
        assert_eq!(m.description, "the designer");
        assert_eq!(m.role, "designer");
        assert!(m.body.starts_with("Persona body"));
    }

    #[test]
    fn parse_persona_role_falls_back_to_name() {
        let text = "---\nname: foo\n---\nbody";
        let m = parse_persona(text);
        assert_eq!(m.role, "foo");
    }
}
