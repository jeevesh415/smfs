//! FUSE mount adapter (Linux only).
//!
//! Bridges [`fuser::Filesystem`] callbacks to our
//! [`vfs::FileSystem`](crate::vfs::FileSystem) trait methods. Each FUSE
//! callback is synchronous; we bridge to our async trait via
//! [`tokio::runtime::Handle::block_on`] on the fuser-managed callback
//! thread (which is not itself a tokio worker, so `block_on` is legal).
//!
//! ## Build gating
//!
//! This file is only compiled when `target_os = "linux"`. On macOS, the
//! `pub mod fuse;` declaration in the parent module is `#[cfg]`-gated out
//! and `fuser` isn't in the dependency tree at all.
//!
//! ## M3e scope
//!
//! This commit delivers the [`FuseAdapter`] struct plus a full
//! `impl fuser::Filesystem`. It does *not* actually mount anything:
//! `mount_fuse()` is still the M3b/M3c stub. M3f will wire it up.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fuser::consts::{
    FUSE_ASYNC_READ, FUSE_CACHE_SYMLINKS, FUSE_NO_OPENDIR_SUPPORT, FUSE_PARALLEL_DIROPS,
    FUSE_WRITEBACK_CACHE,
};
use fuser::{
    Filesystem, KernelConfig, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyOpen, ReplyStatfs, ReplyWrite, Request, TimeOrNow as FuserTimeOrNow,
};
use parking_lot::Mutex;

use crate::vfs::{BoxedFile, FileAttr, FileSystem, FileType, SetAttr, TimeOrNow, Timestamp};

use super::{MountHandle, MountOpts};

/// Attribute cache TTL. We use `Duration::MAX` because the daemon is the
/// only writer — there's no outside process that can invalidate the
/// kernel's dcache, so entries never expire on their own.
const TTL: Duration = Duration::from_secs(60 * 60 * 24 * 365);

// ─── Type conversion helpers ───────────────────────────────────────────────

/// Convert our [`FileType`] into fuser's wider `FileType` enum.
fn file_type_to_fuser(ft: FileType) -> fuser::FileType {
    match ft {
        FileType::Regular => fuser::FileType::RegularFile,
        FileType::Directory => fuser::FileType::Directory,
        FileType::Symlink => fuser::FileType::Symlink,
    }
}

/// Convert a VFS [`Timestamp`] into a `SystemTime` for fuser.
fn timestamp_to_system_time(ts: Timestamp) -> SystemTime {
    UNIX_EPOCH + Duration::new(ts.sec.max(0) as u64, ts.nsec)
}

/// Convert our [`FileAttr`] into fuser's wire-format `FileAttr` struct.
fn file_attr_to_fuser_attr(attr: &FileAttr) -> fuser::FileAttr {
    fuser::FileAttr {
        ino: attr.ino,
        size: attr.size,
        blocks: attr.blocks,
        atime: timestamp_to_system_time(attr.atime),
        mtime: timestamp_to_system_time(attr.mtime),
        ctime: timestamp_to_system_time(attr.ctime),
        crtime: timestamp_to_system_time(attr.ctime),
        kind: file_type_to_fuser(attr.file_type()),
        perm: (attr.mode & 0o7777) as u16,
        nlink: attr.nlink,
        uid: attr.uid,
        gid: attr.gid,
        rdev: attr.rdev,
        blksize: attr.blksize,
        flags: 0,
    }
}

/// Convert fuser's `TimeOrNow` into ours.
fn fuser_time_to_vfs_time(t: FuserTimeOrNow) -> TimeOrNow {
    match t {
        FuserTimeOrNow::Now => TimeOrNow::Now,
        FuserTimeOrNow::SpecificTime(st) => {
            let d = st.duration_since(UNIX_EPOCH).unwrap_or_default();
            TimeOrNow::Time(Timestamp {
                sec: d.as_secs() as i64,
                nsec: d.subsec_nanos(),
            })
        }
    }
}

// ─── FUSE adapter type ────────────────────────────────────────────────────

/// Adapter that implements [`fuser::Filesystem`] by delegating to our
/// [`crate::vfs::FileSystem`] trait.
///
/// FUSE callbacks run on fuser-managed threads (outside tokio), so we
/// carry a [`tokio::runtime::Handle`] and use `rt.block_on(async { ... })`
/// to call async trait methods synchronously from those threads. This
/// requires the caller of `mount_fuse` to be inside a tokio runtime
/// context when constructing the adapter.
pub struct FuseAdapter<F: FileSystem + 'static> {
    fs: Arc<F>,
    rt: tokio::runtime::Handle,
    open_files: Arc<Mutex<HashMap<u64, BoxedFile>>>,
    next_fh: Arc<AtomicU64>,
    default_uid: u32,
    default_gid: u32,
}

