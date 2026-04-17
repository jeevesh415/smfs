//! Shared API key resolution across all commands.

use anyhow::Result;
use std::path::Path;

/// Resolve API key with priority:
/// 1. explicit `--key` flag
/// 2. project-level stored credentials
/// 3. global stored credentials
/// 4. `SUPERMEMORY_API_KEY` env var
pub fn resolve_api_key(explicit_key: Option<&str>, mount_path: Option<&Path>) -> Result<String> {
    if let Some(k) = explicit_key {
        return Ok(k.to_string());
    }
    if let Some(creds) = smfs_core::config::credentials::resolve(mount_path) {
        return Ok(creds.api_key);
    }
    if let Ok(k) = std::env::var("SUPERMEMORY_API_KEY") {
        if !k.is_empty() {
            return Ok(k);
        }
    }
    anyhow::bail!("API key required. Run `smfs login` or pass --key.")
}
