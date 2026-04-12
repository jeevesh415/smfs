//! Conformance tests for SupermemoryFs — mirrors the MemFs tests in vfs/mem.rs.
//!
//! Each test creates a fresh in-memory SQLite database, constructs a
//! SupermemoryFs, and exercises the same operations as the MemFs conformance
//! suite. If these tests pass, SupermemoryFs is a drop-in replacement.

use std::sync::Arc;

use super::db::Db;
use super::fs::SupermemoryFs;
use crate::vfs::mode::{S_IFDIR, S_IFMT, S_IFREG};
use crate::vfs::traits::FileSystem;
use crate::vfs::types::{SetAttr, TimeOrNow, Timestamp};
use crate::vfs::VfsError;

const UID: u32 = 1000;
const GID: u32 = 1000;
const ROOT: u64 = 1;

fn fs() -> SupermemoryFs {
    let db = Arc::new(Db::open_in_memory().unwrap());
    SupermemoryFs::new(db)
}

// ─── Root and sanity ────────────────────────────────────────────────

#[tokio::test]
async fn test_root_exists() {
    let fs = fs();
    let attr = fs.getattr(ROOT).await.unwrap().expect("root must exist");
    assert!(attr.is_directory());
    assert_eq!(attr.ino, ROOT);
}

#[tokio::test]
async fn test_root_readdir_empty() {
    let fs = fs();
    let names = fs.readdir(ROOT).await.unwrap().unwrap();
    assert!(names.is_empty());
}

#[tokio::test]
async fn test_getattr_nonexistent_returns_none() {
    let fs = fs();
    assert!(fs.getattr(999).await.unwrap().is_none());
}

#[tokio::test]
async fn test_lookup_in_empty_root_returns_none() {
    let fs = fs();
    assert!(fs.lookup(ROOT, "nope").await.unwrap().is_none());
}

// ─── Directory creation and removal ─────────────────────────────────

#[tokio::test]
async fn test_mkdir_creates_entry() {
    let fs = fs();
    let dir = fs.mkdir(ROOT, "foo", 0o755, UID, GID).await.unwrap();
    assert!(dir.is_directory());

    let names = fs.readdir(ROOT).await.unwrap().unwrap();
    assert_eq!(names, vec!["foo".to_string()]);
}

#[tokio::test]
async fn test_mkdir_returns_correct_attr() {
    let fs = fs();
    let dir = fs.mkdir(ROOT, "foo", 0o755, UID, GID).await.unwrap();
    assert_eq!(dir.mode & S_IFMT, S_IFDIR);
    assert_eq!(dir.mode & 0o777, 0o755);
    assert_eq!(dir.uid, UID);
    assert_eq!(dir.gid, GID);
    assert_eq!(dir.nlink, 2);
}

#[tokio::test]
async fn test_mkdir_same_name_twice_fails() {
    let fs = fs();
    fs.mkdir(ROOT, "foo", 0o755, UID, GID).await.unwrap();
    let err = fs.mkdir(ROOT, "foo", 0o755, UID, GID).await.unwrap_err();
    assert!(matches!(err, VfsError::AlreadyExists));
}

#[tokio::test]
async fn test_rmdir_empty_works() {
    let fs = fs();
    fs.mkdir(ROOT, "tmp", 0o755, UID, GID).await.unwrap();
    fs.rmdir(ROOT, "tmp").await.unwrap();
    assert!(fs.lookup(ROOT, "tmp").await.unwrap().is_none());
}

#[tokio::test]
async fn test_rmdir_nonempty_returns_not_empty() {
    let fs = fs();
    let dir = fs.mkdir(ROOT, "d", 0o755, UID, GID).await.unwrap();
    fs.create_file(dir.ino, "inside", 0o644, UID, GID)
        .await
        .unwrap();
    let err = fs.rmdir(ROOT, "d").await.unwrap_err();
    assert!(matches!(err, VfsError::NotEmpty));
}

#[tokio::test]
async fn test_rmdir_nonexistent_returns_not_found() {
    let fs = fs();
    let err = fs.rmdir(ROOT, "nope").await.unwrap_err();
    assert!(matches!(err, VfsError::NotFound));
}

// ─── Regular files ──────────────────────────────────────────────────

#[tokio::test]
async fn test_create_file_returns_handle_and_attr() {
    let fs = fs();
    let (attr, handle) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    assert!(attr.is_file());
    assert_eq!(attr.mode & 0o777, 0o644);
    assert_eq!(attr.size, 0);
    let empty = handle.read(0, 100).await.unwrap();
    assert!(empty.is_empty());
}

