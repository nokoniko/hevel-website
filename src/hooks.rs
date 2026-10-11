use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub use crate::web::dom::dismiss_on_outside;

pub const REPO: &str = "nokoniko/hevel";
pub const RELEASES_URL: &str = "https://github.com/nokoniko/hevel/releases/latest";

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Release {
    pub version: Option<String>,
    pub dmg_url: Option<String>,
}

/// Looks up the latest GitHub release on the server (cached for 10 minutes, so visitors never hit
/// GitHub's rate limit) and renders it into the HTML. Failures give an empty release, never an error.
#[server]
pub async fn latest_release() -> Result<Release, ServerFnError> {
    use std::{
        sync::Mutex,
        time::{Duration, Instant},
    };

    #[derive(Deserialize)]
    struct Gh {
        tag_name: Option<String>,
        #[serde(default)]
        assets: Vec<GhAsset>,
    }
    #[derive(Deserialize)]
    struct GhAsset {
        name: String,
        browser_download_url: String,
    }

    static CACHE: Mutex<Option<(Instant, Release)>> = Mutex::new(None);
    const TTL: Duration = Duration::from_secs(600);

    if let Some((at, release)) = CACHE.lock().unwrap().as_ref() {
        if at.elapsed() < TTL {
            return Ok(release.clone());
        }
    }

    let fetched = async {
        let gh: Gh = reqwest::Client::new()
            .get(format!("https://api.github.com/repos/{REPO}/releases/latest"))
            .header("User-Agent", "hevel-site")
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?
            .json()
            .await
            .ok()?;
        Some(Release {
            version: gh.tag_name.map(|t| t.trim_start_matches('v').to_string()).filter(|v| !v.is_empty()),
            dmg_url: gh.assets.into_iter().find(|a| a.name.ends_with(".dmg")).map(|a| a.browser_download_url),
        })
    }
    .await;

    match fetched {
        Some(release) => {
            *CACHE.lock().unwrap() = Some((Instant::now(), release.clone()));
            Ok(release)
        }
        // Serve a stale entry rather than nothing.
        None => Ok(CACHE.lock().unwrap().as_ref().map(|(_, r)| r.clone()).unwrap_or_default()),
    }
}

pub fn use_release() -> Signal<Release> {
    use_context::<Signal<Release>>()
}
