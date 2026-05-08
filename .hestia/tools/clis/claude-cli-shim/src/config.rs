//! Path 解決ヘルパ。
//!
//! 解決順:
//! 1. CLI 引数 (`--registry-path` / `--log-path`)
//! 2. 環境変数 (`CLAUDE_CLI_SHIM_REGISTRY_PATH` / `CLAUDE_CLI_SHIM_LOG_PATH`)
//! 3. 既定値 (`~/.local/share/claude-cli-shim/registry|logs`)

use std::path::PathBuf;

const DEFAULT_SUBDIR: &str = "claude-cli-shim";

pub fn registry_dir(override_path: Option<PathBuf>) -> PathBuf {
    if let Some(p) = override_path {
        return p;
    }
    if let Ok(p) = std::env::var("CLAUDE_CLI_SHIM_REGISTRY_PATH") {
        return PathBuf::from(p);
    }
    base_data_dir().join(DEFAULT_SUBDIR).join("registry")
}

pub fn log_dir(override_path: Option<PathBuf>) -> PathBuf {
    if let Some(p) = override_path {
        return p;
    }
    if let Ok(p) = std::env::var("CLAUDE_CLI_SHIM_LOG_PATH") {
        return PathBuf::from(p);
    }
    base_data_dir().join(DEFAULT_SUBDIR).join("logs")
}

pub fn fifo_path(peer: &str) -> PathBuf {
    PathBuf::from("/tmp").join(format!("claude-cli-shim-{peer}.fifo"))
}

fn base_data_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_dir_uses_override_when_present() {
        let p = registry_dir(Some(PathBuf::from("/custom/reg")));
        assert_eq!(p, PathBuf::from("/custom/reg"));
    }

    #[test]
    fn fifo_path_uses_peer_name() {
        let p = fifo_path("ai-designer");
        assert_eq!(p, PathBuf::from("/tmp/claude-cli-shim-ai-designer.fifo"));
    }

    #[test]
    fn registry_dir_default_under_data_dir() {
        // Override 無し、env 変数も無い前提（test 環境でクリーンならば確認）
        std::env::remove_var("CLAUDE_CLI_SHIM_REGISTRY_PATH");
        let p = registry_dir(None);
        assert!(p.ends_with("claude-cli-shim/registry"));
    }
}