#[tokio::test]
async fn test_write_then_read_round_trip() {
    let fs = fs();
    let (_, handle) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    let n = handle.write(0, b"hello world").await.unwrap();
    assert_eq!(n, 11);
    let data = handle.read(0, 100).await.unwrap();
    assert_eq!(data, b"hello world");
}

#[tokio::test]
async fn test_write_at_offset_extends_file() {
    let fs = fs();
    let (_, handle) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    handle.write(10, b"hi").await.unwrap();
    let attr = handle.getattr().await.unwrap();
    assert_eq!(attr.size, 12);
    let data = handle.read(0, 100).await.unwrap();
    assert_eq!(&data[10..12], b"hi");
    assert_eq!(&data[0..10], &[0; 10]);
}

#[tokio::test]
async fn test_read_past_eof_returns_empty() {
    let fs = fs();
    let (_, handle) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    handle.write(0, b"abc").await.unwrap();
    let data = handle.read(100, 10).await.unwrap();
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_read_empty_file_returns_empty() {
    let fs = fs();
    let (_, handle) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    let data = handle.read(0, 100).await.unwrap();
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_create_file_same_name_twice_fails() {
    let fs = fs();
    fs.create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    let err = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap_err();
    assert!(matches!(err, VfsError::AlreadyExists));
}

#[tokio::test]
async fn test_unlink_removes_entry() {
    let fs = fs();
    fs.create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    fs.unlink(ROOT, "a.txt").await.unwrap();
    assert!(fs.lookup(ROOT, "a.txt").await.unwrap().is_none());
}

#[tokio::test]
async fn test_unlink_nonexistent_returns_not_found() {
    let fs = fs();
    let err = fs.unlink(ROOT, "nope").await.unwrap_err();
    assert!(matches!(err, VfsError::NotFound));
}

#[tokio::test]
async fn test_unlink_directory_returns_is_a_directory() {
    let fs = fs();
    fs.mkdir(ROOT, "d", 0o755, UID, GID).await.unwrap();
    let err = fs.unlink(ROOT, "d").await.unwrap_err();
    assert!(matches!(err, VfsError::IsADirectory));
}

// ─── Readdir variants ───────────────────────────────────────────────

#[tokio::test]
async fn test_readdir_lists_all_children_sorted() {
    let fs = fs();
    fs.create_file(ROOT, "b.txt", 0o644, UID, GID)
        .await
        .unwrap();
    fs.create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    fs.mkdir(ROOT, "c", 0o755, UID, GID).await.unwrap();

    let names = fs.readdir(ROOT).await.unwrap().unwrap();
    assert_eq!(names, vec!["a.txt", "b.txt", "c"]);
}

#[tokio::test]
async fn test_readdir_on_file_returns_none() {
    let fs = fs();
    let (attr, _) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    assert!(fs.readdir(attr.ino).await.unwrap().is_none());
}

#[tokio::test]
async fn test_readdir_plus_includes_attrs() {
    let fs = fs();
    let (file_attr, _) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    let entries = fs.readdir_plus(ROOT).await.unwrap().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "a.txt");
    assert_eq!(entries[0].attr.ino, file_attr.ino);
}

// ─── Rename ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_rename_within_same_directory() {
    let fs = fs();
    let (_, handle) = fs
        .create_file(ROOT, "old.txt", 0o644, UID, GID)
        .await
        .unwrap();
    handle.write(0, b"hi").await.unwrap();

    fs.rename(ROOT, "old.txt", ROOT, "new.txt").await.unwrap();
    assert!(fs.lookup(ROOT, "old.txt").await.unwrap().is_none());
    let moved = fs.lookup(ROOT, "new.txt").await.unwrap().unwrap();
    assert_eq!(moved.size, 2);
}

#[tokio::test]
async fn test_rename_across_directories() {
    let fs = fs();
    let src_dir = fs.mkdir(ROOT, "src", 0o755, UID, GID).await.unwrap();
    let dst_dir = fs.mkdir(ROOT, "dst", 0o755, UID, GID).await.unwrap();
    fs.create_file(src_dir.ino, "f", 0o644, UID, GID)
        .await
        .unwrap();

    fs.rename(src_dir.ino, "f", dst_dir.ino, "f").await.unwrap();
    assert!(fs.lookup(src_dir.ino, "f").await.unwrap().is_none());
    assert!(fs.lookup(dst_dir.ino, "f").await.unwrap().is_some());
}

