//! Configuration and XDG paths.
//!
//! Resolves cache database location, log file paths, and IPC socket paths
//! per operating system. Uses the `directories` crate so we don't branch
//! on OS manually.
//!
//! ## Planned contents (M5)
//!
//! - `project_dirs()` — `ProjectDirs::from("ai", "supermemory", "supermemoryfs")`
//! - `cache_db_path(container_tag)` → `<cache_dir>/<container_tag>.db`
//!   (one database per container tag — mounts are per-tag and can't overlap)
//! - `daemon_log_path()` → `<cache_dir>/daemon.log`
//! - `ipc_socket_path()` → per-OS:
//!   - Linux: `$XDG_RUNTIME_DIR/smfs.sock` with `/tmp/smfs-$UID.sock` fallback
//!   - macOS: `<cache_dir>/smfs.sock`
//!
//! See `.plan/v0-plan.md` milestone M5 for the detailed spec.

// TODO(M5): project_dirs, cache_db_path, daemon_log_path, ipc_socket_path.
