//! `smfs mount` — mount a Supermemory container at a local path.
//!
//! In production this is usually invoked indirectly via
//! `supermemory mount <path> <tag>` from the TypeScript CLI, which reads the
//! user's stored credentials and execs this subcommand with `--token`.
//! It can also be used directly for scripting or debugging.

use anyhow::{bail, Result};
use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Path where the filesystem should be mounted (must exist).
    pub path: PathBuf,

    /// Supermemory container tag to mount. One mount per container tag;
    /// mounts cannot overlap or share a path.
    pub container_tag: String,

    /// Mount backend (`fuse` or `nfs`). Defaults to `fuse` on Linux and `nfs` on macOS.
    #[arg(long)]
    pub backend: Option<String>,

    /// Run the daemon in the foreground instead of detaching into the background.
    #[arg(long)]
    pub foreground: bool,

    /// Supermemory API token. Normally passed by the TS CLI; accepted here for direct use.
    #[arg(long, env = "SUPERMEMORY_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    /// Override the Supermemory API base URL.
    #[arg(long, env = "SUPERMEMORY_API_URL")]
    pub api_url: Option<String>,
}

pub async fn run(_args: Args) -> Result<()> {
    bail!("`smfs mount` not implemented yet (first real mount lands in M4; wired to SQLite + API in M5–M6)")
}