#[tokio::test]
async fn test_rename_nonexistent_returns_not_found() {
    let fs = fs();
    let err = fs.rename(ROOT, "nope", ROOT, "whatever").await.unwrap_err();
    assert!(matches!(err, VfsError::NotFound));
}

#[tokio::test]
async fn test_rename_over_existing_file_replaces() {
    let fs = fs();
    let (_, src_handle) = fs.create_file(ROOT, "src", 0o644, UID, GID).await.unwrap();
    src_handle.write(0, b"new").await.unwrap();
    fs.create_file(ROOT, "dst", 0o644, UID, GID).await.unwrap();

    fs.rename(ROOT, "src", ROOT, "dst").await.unwrap();
    assert!(fs.lookup(ROOT, "src").await.unwrap().is_none());
    let dst = fs.lookup(ROOT, "dst").await.unwrap().unwrap();
    assert_eq!(dst.size, 3);
}

// ─── Setattr ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_setattr_truncate_via_size() {
    let fs = fs();
    let (attr, handle) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    handle.write(0, b"hello world").await.unwrap();
    let updated = fs
        .setattr(
            attr.ino,
            SetAttr {
                size: Some(5),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.size, 5);
    let data = handle.read(0, 100).await.unwrap();
    assert_eq!(data, b"hello");
}

#[tokio::test]
async fn test_setattr_chmod_via_mode() {
    let fs = fs();
    let (attr, _) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    let updated = fs
        .setattr(
            attr.ino,
            SetAttr {
                mode: Some(0o600),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.mode & 0o777, 0o600);
    assert_eq!(updated.mode & S_IFMT, S_IFREG);
}

#[tokio::test]
async fn test_setattr_chown_via_uid_gid() {
    let fs = fs();
    let (attr, _) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    let updated = fs
        .setattr(
            attr.ino,
            SetAttr {
                uid: Some(42),
                gid: Some(99),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.uid, 42);
    assert_eq!(updated.gid, 99);
}

#[tokio::test]
async fn test_setattr_utimens_via_mtime() {
    let fs = fs();
    let (attr, _) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    let target = Timestamp {
        sec: 1_700_000_000,
        nsec: 500,
    };
    let updated = fs
        .setattr(
            attr.ino,
            SetAttr {
                mtime: Some(TimeOrNow::Time(target)),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.mtime, target);
}

// ─── Symlinks ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_symlink_create_and_readlink() {
    let fs = fs();
    let attr = fs
        .symlink(ROOT, "link", "/some/target", UID, GID)
        .await
        .unwrap();
    assert!(attr.is_symlink());
    assert_eq!(attr.size, "/some/target".len() as u64);
    let target = fs.readlink(attr.ino).await.unwrap().unwrap();
    assert_eq!(target, "/some/target");
}

#[tokio::test]
async fn test_readlink_on_regular_file_returns_error() {
    let fs = fs();
    let (attr, _) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    let err = fs.readlink(attr.ino).await.unwrap_err();
    assert!(matches!(err, VfsError::NotASymlink));
}

// ─── Hard links ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_link_creates_second_name() {
    let fs = fs();
    let (attr, handle) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    handle.write(0, b"data").await.unwrap();

    let linked = fs.link(attr.ino, ROOT, "b.txt").await.unwrap();
    assert_eq!(linked.nlink, 2);

    let via_a = fs.lookup(ROOT, "a.txt").await.unwrap().unwrap();
    let via_b = fs.lookup(ROOT, "b.txt").await.unwrap().unwrap();
    assert_eq!(via_a.ino, via_b.ino);
}

#[tokio::test]
async fn test_unlink_one_name_keeps_other() {
    let fs = fs();
    let (attr, handle) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    handle.write(0, b"shared").await.unwrap();
    fs.link(attr.ino, ROOT, "b.txt").await.unwrap();

    fs.unlink(ROOT, "a.txt").await.unwrap();
    assert!(fs.lookup(ROOT, "a.txt").await.unwrap().is_none());

    let remaining = fs.lookup(ROOT, "b.txt").await.unwrap().unwrap();
    assert_eq!(remaining.size, 6);
    assert_eq!(remaining.nlink, 1);
}

// ─── statfs ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_statfs_counts_inodes_and_bytes() {
    let fs = fs();
    let (_, handle) = fs
        .create_file(ROOT, "a.txt", 0o644, UID, GID)
        .await
        .unwrap();
    handle.write(0, b"12345").await.unwrap();

    let stats = fs.statfs().await.unwrap();
    assert!(stats.inodes >= 2);
    assert_eq!(stats.bytes_used, 5);
}
