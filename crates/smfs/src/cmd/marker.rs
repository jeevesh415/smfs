//! Shared `.smfs` marker file reader.

use std::path::Path;

pub struct SmfsMarker {
    pub tag: String,
    pub api_url: String,
    pub mount_path: Option<String>,
}

pub fn parse_marker(content: &str) -> Option<SmfsMarker> {
    let mut tag = None;
    let mut url = None;
    let mut mount_path = None;
    for line in content.lines() {
        if let Some(v) = line.strip_prefix("container_tag=") {
            let v = v.trim();
            if !v.is_empty() {
                tag = Some(v.to_string());
            }
        }
        if let Some(v) = line.strip_prefix("api_url=") {
            url = Some(v.to_string());
        }
        if let Some(v) = line.strip_prefix("mount_path=") {
            mount_path = Some(v.to_string());
        }
    }
    Some(SmfsMarker {
        tag: tag?,
        api_url: url.unwrap_or_else(|| "https://api.supermemory.ai".to_string()),
        mount_path,
    })
}

/// Walk up from `start` looking for a `.smfs` marker file.
pub fn find_marker_from(start: &Path) -> Option<SmfsMarker> {
    let mut dir = start.to_path_buf();
    loop {
        let marker = dir.join(".smfs");
        if marker.exists() {
            let content = std::fs::read_to_string(&marker).ok()?;
            return parse_marker(&content);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Walk up from CWD looking for a `.smfs` marker file.
pub fn read_smfs_marker() -> Option<SmfsMarker> {
    let dir = std::env::current_dir().ok()?;
    find_marker_from(&dir)
}
