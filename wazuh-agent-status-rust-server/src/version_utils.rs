use crate::models::VersionInfo;

pub fn is_version_higher(online: &str, local: &str) -> bool {
    let online = online.trim_start_matches('v');
    let local = local.trim_start_matches('v');

    let online_base = online.split('-').next().unwrap_or(online);
    let local_base = local.split('-').next().unwrap_or(local);

    let online_parts: Vec<&str> = online_base.split('.').collect();
    let local_parts: Vec<&str> = local_base.split('.').collect();

    for i in 0..std::cmp::min(online_parts.len(), local_parts.len()) {
        let online_num: u32 = online_parts[i].parse().unwrap_or(0);
        let local_num: u32 = local_parts[i].parse().unwrap_or(0);

        if online_num > local_num {
            return true;
        }
        if online_num < local_num {
            return false;
        }
    }

    if online_parts.len() != local_parts.len() {
        return online_parts.len() > local_parts.len();
    }

    let online_is_prerelease = online.contains('-');
    let local_is_prerelease = local.contains('-');

    if online_is_prerelease && !local_is_prerelease {
        return false;
    }
    if !online_is_prerelease && local_is_prerelease {
        return true;
    }

    false
}

pub async fn fetch_version_info(url: &str) -> Option<VersionInfo> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    match client.get(url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                resp.json::<VersionInfo>().await.ok()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn should_show_prerelease(version_info: &VersionInfo, agent_groups: &[String]) -> bool {
    if version_info.prerelease_test_groups.is_empty() || agent_groups.is_empty() {
        return false;
    }

    for agent_group in agent_groups {
        for test_group in &version_info.prerelease_test_groups {
            if agent_group.to_lowercase() == test_group.to_lowercase() {
                return true;
            }
        }
    }

    false
}
