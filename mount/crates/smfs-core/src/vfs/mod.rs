//! Virtual filesystem trait and data model.
//!
//! Every filesystem operation supermemoryfs supports is defined here as an
//! async trait method. Mount adapters (FUSE, NFS) translate kernel callbacks
//! into trait method calls; storage backends (in-memory for tests, SQLite for
//! production) implement the trait.
//!
//! Keeping this module free of FUSE/NFS/SQLite concerns is deliberate — it
//! means the trait is the single source of truth for filesystem semantics, and
//! the same contract can be exercised against any backend without touching
//! mount code.
//!
//! ## Planned contents (M2)
//!
//! - `Inode`, `FileAttr`, `FileType`, `SetAttr`, `DirEntry` data types
//! - `VfsError` with an `errno` mapping
//! - Path normalization helpers (reject `..` escapes, strip redundant separators)
//! - `pub trait FileSystem` with: `lookup`, `getattr`, `setattr`, `readdir`,
//!   `read`, `write`, `create`, `unlink`, `mkdir`, `rmdir`, `rename`
//! - `MemVfs` — an in-memory reference implementation used by tests and by
//!   M4's first real mount demo
//!
//! See `.plan/v0-plan.md` milestone M2 for the detailed spec.

// TODO(M2): implement per module doc comment above.
