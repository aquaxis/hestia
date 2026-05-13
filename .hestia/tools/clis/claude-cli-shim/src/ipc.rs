//! FIFO-based inter-peer IPC.
//!
//! - On `Run` startup: creates `/tmp/claude-cli-shim-<peer>.fifo` via mkfifo (0600).
//! - Receiving: a separate task reads the FIFO asynchronously, pushing lines to a channel.
//! - Sending (`Send` subcommand): looks up the registry and writes to the corresponding FIFO.

use crate::{config, registry};
use anyhow::{anyhow, bail, Context, Result};
use std::ffi::CString;
use std::path::{Path, PathBuf};

/// Create a FIFO with mkfifo (ignore if it already exists).
pub fn ensure_fifo(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    let cstr = CString::new(path.as_os_str().to_string_lossy().as_bytes())
        .context("CString::new fifo path")?;
    let rc = unsafe { libc::mkfifo(cstr.as_ptr(), 0o600) };
    if rc != 0 {
        let err = std::io::Error::last_os_error();
        // EEXIST is harmless (another process created it first due to a race condition)
        if err.raw_os_error() != Some(libc::EEXIST) {
            return Err(anyhow!("mkfifo {} failed: {err}", path.display()));
        }
    }
    Ok(())
}

/// `Send` subcommand body: looks up the registry and writes text to the peer's FIFO.
pub fn send(peer: &str, text: &str, registry_path: Option<PathBuf>) -> Result<()> {
    let registry_dir = config::registry_dir(registry_path);
    let entry = registry::read_entry(&registry_dir, peer)?
        .ok_or_else(|| anyhow!("peer '{peer}' not found in registry {}", registry_dir.display()))?;
    let fifo = PathBuf::from(&entry.fifo_path);
    if !fifo.exists() {
        bail!("FIFO {} does not exist for peer '{peer}'", fifo.display());
    }
    // Write to FIFO. Newline-delimited (receiver reads line by line).
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
        // FIFOs differ from regular files: verify via stat
        let meta = std::fs::metadata(&p).unwrap();
        use std::os::unix::fs::FileTypeExt;
        assert!(meta.file_type().is_fifo());
    }

    #[test]
    fn ensure_fifo_idempotent() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("test2.fifo");
        ensure_fifo(&p).unwrap();
        ensure_fifo(&p).unwrap();  // Second call should not error
    }
}