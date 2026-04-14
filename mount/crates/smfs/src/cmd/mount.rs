//! `smfs mount` — mount a Supermemory container at a local path.
//!
//! In production this is usually invoked indirectly via
//! `supermemory mount <path> <tag>` from the TypeScript CLI, which reads the
//! user's stored credentials and execs this subcommand with `--token`.
//! It can also be used directly for scripting or debugging.

use anyhow::Result;
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

pub async fn run(args: Args) -> Result<()> {
    use smfs_core::cache::{Db, SupermemoryFs};
    use smfs_core::mount::{mount_fs, MountBackend, MountOpts};
    use std::sync::Arc;

    // 1. Parse backend (or use OS default).
    let backend = match &args.backend {
        Some(b) => b.parse::<MountBackend>()?,
        None => MountBackend::default(),
    };

    // 2. Create mountpoint if it doesn't exist.
    if !args.path.exists() {
        std::fs::create_dir_all(&args.path)?;
    }

    // 3. Get effective uid/gid of the calling user.
    #[allow(unsafe_code)]
    let (uid, gid) = unsafe { (libc::geteuid(), libc::getegid()) };

    // 4. Build MountOpts.
    let opts = MountOpts::new(args.path.clone(), backend).with_ownership(uid, gid);

    // 5. Open SQLite cache and create SupermemoryFs.
    let db_path = smfs_core::config::cache_db_path(&args.container_tag);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let db = Arc::new(Db::open(&db_path)?);

    let fs = Arc::new(match &args.token {
        Some(token) => {
            let api = Arc::new(smfs_core::api::ApiClient::new(
                args.api_url.as_deref().unwrap_or("https://api.supermemory.ai"),
                token,
                &args.container_tag,
            ));
            SupermemoryFs::with_api(db, api)
        }
        None => SupermemoryFs::new(db),
    });
    let handle = mount_fs(fs, opts).await?;

    eprintln!(
        "supermemoryfs mounted at {} (backend: {}, ctrl+c to unmount)",
        handle.mountpoint().display(),
        handle.backend(),
    );

    // 6. Hold mount until Ctrl+C.
    tokio::signal::ctrl_c().await?;
    eprintln!("\nunmounting...");

    drop(handle);
    Ok(())
}
