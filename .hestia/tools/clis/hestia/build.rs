//! Phase 127 — `git describe --tags --dirty` を build 時に取得し、
//! `HESTIA_BUILD_VERSION` env として埋め込む。
//!
//! - tag 一致ビルド (例: v0.1.5 commit ちょうど): `0.1.5`
//! - tag からの diff があるビルド: `0.1.5-3-gabc1234`
//! - 作業ツリー dirty: `0.1.5-3-gabc1234-dirty`
//!
//! git 取得失敗時 (リポジトリ外 / git 不在) は何もしない。main.rs の
//! `option_env!` が `CARGO_PKG_VERSION` (= [workspace.package] version)
//! にフォールバックする。

use std::process::Command;

fn main() {
    // tag 作成 / commit / branch 切替で再ビルドが走るよう watch を出力。
    // - .git/HEAD: branch 切替を検知
    // - .git/logs/HEAD: 全 commit/checkout 操作で append される (現 branch 不問)
    // - .git/refs/tags: 新規 tag 作成を検知
    // パスは Cargo の慣例で manifest dir 相対。
    println!("cargo:rerun-if-changed=../../../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../../../.git/logs/HEAD");
    println!("cargo:rerun-if-changed=../../../../.git/refs/tags");

    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty=-dirty"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // 先頭 'v' プレフィクスを除去 (例: "v0.1.5-3-gabc1234" → "0.1.5-3-gabc1234")
            let normalized = raw.strip_prefix('v').unwrap_or(&raw);
            if !normalized.is_empty() {
                println!("cargo:rustc-env=HESTIA_BUILD_VERSION={normalized}");
            }
        }
    }
}
