//! `smfs sync` — force a sync cycle now, without waiting for the background engine's poll interval.

use anyhow::{bail, Result};

pub async fn run() -> Result<()> {
    bail!("`smfs sync` not implemented yet (M10 — triggered via IPC to the running daemon)")
}
