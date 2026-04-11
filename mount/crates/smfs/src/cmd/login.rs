//! `smfs login` — authenticate with Supermemory.
//!
//! In the normal flow, users run `supermemory login` from the existing
//! TypeScript CLI, which handles the browser OAuth / API key flow and
//! stores credentials. This subcommand exists for direct-from-`smfs` use
//! and for scripting, and will be wired up in M6 alongside the API client.

use anyhow::{bail, Result};
use clap::Args as ClapArgs;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Supermemory API token. If omitted, reads from the SUPERMEMORY_TOKEN environment variable.
    #[arg(long, env = "SUPERMEMORY_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    /// Override the Supermemory API base URL (defaults to production).
    #[arg(long, env = "SUPERMEMORY_API_URL")]
    pub api_url: Option<String>,
}

pub async fn run(_args: Args) -> Result<()> {
    bail!("`smfs login` not implemented yet (M6). In the meantime, use `supermemory login` from the TypeScript CLI.")
}
