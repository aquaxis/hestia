//! 環境診断（agent-cli `doctor` 互換出力）。

use crate::config;
use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("[claude-cli-shim doctor]");

    // 1. claude バイナリ
    let claude_ok = match Command::new("claude").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout);
            println!("  claude binary  : OK  ({})", v.trim());
            true
        }
        _ => {
            println!("  claude binary  : MISSING (run `which claude`)");
            false
        }
    };

    // 2. ANTHROPIC_API_KEY
    let key_ok = std::env::var("ANTHROPIC_API_KEY").is_ok();
    println!(
        "  ANTHROPIC_API_KEY: {}",
        if key_ok { "set" } else { "NOT set" }
    );

    // 3. registry / log path
    let reg = config::registry_dir(None);
    let log = config::log_dir(None);
    println!("  registry path  : {}", reg.display());
    println!("  log path       : {}", log.display());

    // 4. data dir 書込権限
    let writable = check_writable(&reg);
    println!(
        "  registry writable: {}",
        if writable { "OK" } else { "NG" }
    );

    let overall = claude_ok && key_ok && writable;
    println!();
    println!(
        "Overall: {}",
        if overall { "OK" } else { "ISSUES DETECTED" }
    );
    Ok(())
}

fn check_writable(dir: &std::path::Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".write-probe");
    let r = std::fs::write(&probe, b"x").is_ok();
    let _ = std::fs::remove_file(&probe);
    r
}
