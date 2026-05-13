//! claude-cli-shim -- Claude Code wrapper that mimics `agent-cli` for hestia.
//!
//! Wrapper layer introduced in Phase 113 (instructions.md v18).
//! Provides agent-cli compatible subcommands (`run` / `list` / `send` / `providers` / `doctor`)
//! and keeps a Claude Code (`claude`) CLI process running as a child subprocess
//! to implement persistent sessions.

use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod doctor;
mod ipc;
mod log;
mod registry;
mod session;
mod transcoder;

use cli::{Cli, SubCmd};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        SubCmd::Run {
            persona,
            name,
            auto_approve_tools,
            provider,
            model,
            registry_path,
            log_path,
        } => {
            session::run(session::RunOpts {
                persona,
                name,
                auto_approve_tools,
                provider,
                model,
                registry_path,
                log_path,
            })
            .await
        }
        SubCmd::List { registry_path } => registry::list(registry_path),
        SubCmd::Send {
            peer,
            text,
            registry_path,
        } => ipc::send(&peer, &text, registry_path),
        SubCmd::Providers => {
            println!("Active provider: claude");
            println!();
            println!("Supported backends:");
            println!("  - claude       model=claude-opus-4-7      env ANTHROPIC_API_KEY: {}",
                if std::env::var("ANTHROPIC_API_KEY").is_ok() { "set" } else { "NOT set" });
            Ok(())
        }
        SubCmd::Doctor => doctor::run(),
    }
}