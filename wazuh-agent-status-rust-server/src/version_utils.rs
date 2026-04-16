//! Version comparison and remote manifest fetching utilities.

use reqwest::Client;
use std::time::Duration;

use crate::models::VersionInfo;

/// Returns `true` if `online` is a higher version than `local`.
///
/// Handles `v`-prefix stripping, pre-release suffixes, and segment-length
/// differences (e.g. `4.7` vs `4.7.2`).
pub fn is_version_higher(online: &str, local: &str) -> bool {
    let online = online.trim_start_matches('v');
    let local  = local.trim_start_matches('v');

    let online_base = online.split('-').next().unwrap_or(online);
    let local_base  = local.split('-').next().unwrap_or(local);

    let online_parts: Vec<u32> = online_base.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    let local_parts:  Vec<u32> = local_base.split('.').map(|p| p.parse().unwrap_or(0)).collect();

    let len = online_parts.len().max(local_parts.len());
    for i in 0..len {
        let o = *online_parts.get(i).unwrap_or(&0);
        let l = *local_parts.get(i).unwrap_or(&0);
        if o > l { return true; }
        if o < l { return false; }
    }

    // Same base — stable beats prerelease
    let online_is_pre = online.contains('-');
    let local_is_pre  = local.contains('-');
    matches!((online_is_pre, local_is_pre), (false, true))
}

/// Fetch and deserialise the remote [`VersionInfo`] manifest.
///
/// Returns `None` on any network or parse failure (errors are logged at warn
/// level internally by the caller).
pub async fn fetch_version_info(url: &str) -> Option<VersionInfo> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;

    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json::<VersionInfo>().await.ok()
}

/// Returns `true` if any of the agent's groups matches a prerelease test group
/// in the manifest (case-insensitive).
pub fn should_show_prerelease(version_info: &VersionInfo, agent_groups: &[String]) -> bool {
    if version_info.prerelease_test_groups.is_empty() || agent_groups.is_empty() {
        return false;
    }

    agent_groups.iter().any(|ag| {
        version_info
            .prerelease_test_groups
            .iter()
            .any(|tg| ag.eq_ignore_ascii_case(tg))
    })
}

#[cfg(test)]
mod tests {
    use super::is_version_higher;

    #[test]
    fn higher_major_version() {
        assert!(is_version_higher("5.0.0", "4.9.9"));
    }

    #[test]
    fn same_version_is_not_higher() {
        assert!(!is_version_higher("4.7.2", "4.7.2"));
    }

    #[test]
    fn stable_beats_prerelease_of_same_base() {
        assert!(is_version_higher("4.7.2", "4.7.2-rc1"));
    }

    #[test]
    fn prerelease_does_not_beat_stable() {
        assert!(!is_version_higher("4.7.2-rc1", "4.7.2"));
    }

    #[test]
    fn v_prefix_stripped() {
        assert!(is_version_higher("v5.0.0", "v4.0.0"));
    }
}