impl<F: FileSystem + 'static> FuseAdapter<F> {
    /// Construct a new adapter wrapping the given filesystem.
    ///
    /// `rt` must be a handle to a tokio runtime that will be alive for
    /// the lifetime of this adapter. `default_uid`/`default_gid` are used
    /// for ownership of files/dirs/symlinks created through the mount
    /// when the caller doesn't specify otherwise.
    pub fn new(fs: Arc<F>, rt: tokio::runtime::Handle, default_uid: u32, default_gid: u32) -> Self {
        Self {
            fs,
            rt,
            open_files: Arc::new(Mutex::new(HashMap::new())),
            next_fh: Arc::new(AtomicU64::new(1)),
            default_uid,
            default_gid,
        }
    }

    /// Allocate a new unique file handle and store the backing
    /// [`BoxedFile`].
    fn register_handle(&self, file: BoxedFile) -> u64 {
        let fh = self.next_fh.fetch_add(1, Ordering::Relaxed);
        self.open_files.lock().insert(fh, file);
        fh
    }

    /// Look up the [`BoxedFile`] for an open handle.
    fn get_handle(&self, fh: u64) -> Option<BoxedFile> {
        self.open_files.lock().get(&fh).cloned()
    }

    /// Remove and drop the [`BoxedFile`] for a handle (on release).
    fn release_handle(&self, fh: u64) {
        self.open_files.lock().remove(&fh);
    }
}

// Manual `Debug` impl because `F: FileSystem` doesn't require `Debug`
// as a supertrait. Print only the fields we can safely show.
impl<F: FileSystem + 'static> std::fmt::Debug for FuseAdapter<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuseAdapter")
            .field("default_uid", &self.default_uid)
            .field("default_gid", &self.default_gid)
            .field("open_files_len", &self.open_files.lock().len())
            .finish_non_exhaustive()
    }
}

// ─── fuser::Filesystem implementation ─────────────────────────────────────

