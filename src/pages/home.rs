use crate::components::{
    apple_logo::AppleLogo,
    download::{Download, DownloadLink},
    icons::{Feature, FeatureIcon},
    notch_demo::NotchDemo,
    scroll_cue::ScrollCue,
};
use dioxus::prelude::*;

const FEATURES: [(FeatureIcon, &str, &str); 8] = [
    (FeatureIcon::Music, "Now playing", "everywhere"),
    (FeatureIcon::Pointer, "Click to", "jump back"),
    (FeatureIcon::Charge, "Charging", "ring"),
    (FeatureIcon::Lock, "Lock screen", "icon"),
    (FeatureIcon::Flip, "Cover flip", "on track change"),
    (FeatureIcon::Fullscreen, "Hides in", "fullscreen"),
    (FeatureIcon::Pause, "Lingers on", "pause"),
    (FeatureIcon::Pill, "Works without", "a notch"),
];

const PRINCIPLES: [(&str, &str); 4] = [
    ("< 50 MB", "of RAM. A single lightweight helper, nothing polling in the background."),
    ("0", "duplicates of what macOS already shows. No privacy dots, no volume HUD."),
    ("Signed", "Ed25519-verified updates. Only the maintainer can ship code to your Mac."),
    ("Open source", "The code is public, so you can see exactly what you’re running."),
];

#[component]
pub fn Home() -> Element {
    rsx! {
        main {
            section { class: "hero",
                h1 { class: "rise", style: "--d: .1s",
                    "A Dynamic Island"
                    br {}
                    "for your "
                    AppleLogo {}
                    "Mac."
                }
                p { class: "lead rise", style: "--d: .25s",
                    "Hevel turns the notch into a quiet, useful space for what’s playing and what’s charging."
                }
                NotchDemo {}
                Download {}
            }

            section { class: "section reveal", id: "features",
                h2 { "Small. Smooth. Native." }
                div { class: "features",
                    for (i , (icon , a , b)) in FEATURES.into_iter().enumerate() {
                        div { key: "{a}", class: "feature", style: "--i: {i}",
                            Feature { icon }
                            h3 {
                                "{a}"
                                br {}
                                "{b}"
                            }
                        }
                    }
                }
            }

            section { class: "section reveal", id: "principles",
                div { class: "principles",
                    for (big , text) in PRINCIPLES {
                        div { key: "{big}",
                            div { class: "big", "{big}" }
                            p { "{text}" }
                        }
                    }
                }
            }

            section { class: "section install reveal", id: "install",
                h2 { "Install in a minute" }
                ol {
                    li {
                        DownloadLink { "Download" }
                        " the DMG and drag Hevel to Applications."
                    }
                    li {
                        "The app is ad-hoc signed: on first launch, right-click it and choose "
                        b { "Open" }
                        "."
                    }
                    li { "Allow Automation access when asked, so Hevel can read Spotify and Music instantly." }
                }
            }
        }

        ScrollCue { target_id: "features" }
    }
}
