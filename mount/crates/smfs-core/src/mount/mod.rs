//! Mount backend abstraction.
//!
//! Unifies the FUSE (Linux) and NFSv3-over-localhost (macOS) mount paths
//! behind a single API. This module is the *only* place in the codebase
//! that knows about FUSE or NFS — everything else talks to the
//! [`vfs::FileSystem`](crate::vfs::FileSystem) trait.
//!
//! ## Sub-modules
//!
//! - [`fuse`] — FUSE adapter, Linux only (via the `fuser` crate)
//! - [`nfs`] — NFSv3 adapter, unix-wide (via the `nfsserve` crate)
//!
//! ## Build status
//!
//! Currently M3a scaffolding: empty submodule stubs, no mount API yet.
//! The real adapter implementations and unified mount API land in
//! M3b–M3f as small sub-commits.

#[cfg(target_os = "linux")]
pub mod fuse;

#[cfg(unix)]
pub mod nfs;

// TODO(M3b): MountOpts, MountBackend, MountHandle, mount_fs()
// TODO(M3c-d): NFS adapter + mount_nfs exec
// TODO(M3e-f): FUSE adapter + mount_fuse exec