impl<F: FileSystem + 'static> Filesystem for FuseAdapter<F> {
    // ─── Lifecycle ─────────────────────────────────────────────────────

    fn init(&mut self, _req: &Request, config: &mut KernelConfig) -> Result<(), libc::c_int> {
        // Enable the same performance capabilities AgentFS enables.
        let _ = config.add_capabilities(
            FUSE_ASYNC_READ
                | FUSE_WRITEBACK_CACHE
                | FUSE_PARALLEL_DIROPS
                | FUSE_CACHE_SYMLINKS
                | FUSE_NO_OPENDIR_SUPPORT,
        );
        Ok(())
    }

    fn destroy(&mut self) {
        // Drop the whole open-file table on unmount. Each `BoxedFile`'s
        // Drop releases its underlying handle.
        self.open_files.lock().clear();
    }

    // ─── Name resolution + metadata ───────────────────────────────────

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let Some(name_str) = name.to_str() else {
            reply.error(libc::EINVAL);
            return;
        };
        let fs = self.fs.clone();
        let name_owned = name_str.to_string();
        let result = self
            .rt
            .block_on(async move { fs.lookup(parent, &name_owned).await });
        match result {
            Ok(Some(attr)) => reply.entry(&TTL, &file_attr_to_fuser_attr(&attr), 0),
            Ok(None) => reply.error(libc::ENOENT),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    fn forget(&mut self, _req: &Request, _ino: u64, _nlookup: u64) {
        // No-op: we don't reference-count kernel inode lookups.
    }

    fn getattr(&mut self, _req: &Request, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        let fs = self.fs.clone();
        let result = self.rt.block_on(async move { fs.getattr(ino).await });
        match result {
            Ok(Some(attr)) => reply.attr(&TTL, &file_attr_to_fuser_attr(&attr)),
            Ok(None) => reply.error(libc::ENOENT),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    #[allow(clippy::too_many_arguments)] // dictated by fuser's trait signature
    fn setattr(
        &mut self,
        _req: &Request,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<FuserTimeOrNow>,
        mtime: Option<FuserTimeOrNow>,
        _ctime: Option<SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        let set_attr = SetAttr {
            mode,
            uid,
            gid,
            size,
            atime: atime.map(fuser_time_to_vfs_time),
            mtime: mtime.map(fuser_time_to_vfs_time),
        };
        let fs = self.fs.clone();
        let result = self
            .rt
            .block_on(async move { fs.setattr(ino, set_attr).await });
        match result {
            Ok(attr) => reply.attr(&TTL, &file_attr_to_fuser_attr(&attr)),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    fn readlink(&mut self, _req: &Request, ino: u64, reply: ReplyData) {
        let fs = self.fs.clone();
        let result = self.rt.block_on(async move { fs.readlink(ino).await });
        match result {
            Ok(Some(target)) => reply.data(target.as_bytes()),
            Ok(None) => reply.error(libc::ENOENT),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    // ─── Directory operations ─────────────────────────────────────────

    fn mkdir(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        let Some(name_str) = name.to_str() else {
            reply.error(libc::EINVAL);
            return;
        };
        let name_owned = name_str.to_string();
        let fs = self.fs.clone();
        let uid = self.default_uid;
        let gid = self.default_gid;
        let result = self
            .rt
            .block_on(async move { fs.mkdir(parent, &name_owned, mode, uid, gid).await });
        match result {
            Ok(attr) => reply.entry(&TTL, &file_attr_to_fuser_attr(&attr), 0),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    fn rmdir(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let Some(name_str) = name.to_str() else {
            reply.error(libc::EINVAL);
            return;
        };
        let name_owned = name_str.to_string();
        let fs = self.fs.clone();
        let result = self
            .rt
            .block_on(async move { fs.rmdir(parent, &name_owned).await });
        match result {
            Ok(()) => reply.ok(),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    fn opendir(&mut self, _req: &Request, _ino: u64, _flags: i32, reply: ReplyOpen) {
        // FUSE_NO_OPENDIR_SUPPORT is enabled in init, but some kernels
        // may still call this. We don't track dir handles, so return
        // fh=0 and zero flags.
        reply.opened(0, 0);
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let fs = self.fs.clone();
        let result = self.rt.block_on(async move { fs.readdir_plus(ino).await });
        let entries = match result {
            Ok(Some(entries)) => entries,
            Ok(None) => {
                reply.error(libc::ENOTDIR);
                return;
            }
            Err(e) => {
                reply.error(e.to_errno());
                return;
            }
        };

        // FUSE readdir is offset-based. `offset` is the cursor the kernel
        // wants us to resume from; we return entries starting at that
        // offset. `reply.add` returns `true` when the reply buffer is
        // full, which is our signal to stop.
        for (i, entry) in entries.iter().enumerate().skip(offset as usize) {
            let next_offset = (i + 1) as i64;
            let full = reply.add(
                entry.attr.ino,
                next_offset,
                file_type_to_fuser(entry.attr.file_type()),
                &entry.name,
            );
            if full {
                break;
            }
        }
        reply.ok();
    }

    fn releasedir(&mut self, _req: &Request, _ino: u64, _fh: u64, _flags: i32, reply: ReplyEmpty) {
        reply.ok();
    }

    // ─── File operations (handle-based) ───────────────────────────────

    fn open(&mut self, _req: &Request, ino: u64, flags: i32, reply: ReplyOpen) {
        let fs = self.fs.clone();
        let result = self.rt.block_on(async move { fs.open(ino, flags).await });
        match result {
            Ok(file) => {
                let fh = self.register_handle(file);
                reply.opened(fh, 0);
            }
            Err(e) => reply.error(e.to_errno()),
        }
    }

    #[allow(clippy::too_many_arguments)] // fuser trait shape
    fn create(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        let Some(name_str) = name.to_str() else {
            reply.error(libc::EINVAL);
            return;
        };
        let name_owned = name_str.to_string();
        let fs = self.fs.clone();
        let uid = self.default_uid;
        let gid = self.default_gid;
        let result = self
            .rt
            .block_on(async move { fs.create_file(parent, &name_owned, mode, uid, gid).await });
        match result {
            Ok((attr, file)) => {
                let fh = self.register_handle(file);
                reply.created(&TTL, &file_attr_to_fuser_attr(&attr), 0, fh, 0);
            }
            Err(e) => reply.error(e.to_errno()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn read(
        &mut self,
        _req: &Request,
        _ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        let Some(file) = self.get_handle(fh) else {
            reply.error(libc::EBADF);
            return;
        };
        let result = self
            .rt
            .block_on(async move { file.read(offset as u64, size as usize).await });
        match result {
            Ok(data) => reply.data(&data),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn write(
        &mut self,
        _req: &Request,
        _ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        let Some(file) = self.get_handle(fh) else {
            reply.error(libc::EBADF);
            return;
        };
        let data_owned = data.to_vec();
        let result = self
            .rt
            .block_on(async move { file.write(offset as u64, &data_owned).await });
        match result {
            Ok(written) => reply.written(written),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    fn flush(&mut self, _req: &Request, _ino: u64, fh: u64, _lock_owner: u64, reply: ReplyEmpty) {
        let Some(file) = self.get_handle(fh) else {
            reply.error(libc::EBADF);
            return;
        };
        let result = self.rt.block_on(async move { file.flush().await });
        match result {
            Ok(()) => reply.ok(),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    fn release(
        &mut self,
        _req: &Request,
        _ino: u64,
        fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        self.release_handle(fh);
        reply.ok();
    }

    fn fsync(&mut self, _req: &Request, _ino: u64, fh: u64, _datasync: bool, reply: ReplyEmpty) {
        let Some(file) = self.get_handle(fh) else {
            reply.error(libc::EBADF);
            return;
        };
        let result = self.rt.block_on(async move { file.fsync().await });
        match result {
            Ok(()) => reply.ok(),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    // ─── Remove + rename ─────────────────────────────────────────────

    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let Some(name_str) = name.to_str() else {
            reply.error(libc::EINVAL);
            return;
        };
        let name_owned = name_str.to_string();
        let fs = self.fs.clone();
        let result = self
            .rt
            .block_on(async move { fs.unlink(parent, &name_owned).await });
        match result {
            Ok(()) => reply.ok(),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    fn rename(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        _flags: u32,
        reply: ReplyEmpty,
    ) {
        let (Some(old), Some(new)) = (name.to_str(), newname.to_str()) else {
            reply.error(libc::EINVAL);
            return;
        };
        let old_owned = old.to_string();
        let new_owned = new.to_string();
        let fs = self.fs.clone();
        let result = self
            .rt
            .block_on(async move { fs.rename(parent, &old_owned, newparent, &new_owned).await });
        match result {
            Ok(()) => reply.ok(),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    // ─── Symbolic + hard links ────────────────────────────────────────

    fn symlink(
        &mut self,
        _req: &Request,
        parent: u64,
        link_name: &OsStr,
        target: &std::path::Path,
        reply: ReplyEntry,
    ) {
        let Some(name_str) = link_name.to_str() else {
            reply.error(libc::EINVAL);
            return;
        };
        let Some(target_str) = target.to_str() else {
            reply.error(libc::EINVAL);
            return;
        };
        let name_owned = name_str.to_string();
        let target_owned = target_str.to_string();
        let fs = self.fs.clone();
        let uid = self.default_uid;
        let gid = self.default_gid;
        let result = self.rt.block_on(async move {
            fs.symlink(parent, &name_owned, &target_owned, uid, gid)
                .await
        });
        match result {
            Ok(attr) => reply.entry(&TTL, &file_attr_to_fuser_attr(&attr), 0),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    fn link(
        &mut self,
        _req: &Request,
        ino: u64,
        newparent: u64,
        newname: &OsStr,
        reply: ReplyEntry,
    ) {
        let Some(name_str) = newname.to_str() else {
            reply.error(libc::EINVAL);
            return;
        };
        let name_owned = name_str.to_string();
        let fs = self.fs.clone();
        let result = self
            .rt
            .block_on(async move { fs.link(ino, newparent, &name_owned).await });
        match result {
            Ok(attr) => reply.entry(&TTL, &file_attr_to_fuser_attr(&attr), 0),
            Err(e) => reply.error(e.to_errno()),
        }
    }

    // ─── Filesystem-wide ──────────────────────────────────────────────

    fn statfs(&mut self, _req: &Request, _ino: u64, reply: ReplyStatfs) {
        let fs = self.fs.clone();
        let result = self.rt.block_on(async move { fs.statfs().await });
        match result {
            Ok(stats) => {
                reply.statfs(
                    stats.bytes_used / 4096, // blocks
                    u64::MAX / 2,            // bfree
                    u64::MAX / 2,            // bavail
                    stats.inodes,            // files
                    u64::MAX / 2,            // ffree
                    4096,                    // bsize
                    255,                     // namelen
                    4096,                    // frsize
                );
            }
            Err(e) => reply.error(e.to_errno()),
        }
    }

    fn mknod(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _umask: u32,
        _rdev: u32,
        reply: ReplyEntry,
    ) {
        // Supermemory has no FIFOs, character devices, block devices,
        // or sockets. Return ENOSYS for any such attempt.
        reply.error(libc::ENOSYS);
    }
}

// ─── mount_fuse stub — still unchanged until M3f ─────────────────────────

/// Mount a filesystem using the FUSE backend (Linux only).
///
/// Stub for M3b/M3c/M3d/M3e — the real implementation lands in M3f
/// (adapter wiring + `fuser::spawn_mount2` invocation). Currently
/// always returns "not implemented".
#[allow(clippy::needless_pass_by_value)] // signature matches the eventual real one
pub async fn mount_fuse<F>(fs: Arc<F>, opts: MountOpts) -> anyhow::Result<MountHandle>
where
    F: FileSystem + 'static,
{
    let _ = (fs, opts);
    anyhow::bail!("FUSE mount not implemented yet — lands in M3f")
}

// TODO(M3f): build MountOption vec, call fuser::spawn_mount2 inside
// spawn_blocking, store BackgroundSession in MountHandleInner::Fuse.
