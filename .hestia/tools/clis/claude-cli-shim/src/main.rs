//! claude-cli-shim — Claude Code wrapper that mimics `agent-cli` for hestia.
//!
//! Phase 113 (instructions.md 第 18 版) で導入された wrapper レイヤ。
//! agent-cli 互換の subcommand (`run` / `list` / `send` / `providers` / `doctor`)
//! を提供し、内部で Claude Code (`claude`) CLI を子プロセスとして保持して
//! 永続 session を実現する。

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
