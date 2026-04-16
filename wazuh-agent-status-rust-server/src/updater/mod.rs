//! Agent update orchestration.
//!
//! [`NativeUpdater`] is the single entry point regardless of platform.
//! Progress messages are streamed back as `UPDATE_PROGRESS: <msg>` strings
//! through a [`tokio::sync::mpsc::Sender`].

use std::sync::Arc;

use tokio::sync::mpsc;

use crate::config::AgentPaths;

// Platform-specific implementations ────────────────────────────────────────────

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

// ── NativeUpdater ─────────────────────────────────────────────────────────────

/// Runs agent update operations and streams progress to the caller.
///
/// Construct once and share via [`Arc`].
pub struct NativeUpdater {
    paths:       Arc<AgentPaths>,
    version_url: String,
}

impl NativeUpdater {
    pub fn new(paths: Arc<AgentPaths>, version_url: String) -> Self {
        Self { paths, version_url }
    }

    /// Execute an update and send `UPDATE_PROGRESS: <msg>` strings on `tx`.
    ///
    /// - `prerelease = false` → stable update
    /// - `prerelease = true`  → prerelease update
    pub async fn run_update(&self, prerelease: bool, tx: mpsc::Sender<String>) {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        unix::run_update(&self.paths, &self.version_url, prerelease, tx).await;

        #[cfg(target_os = "windows")]
        windows::run_update(&self.paths, &self.version_url, prerelease, tx).await;
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Format a progress string in the wire protocol format.
pub(crate) fn progress(msg: &str) -> String {
    format!("UPDATE_PROGRESS: {msg}\n")
}

/// Send a progress message, ignoring send errors (client may have disconnected).
pub(crate) async fn send_progress(tx: &mpsc::Sender<String>, msg: &str) {
    let _ = tx.send(progress(msg)).await;
}
