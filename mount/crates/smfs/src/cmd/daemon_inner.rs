//! `smfs daemon-inner` — hidden subcommand invoked after the CLI forks itself
//! into a background process.
//!
//! Users never type this directly. The `mount` subcommand forks via the
//! `daemonize` crate (M10), and the child re-execs the binary with this
//! subcommand to become the long-running daemon that owns the mount.

use anyhow::{bail, Result};
use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Mountpoint the daemon owns.
    #[arg(long)]
    pub mount: PathBuf,

    /// Container tag this daemon serves.
    #[arg(long)]
    pub container_tag: String,

    /// Supermemory API token (passed by the parent process).
    #[arg(long, hide_env_values = true)]
    pub token: Option<String>,

    /// Supermemory API base URL.
    #[arg(long)]
    pub api_url: Option<String>,

    /// File descriptor to signal readiness back to the parent process.
    #[arg(long)]
    pub ready_fd: Option<i32>,
}

pub async fn run(_args: Args) -> Result<()> {
    bail!("`smfs daemon-inner` not implemented yet (M10 — daemon lifecycle)")
}
