use super::apple_logo::AppleLogo;
use crate::hooks::{use_release, RELEASES_URL};
use dioxus::prelude::*;

#[component]
pub fn Download() -> Element {
    let release = use_release();
    let r = release();
    let href = r.dmg_url.clone().unwrap_or_else(|| RELEASES_URL.to_string());
    let version = r.version.map(|v| format!("Version {v} · ")).unwrap_or_default();
    rsx! {
        div { class: "download rise", style: "--d: .75s",
            a { class: "btn btn-lg", href, download: r.dmg_url.is_some().then_some(""),
                "Download for macOS"
                AppleLogo {}
            }
            p { class: "muted small", "{version}macOS 14+ · Works best on MacBooks with a notch" }
        }
    }
}

/// Inline text link that downloads the DMG, for use inside sentences.
#[component]
pub fn DownloadLink(children: Element) -> Element {
    let release = use_release();
    let r = release();
    let href = r.dmg_url.clone().unwrap_or_else(|| RELEASES_URL.to_string());
    rsx! {
        a { class: "inline-link", href, download: r.dmg_url.is_some().then_some(""), {children} }
    }
}
