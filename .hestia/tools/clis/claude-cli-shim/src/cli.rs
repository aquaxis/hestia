//! agent-cli 互換のコマンドライン定義。

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "claude-cli-shim",
    version,
    about = "Claude Code wrapper that mimics agent-cli for hestia (Phase 113)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: SubCmd,
}

#[derive(Subcommand, Debug)]
pub enum SubCmd {
    /// Run a peer session, keeping a Claude Code process alive.
    Run {
        /// Path to the persona Markdown file (with YAML frontmatter).
        #[arg(long)]
        persona: PathBuf,
        /// Peer name (used for registry / FIFO / logs).
        #[arg(long)]
        name: String,
        /// Pass `--dangerously-skip-permissions` to claude.
        #[arg(long, default_value_t = false)]
        auto_approve_tools: bool,
        /// agent-cli 互換 (`--provider claude` 等)。shim では `claude` 固定だが受け取る。
        #[arg(long)]
        provider: Option<String>,
        /// Claude model id (例: `claude-opus-4-7`)。
        #[arg(long)]
        model: Option<String>,
        /// Override registry directory (default: `~/.local/share/claude-cli-shim/registry`).
        #[arg(long)]
        registry_path: Option<PathBuf>,
        /// Override log directory (default: `~/.local/share/claude-cli-shim/logs`).
        #[arg(long)]
        log_path: Option<PathBuf>,
    },
    /// Print the registry in agent-cli compatible columns
    /// (`ID NAME PROVIDER MODEL ROLE [SKILLS]`).
    List {
        #[arg(long)]
        registry_path: Option<PathBuf>,
    },
    /// Send a text message to a registered peer (writes to its FIFO).
    Send {
        /// Peer name to send to.
        peer: String,
        /// Message body (single argument; agent-cli compatibility).
        text: String,
        #[arg(long)]
        registry_path: Option<PathBuf>,
    },
    /// Show the supported backend list (agent-cli compatible).
    Providers,
    /// Run an environment health check.
    Doctor,
}
