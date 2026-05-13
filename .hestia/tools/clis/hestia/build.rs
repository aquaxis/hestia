//! Phase 127 -- Obtain `git describe --tags --dirty` at build time and
//! embed it as the `HESTIA_BUILD_VERSION` env variable.
//!
//! - Tag-matched build (e.g. exactly at v0.1.5 commit): `0.1.5`
//! - Build with diffs from tag: `0.1.5-3-gabc1234`
//! - Dirty working tree: `0.1.5-3-gabc1234-dirty`
//!
//! If git is unavailable (outside a repo / git not installed), this does nothing.
//! main.rs falls back to `option_env!` -> `CARGO_PKG_VERSION` (= [workspace.package] version).

use std::process::Command;

fn main() {
    // Output watch directives so rebuilds trigger on tag creation / commit / branch switch.
    // - .git/HEAD: detects branch switches
    // - .git/logs/HEAD: appended on every commit/checkout operation (regardless of current branch)
    // - .git/refs/tags: detects new tag creation
    // Paths follow the Cargo convention of being relative to the manifest dir.
    println!("cargo:rerun-if-changed=../../../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../../../.git/logs/HEAD");
    println!("cargo:rerun-if-changed=../../../../.git/refs/tags");

    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty=-dirty"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // Strip leading 'v' prefix (e.g. "v0.1.5-3-gabc1234" -> "0.1.5-3-gabc1234")
            let normalized = raw.strip_prefix('v').unwrap_or(&raw);
            if !normalized.is_empty() {
                println!("cargo:rustc-env=HESTIA_BUILD_VERSION={normalized}");
            }
        }
    }
}