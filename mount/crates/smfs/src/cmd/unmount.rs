//! `smfs unmount` — unmount a running supermemoryfs mount and stop its daemon.

use anyhow::{bail, Result};
use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Mountpoint to unmount. If omitted, unmounts the current shell's active mount.
    pub path: Option<PathBuf>,

    /// Force unmount even if the filesystem is busy (lazy unmount).
    #[arg(long)]
    pub force: bool,
}

pub async fn run(_args: Args) -> Result<()> {
    bail!("`smfs unmount` not implemented yet (M10 — daemon lifecycle + IPC)")
}
