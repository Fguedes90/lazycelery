//! Auto-update checking for LazyCelery
//!
//! This module checks if a new version is available on GitHub releases.

use serde::Deserialize;

/// Information about the latest release from GitHub
#[derive(Debug, Deserialize)]
pub struct ReleaseInfo {
    /// The tag name (e.g., "v0.7.2")
    pub tag_name: String,
    /// Release name
    pub name: Option<String>,
    /// URL to the release page
    pub html_url: String,
    /// Whether this is a pre-release
    pub prerelease: bool,
}

/// Check if a new version is available
///
/// Returns Some(new_version) if update available, None otherwise
pub async fn check_for_update(current_version: &str) -> Option<UpdateInfo> {
    // Parse current version
    let current = parse_version(current_version)?;

    // Fetch latest release from GitHub
    let client = reqwest::Client::builder()
        .user_agent("lazycelery")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let response = client
        .get("https://api.github.com/repos/Fguedes90/lazycelery/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let release: ReleaseInfo = response.json().await.ok()?;

    // Parse latest version from tag (remove 'v' prefix if present)
    let latest_tag = release.tag_name.trim_start_matches('v');
    let latest = parse_version(latest_tag)?;

    // Compare versions
    if latest > current {
        Some(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: latest_tag.to_string(),
            release_url: release.html_url,
            is_prerelease: release.prerelease,
        })
    } else {
        None
    }
}

/// Parse version string into comparable parts
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 3 {
        return None;
    }

    let major = parts[0].parse().ok()?;
    let minor = parts[1].parse().ok()?;
    let patch = parts[2]
        .split('-')
        .next()
        .and_then(|s| s.parse().ok())?;

    Some((major, minor, patch))
}

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Current version running
    pub current_version: String,
    /// Latest available version
    pub latest_version: String,
    /// URL to the release page
    pub release_url: String,
    /// Whether the latest version is a pre-release
    pub is_prerelease: bool,
}

impl UpdateInfo {
    /// Print update notification to stderr
    pub fn print_notification(&self) {
        eprintln!();
        if self.is_prerelease {
            eprintln!("⚠️  A new pre-release version is available!");
        } else {
            eprintln!("🔔 A new version of LazyCelery is available!");
        }
        eprintln!("   Current: v{}", self.current_version);
        eprintln!("   Latest:  v{}", self.latest_version);
        eprintln!("   {}", self.release_url);
        eprintln!();
        eprintln!("   To update, run:");
        eprintln!("   curl -sSL https://raw.githubusercontent.com/Fguedes90/lazycelery/main/install.sh | sh");
        eprintln!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("0.7.2"), Some((0, 7, 2)));
        assert_eq!(parse_version("1.0.0"), Some((1, 0, 0)));
        assert_eq!(parse_version("0.7.2-beta"), Some((0, 7, 2)));
        assert_eq!(parse_version("0.7"), None);
    }

    #[test]
    fn test_version_comparison() {
        let current = parse_version("0.7.2").unwrap();
        let newer = parse_version("0.7.3").unwrap();
        let older = parse_version("0.7.1").unwrap();
        let major_newer = parse_version("1.0.0").unwrap();

        assert!(newer > current);
        assert!(current > older);
        assert!(major_newer > current);
    }
}
