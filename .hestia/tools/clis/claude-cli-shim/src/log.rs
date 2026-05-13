//! agent-cli compatible JSONL log writer.
//!
//! Schema (`derive_status_from_log` compatible):
//! ```jsonl
//! {"kind":"thinking","ts":"...","text":"..."}
//! {"kind":"tool_call","ts":"...","tool":"Read","args":{...}}
//! {"kind":"tool_result","ts":"...","tool":"Read","ok":true,"result":"..."}
//! {"kind":"user","ts":"...","text":"..."}
//! {"kind":"assistant","ts":"...","text":"..."}
//! ```

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct LogWriter {
    file: File,
}

impl LogWriter {
    /// Create or append to `<log_dir>/<agent_id>/session.jsonl`.
    pub fn open(log_dir: &Path, agent_id: &str) -> Result<(Self, PathBuf)> {
        let dir = log_dir.join(agent_id);
        fs::create_dir_all(&dir)
            .with_context(|| format!("create_dir_all {}", dir.display()))?;
        let path = dir.join("session.jsonl");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("open log {}", path.display()))?;
        Ok((Self { file }, path))
    }

    pub fn write_event(&mut self, event: &Value) -> Result<()> {
        let line = serde_json::to_string(event)?;
        self.file.write_all(line.as_bytes())?;
        self.file.write_all(b"\n")?;
        self.file.flush()?;
        Ok(())
    }

    /// Convenience helper: write an event with only `kind` + `text`.
    pub fn write_simple(&mut self, kind: &str, text: &str) -> Result<()> {
        let ev = serde_json::json!({
            "kind": kind,
            "ts": chrono::Utc::now().to_rfc3339(),
            "text": text,
        });
        self.write_event(&ev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn open_creates_session_file() {
        let dir = tempdir().unwrap();
        let (_w, path) = LogWriter::open(dir.path(), "shim-test").unwrap();
        assert!(path.exists());
        assert!(path.ends_with("shim-test/session.jsonl"));
    }

    #[test]
    fn write_simple_appends_jsonl() {
        let dir = tempdir().unwrap();
        let (mut w, path) = LogWriter::open(dir.path(), "shim-test").unwrap();
        w.write_simple("thinking", "hello").unwrap();
        w.write_simple("user", "world").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.trim().split('\n').collect();
        assert_eq!(lines.len(), 2);
        let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(first["kind"], "thinking");
        assert_eq!(first["text"], "hello");
    }
}