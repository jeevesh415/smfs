//! Local SQLite cache.
//!
//! Persists inodes, dentries, file chunks, and sync state between daemon
//! restarts. Schema is adapted from the AgentFS agent filesystem spec
//! with supermemoryfs-specific additions.
//!
//! The cache is a *passive store*: it never calls the API or spawns
//! background tasks. The sync engine (in [`crate::sync`]) is the only
//! thing that mutates sync-state fields (added in M7–M8).

mod db;
mod file;
mod fs;
pub mod profile;

pub use db::{Db, DEFAULT_CHUNK_SIZE, DENTRY_CACHE_MAX, ROOT_INO};
pub use fs::SupermemoryFs;

#[cfg(test)]
mod tests;
