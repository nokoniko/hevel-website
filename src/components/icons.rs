use dioxus::prelude::*;

#[component]
pub fn GithubIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", width: "22", height: "22", fill: "currentColor", "aria-hidden": "true",
            path { d: "M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" }
        }
    }
}

#[component]
pub fn FaqIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24", width: "22", height: "22", fill: "none", stroke: "currentColor",
            stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "M9.1 9a3 3 0 0 1 5.8 1c0 2-3 2.5-3 4" }
            path { d: "M12 17.5h.01" }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum FeatureIcon {
    Music,
    Pointer,
    Charge,
    Lock,
    Flip,
    Fullscreen,
    Pause,
    Pill,
}

impl FeatureIcon {
    fn name(self) -> &'static str {
        match self {
            Self::Music => "music",
            Self::Pointer => "pointer",
            Self::Charge => "charge",
            Self::Lock => "lock",
            Self::Flip => "flip",
            Self::Fullscreen => "fullscreen",
            Self::Pause => "pause",
            Self::Pill => "pill",
        }
    }
}

#[component]
pub fn Feature(icon: FeatureIcon) -> Element {
    let class = format!("ic ic-{}", icon.name());
    rsx! {
        svg {
            class, view_box: "0 0 64 64", width: "64", height: "64", "aria-hidden": "true",
            fill: "none", stroke: "currentColor", stroke_width: "2.4", stroke_linecap: "round", stroke_linejoin: "round",
            match icon {
                FeatureIcon::Music => rsx! {
                    path { d: "M26 46V18l22-5v28" }
                    circle { cx: "19", cy: "46", r: "7" }
                    circle { cx: "41", cy: "41", r: "7" }
                },
                FeatureIcon::Pointer => rsx! {
                    path { d: "M22 12v26l7-6 5 12 6-3-5-11h10z" }
                },
                FeatureIcon::Charge => rsx! {
                    circle { cx: "32", cy: "32", r: "22" }
                    path { d: "M35 17 24 34h8l-3 13 11-17h-8z", fill: "currentColor" }
                },
                FeatureIcon::Lock => rsx! {
                    rect { x: "12", y: "14", width: "40", height: "32", rx: "6" }
                    rect { x: "26", y: "26", width: "12", height: "11", rx: "2.5", fill: "currentColor" }
                    path { d: "M28 26v-3a4 4 0 0 1 8 0v3" }
                    path { d: "M24 54h16" }
                },
                FeatureIcon::Flip => rsx! {
                    rect { x: "20", y: "16", width: "24", height: "32", rx: "5" }
                    path { d: "M10 26c-2 4-2 8 0 12M54 26c2 4 2 8 0 12" }
                },
                FeatureIcon::Fullscreen => rsx! {
                    path { d: "M12 24V12h12M40 12h12v12M52 40v12H40M24 52H12V40" }
                    path { d: "M26 32h12" }
                },
                FeatureIcon::Pause => rsx! {
                    circle { cx: "32", cy: "32", r: "22" }
                    path { d: "M26 24v16M38 24v16" }
                },
                FeatureIcon::Pill => rsx! {
                    rect { x: "8", y: "10", width: "48", height: "30", rx: "6" }
                    rect { x: "22", y: "10", width: "20", height: "8", rx: "4", fill: "currentColor" }
                    path { d: "M20 50h24" }
                },
            }
        }
    }
}
