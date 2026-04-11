//! Background sync engine.
//!
//! Bridges the local SQLite cache and the Supermemory API. Handles both
//! directions:
//!
//! - **Pull** — on directory TTL expiry or cache miss, fetch from the API
//!   and populate inodes/chunks in the cache.
//! - **Push** — drain the `sync_queue` table, uploading dirty inodes to the
//!   API with exponential-backoff retry.
//!
//! The sync engine is a single long-running tokio task spawned alongside the
//! mount. It wakes up on explicit notification (when the VFS marks an inode
//! dirty) or on a poll interval (to discover work the notify path missed).
//!
//! ## Planned contents
//!
//! **M7 — pull path:**
//! - `sync_directory(db, api, dir_ino)` — list remote docs in a path prefix,
//!   reconcile with local inodes/dentries, update `fs_dir_cache.last_listed_at`
//! - `sync_file_content(db, api, ino)` — fetch remote content, replace local
//!   `fs_data` chunks, mark inode clean
//! - `/unfiled/{doc_id}` fallback for documents without a `filepath` field
//!
//! **M8 — push path:**
//! - `push_job(db, api, job)` — handle `Upload`/`Update`/`Delete`/`Rename` ops
//! - `SyncEngine::run(shutdown)` — the polling loop with `notify.notified()`
//!   and exponential backoff on failures
//! - State transitions: `dirty → pushing → clean | error`
//!
//! See `.plan/v0-plan.md` milestones M7 and M8 for the detailed spec.

// TODO(M7): pull path — sync_directory, sync_file_content.
// TODO(M8): push path — push_job, SyncEngine::run, backoff policy.
