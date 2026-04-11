//! Daemon lifecycle and IPC.
//!
//! Manages the transition from short-lived CLI invocation to long-running
//! background process, plus the unix-domain-socket control channel that
//! `smfs status`, `smfs unmount`, and `smfs sync` use to talk to the running
//! daemon.
//!
//! ## Planned contents (M10)
//!
//! - `spawn::spawn_daemon` — fork via the `daemonize` crate, child re-execs
//!   with the hidden `daemon-inner` subcommand, parent waits for a readiness
//!   signal over a pipe before returning to the user
//! - `lifecycle` — startup sequencing, signal handling, graceful shutdown
//! - `ipc` — unix domain socket server/client. Protocol: JSON lines.
//!   Requests: `Status`, `SyncNow`, `Stop`. Responses: typed status info or
//!   error.
//! - Socket paths:
//!   - Linux: `$XDG_RUNTIME_DIR/smfs.sock` (fallback `/tmp/smfs-$UID.sock`)
//!   - macOS: `<cache_dir>/smfs.sock`
//!
//! See `.plan/v0-plan.md` milestone M10 for the detailed spec.

// TODO(M10): spawn, lifecycle, ipc submodules per module doc comment above.
