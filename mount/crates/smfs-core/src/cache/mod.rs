//! Local SQLite cache.
//!
//! Persists inodes, dentries, file chunks, and sync state between daemon
//! restarts. Schema is adapted from the AgentFS agent filesystem spec
//! (see `agentfs/SPEC.md`) with supermemoryfs-specific additions for
//! remote document mapping and a durable sync queue.
//!
//! The cache is a *passive store*: it never calls the API or spawns
//! background tasks. The sync engine (in [`crate::sync`]) is the only
//! thing that mutates sync-state fields.
//!
//! ## Planned contents (M5)
//!
//! - `schema.sql` embedded via `include_str!` covering:
//!   - AgentFS tables: `fs_config`, `fs_inode`, `fs_dentry`, `fs_data`, `fs_symlink`, `kv_store`
//!   - supermemoryfs extensions: `fs_remote`, `fs_dir_cache`, `sync_queue`
//!   - `schema_version` for forward migrations
//! - `Db` type backed by an `r2d2` SQLite connection pool in WAL mode
//! - Inode CRUD methods, chunked read/write, remote-mapping helpers,
//!   sync-queue helpers
//! - `SupermemoryFs` — the real `FileSystem` implementation that backs the
//!   mount once the cache exists
//!
//! See `.plan/v0-plan.md` milestone M5 for the detailed spec.

// TODO(M5): implement per module doc comment above.
