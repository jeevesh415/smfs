//! Mount backend abstraction.
//!
//! Unifies the FUSE (Linux) and NFS-over-localhost (macOS) mount paths behind
//! a single API: `MountOpts`, `MountHandle`, `mount_fs(fs, opts)`. The mount
//! backend is the *only* place in the codebase that knows about FUSE or NFS —
//! everything else talks to the `vfs::FileSystem` trait.
//!
//! ## Vendored upstream code
//!
//! The vendored `fuser` (MIT, Christopher Berner) and `nfsserve`
//! (BSD 3-Clause, XetData) crates will live as sibling modules under this
//! directory once M3 lands. We copy the source into our tree for the same
//! reasons AgentFS does: inherit their real-kernel debugging, avoid crates.io
//! breaking-release surprises, and retain the option to patch without
//! upstream round-trips.
//!
//! ## Planned contents (M3)
//!
//! - `src/mount/fuser/` — vendored from `agentfs/cli/src/fuser/` (Linux only)
//! - `src/mount/nfsserve/` — vendored from `agentfs/cli/src/nfsserve/` (unix)
//! - `MountOpts`, `MountHandle` (with RAII unmount-on-drop), `MountBackend`
//! - `pub async fn mount_fs(fs: Arc<Mutex<dyn FileSystem>>, opts: MountOpts) -> MountHandle`
//! - FUSE adapter: `impl fuser::Filesystem` bridging to `vfs::FileSystem`
//! - NFS adapter: `impl nfsserve::vfs::NFSFileSystem` bridging to `vfs::FileSystem`
//! - Platform-specific mount command exec:
//!   - macOS: `/sbin/mount_nfs -o locallocks,vers=3,tcp,port=X,...`
//!   - Linux: `mount -t nfs -o vers=3,tcp,port=X,...` or `fuser::spawn_mount2`
//!
//! See `.plan/v0-plan.md` milestone M3 for the detailed spec.

// TODO(M3): vendor fuser + nfsserve, implement adapter shims and unified API.
