//! FIFO ベースの peer 間 IPC。
//!
//! - `Run` 起動時: `/tmp/claude-cli-shim-<peer>.fifo` を mkfifo (0600) で作成。
//! - 受信: 別 task で FIFO を非同期 read、行単位で channel に push。
//! - 送信 (`Send` subcommand): registry を引き、対応 FIFO に書き込む。

use crate::{config, registry};
use anyhow::{anyhow, bail, Context, Result};
use std::ffi::CString;
use std::path::{Path, PathBuf};

/// FIFO を mkfifo で作成（既に存在する場合は無視）。
pub fn ensure_fifo(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    let cstr = CString::new(path.as_os_str().to_string_lossy().as_bytes())
        .context("CString::new fifo path")?;
    let rc = unsafe { libc::mkfifo(cstr.as_ptr(), 0o600) };
    if rc != 0 {
        let err = std::io::Error::last_os_error();
        // EEXIST は問題なし（race conditon で別プロセスが先に作った）
        if err.raw_os_error() != Some(libc::EEXIST) {
            return Err(anyhow!("mkfifo {} failed: {err}", path.display()));
        }
    }
    Ok(())
}

/// `Send` subcommand 本体: registry を引き、peer の FIFO に text を書き込む。
pub fn send(peer: &str, text: &str, registry_path: Option<PathBuf>) -> Result<()> {
    let registry_dir = config::registry_dir(registry_path);
    let entry = registry::read_entry(&registry_dir, peer)?
        .ok_or_else(|| anyhow!("peer '{peer}' not found in registry {}", registry_dir.display()))?;
    let fifo = PathBuf::from(&entry.fifo_path);
    if !fifo.exists() {
        bail!("FIFO {} does not exist for peer '{peer}'", fifo.display());
    }
    // FIFO への書込。改行で行区切り（受信側は行単位で read）。
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(&fifo)
        .with_context(|| format!("open FIFO for write: {}", fifo.display()))?;
    file.write_all(text.as_bytes())?;
    if !text.ends_with('\n') {
        file.write_all(b"\n")?;
    }
    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn ensure_fifo_creates_fifo() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("test.fifo");
        ensure_fifo(&p).unwrap();
        // FIFO は通常ファイルとは異なる: stat で確認
        let meta = std::fs::metadata(&p).unwrap();
        use std::os::unix::fs::FileTypeExt;
        assert!(meta.file_type().is_fifo());
    }

    #[test]
    fn ensure_fifo_idempotent() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("test2.fifo");
        ensure_fifo(&p).unwrap();
        ensure_fifo(&p).unwrap();  // 二度目もエラーにならない
    }
}
