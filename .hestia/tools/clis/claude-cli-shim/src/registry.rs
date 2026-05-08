//! Peer registry の永続化。
//!
//! 配置: `<registry_dir>/<peer>.json`
//! agent-cli 互換の `list` は `ID NAME PROVIDER MODEL ROLE [SKILLS]` 列を出力する。

use crate::config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEntry {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model: String,
    pub role: String,
    #[serde(default)]
    pub skills: Vec<String>,
    pub pid: u32,
    pub fifo_path: String,
    pub log_path: String,
    pub started_at: String,
}

pub fn entry_path(registry_dir: &Path, peer: &str) -> PathBuf {
    registry_dir.join(format!("{peer}.json"))
}

pub fn write_entry(registry_dir: &Path, entry: &PeerEntry) -> Result<()> {
    fs::create_dir_all(registry_dir)
        .with_context(|| format!("create_dir_all {}", registry_dir.display()))?;
    let path = entry_path(registry_dir, &entry.name);
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(entry)?;
    fs::write(&tmp, json).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, &path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

pub fn remove_entry(registry_dir: &Path, peer: &str) -> Result<()> {
    let path = entry_path(registry_dir, peer);
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
    }
    Ok(())
}

pub fn read_entry(registry_dir: &Path, peer: &str) -> Result<Option<PeerEntry>> {
    let path = entry_path(registry_dir, peer);
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let entry: PeerEntry = serde_json::from_str(&text)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(entry))
}

pub fn read_all(registry_dir: &Path) -> Result<Vec<PeerEntry>> {
    if !registry_dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for ent in fs::read_dir(registry_dir)
        .with_context(|| format!("read_dir {}", registry_dir.display()))?
    {
        let ent = ent?;
        let path = ent.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        if let Ok(entry) = serde_json::from_str::<PeerEntry>(&text) {
            out.push(entry);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// `list` subcommand の本体。agent-cli 互換の整形済テキストを stdout に出力。
///
/// Phase 115 — `hestia status` の列検出（≥2 spaces を separator とみなす）に
/// 確実に追従するため、各列間に **2 spaces を literal で挿入** し、ID 列の
/// 既定幅も `shim-<uuid>` (= 41 chars) に合わせる。1 space 区切り + `{:<36}`
/// だと shim ID が overflow した時に ID-NAME 間が 1 space に縮退し、hestia の
/// `nth_separator_end(line, 2)` が separator 1 を取りこぼして STATUS 列を
/// PROVIDER と MODEL の間に誤挿入していた。
pub fn list(registry_path: Option<PathBuf>) -> Result<()> {
    let dir = config::registry_dir(registry_path);
    let entries = read_all(&dir)?;
    println!("ID                                         NAME                  PROVIDER   MODEL                        ROLE           SKILLS");
    for e in entries {
        let skills = if e.skills.is_empty() {
            String::new()
        } else {
            e.skills.join(",")
        };
        println!(
            "{:<41}  {:<20}  {:<9}  {:<27}  {:<13}  {}",
            e.id, e.name, e.provider, e.model, e.role, skills
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_entry(name: &str) -> PeerEntry {
        PeerEntry {
            id: format!("shim-{name}-id"),
            name: name.to_string(),
            provider: "claude".to_string(),
            model: "claude-opus-4-7".to_string(),
            role: "designer".to_string(),
            skills: vec!["spec".into(), "review".into()],
            pid: 12345,
            fifo_path: format!("/tmp/claude-cli-shim-{name}.fifo"),
            log_path: format!("/tmp/log-{name}/session.jsonl"),
            started_at: "2026-05-08T16:30:00Z".to_string(),
        }
    }

    #[test]
    fn write_then_read_returns_same_entry() {
        let dir = tempdir().unwrap();
        let e = sample_entry("alpha");
        write_entry(dir.path(), &e).unwrap();
        let got = read_entry(dir.path(), "alpha").unwrap().unwrap();
        assert_eq!(got.name, "alpha");
        assert_eq!(got.provider, "claude");
        assert_eq!(got.skills, vec!["spec", "review"]);
    }

    #[test]
    fn read_all_sorted_by_name() {
        let dir = tempdir().unwrap();
        write_entry(dir.path(), &sample_entry("beta")).unwrap();
        write_entry(dir.path(), &sample_entry("alpha")).unwrap();
        write_entry(dir.path(), &sample_entry("gamma")).unwrap();
        let all = read_all(dir.path()).unwrap();
        let names: Vec<_> = all.iter().map(|e| e.name.clone()).collect();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn remove_entry_removes_file() {
        let dir = tempdir().unwrap();
        write_entry(dir.path(), &sample_entry("zeta")).unwrap();
        assert!(read_entry(dir.path(), "zeta").unwrap().is_some());
        remove_entry(dir.path(), "zeta").unwrap();
        assert!(read_entry(dir.path(), "zeta").unwrap().is_none());
    }

    #[test]
    fn read_all_empty_when_dir_missing() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let v = read_all(&missing).unwrap();
        assert!(v.is_empty());
    }
}
